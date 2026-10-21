use std::sync::{Arc, Mutex, atomic::Ordering};
use crate::{app_lifecycle::AppLifecycle, app_lifecycle_runtime::{local_readiness, flush_state},
    runtime_readiness::{self, ReadinessState}, session::SessionManager, session_types::DeskState};
use tower::ServiceExt;

fn initialized() -> Arc<AppLifecycle> {
    let lifecycle = Arc::new(AppLifecycle::new("test", std::path::PathBuf::new()));
    lifecycle.tray_initialized.store(true, Ordering::Release);
    lifecycle.persistence_initialized.store(true, Ordering::Release);
    lifecycle.setup_finished();
    lifecycle
}

#[tokio::test]
async fn live_readiness_is_bounded_under_db_and_sensor_contention() {
    let lifecycle = initialized();
    let db = Arc::new(Mutex::new(Some(rusqlite::Connection::open_in_memory().unwrap())));
    let port = Arc::new(Mutex::new(Some("fixture".to_owned())));
    let read = {
        let (lifecycle, db, port) = (lifecycle.clone(), db.clone(), port.clone());
        Arc::new(move || local_readiness(&lifecycle, &db, &port, false))
    };
    assert_eq!(read().state, ReadinessState::Ready);
    let db_held = db.lock().unwrap();
    let port_held = port.lock().unwrap();
    let route_read = read.clone();
    let response = tokio::time::timeout(std::time::Duration::from_millis(500),
        tokio::task::spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(
                runtime_readiness::routes(route_read).oneshot(
                    axum::http::Request::builder().uri(runtime_readiness::PATH)
                        .body(axum::body::Body::empty()).unwrap()))
        })).await;
    // Release locks even on timeout so a blocking regression cannot hang teardown.
    drop(db_held); drop(port_held);
    let response = response.unwrap().unwrap().unwrap();
    let body = axum::body::to_bytes(response.into_body(), 4096).await.unwrap();
    let status: runtime_readiness::ReadinessStatus = serde_json::from_slice(&body).unwrap();
    assert!(status.ready);
    assert!(status.persistence_initialized);
    assert_eq!(status.state, ReadinessState::Degraded);
    assert_eq!(status.reason.as_deref(), Some("sensor_missing"));
    *port.lock().unwrap() = None;
    assert_eq!(read().state, ReadinessState::Degraded);
    assert!(read().ready);
    *db.lock().unwrap() = None;
    let status = read();
    assert!(!status.ready);
    assert!(!status.persistence_initialized);
    assert_eq!(status.reason.as_deref(), Some("persistence_not_ready"));
}

#[test]
fn db_contention_cannot_certify_uninitialized_or_failed_persistence() {
    let lifecycle = initialized();
    let db = Mutex::new(Some(rusqlite::Connection::open_in_memory().unwrap()));
    let port = Mutex::new(Some("fixture".to_owned()));
    lifecycle.persistence_initialized.store(false, Ordering::Release);
    // Even an available connection is insufficient without successful initialization.
    assert!(!local_readiness(&lifecycle, &db, &port, false).ready);
    let held = db.lock().unwrap();
    let status = local_readiness(&lifecycle, &db, &port, false);
    assert!(!status.ready);
    assert!(!status.persistence_initialized);
    lifecycle.persistence_initialized.store(true, Ordering::Release);
    lifecycle.failed.store(true, Ordering::Release);
    let status = local_readiness(&lifecycle, &db, &port, false);
    assert!(!status.ready);
    assert_eq!(status.state, ReadinessState::Failed);
    assert_eq!(status.reason.as_deref(), Some("observed_failure"));
    drop(held);
}

#[test]
fn poisoned_db_cannot_report_ready_despite_known_initialization() {
    let lifecycle = initialized();
    let db = Arc::new(Mutex::new(Some(rusqlite::Connection::open_in_memory().unwrap())));
    let poison_db = db.clone();
    assert!(std::thread::spawn(move || {
        let _held = poison_db.lock().unwrap();
        panic!("fixture: poison database mutex");
    }).join().is_err());
    let status = local_readiness(&lifecycle, &db, &Mutex::new(None), false);
    assert!(!status.ready);
    assert!(!status.persistence_initialized);
    assert_eq!(status.reason.as_deref(), Some("persistence_not_ready"));
}

fn sitting() -> Mutex<SessionManager> {
    let mut session = SessionManager::new();
    session.state.state = DeskState::Sitting;
    session.state.sitting_started = Some(chrono::Utc::now() - chrono::Duration::seconds(60));
    Mutex::new(session)
}

#[test]
fn shutdown_store_failure_never_commits_a_partial_history_row() {
    let session = sitting();
    let connection = rusqlite::Connection::open_in_memory().unwrap();
    crate::db::init_schema(&connection).unwrap();
    let db = Mutex::new(Some(connection));
    assert_eq!(flush_state(&session, &db, |_| "failed"), "store=failed database=not_attempted");
    let guard = db.lock().unwrap();
    let count: i64 = guard.as_ref().unwrap().query_row("SELECT COUNT(*) FROM sessions",
        [], |row| row.get(0)).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn shutdown_reports_skipped_and_failed_flush_without_invented_success() {
    let session = sitting();
    let db = Mutex::new(None);
    assert_eq!(flush_state(&session, &db, |_| "skipped_store_missing"), "store=skipped_store_missing database=not_attempted");
    assert!(flush_state(&session, &db, |_| "saved").contains("database=skipped_db_missing"));
    let held = db.lock().unwrap();
    assert!(flush_state(&session, &db, |_| "saved").contains("database=skipped_db_locked"));
    drop(held);
    *db.lock().unwrap() = Some(rusqlite::Connection::open_in_memory().unwrap()); // missing schema
    assert!(flush_state(&session, &db, |_| "saved").contains("database=failed"));
    let held = session.lock().unwrap();
    assert!(flush_state(&session, &db, |_| panic!("must not save")).contains("skipped_session_locked"));
    drop(held);
    assert!(flush_state(&Mutex::new(SessionManager::new()), &db, |_| "saved")
        .contains("database=not_needed"));
}

fn shutdown_database() -> Mutex<Option<rusqlite::Connection>> {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    crate::db::init_schema(&conn).unwrap();
    crate::db_sessions::insert_session(&conn, "2026-10-20T08:00:00Z", "2026-10-20T08:00:09Z", "Walking", 9).unwrap();
    Mutex::new(Some(conn))
}

#[test]
fn failed_flush_then_changed_session_retries_current_state_without_duplicate_history() {
    use crate::app_lifecycle_runtime::{FlushProgress, flush_state_progress};
    let session = sitting(); let db = shutdown_database(); let mut progress = FlushProgress::new();
    assert_eq!(flush_state_progress(&session, &db, &mut progress, |_| "failed"),
        "store=failed database=not_attempted");
    assert!(!progress.complete());
    session.lock().unwrap().state.sitting_seconds = 234;
    assert_eq!(flush_state_progress(&session, &db, &mut progress, |s| {
        assert_eq!(s.state.sitting_seconds, 234); "saved"
    }), "store=saved database=saved");
    assert!(progress.complete());
    assert_eq!(flush_state_progress(&session, &db, &mut progress, |_| panic!("already complete")),
        "store=saved database=saved");
    let conn = db.lock().unwrap();
    let rows: Vec<(String, Option<String>, i64)> = conn.as_ref().unwrap().prepare(
        "SELECT started_at, ended_at, duration_seconds FROM sessions ORDER BY started_at"
    ).unwrap().query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .unwrap().collect::<Result<_, _>>().unwrap();
    assert_eq!(rows.len(), 3, "shutdown retry must preserve both distinct known sitting spans");
    assert_eq!(rows[0], ("2026-10-20T08:00:00Z".into(), Some("2026-10-20T08:00:09Z".into()), 9));
    let sitting_rows: Vec<_> = rows.iter().filter(|(_, _, duration)| *duration != 9).collect();
    assert_eq!(sitting_rows.len(), 2);
    for (started, ended, duration) in &sitting_rows {
        let ended = ended.as_ref().expect("known sitting span has an end");
        let elapsed = (chrono::DateTime::parse_from_rfc3339(ended).unwrap()
            - chrono::DateTime::parse_from_rfc3339(started).unwrap()).num_seconds();
        assert_eq!(*duration, elapsed, "stored duration must match its observed endpoints");
    }
    assert!(sitting_rows[0].1.as_ref().unwrap() <= &sitting_rows[1].0,
        "known shutdown spans must not overlap");
    assert_eq!(conn.as_ref().unwrap().query_row("SELECT duration_seconds FROM sessions WHERE started_at='2026-10-20T08:00:00Z'", [], |r| r.get::<_, i64>(0)).unwrap(), 9);
}

#[test]
fn unfinished_and_cancelled_attempts_resave_manager_flags_and_reset_with_same_state() {
    use crate::app_lifecycle_runtime::{FlushProgress, flush_state_progress};
    let session = sitting(); let db = shutdown_database(); let mut progress = FlushProgress::new();
    let mut snapshots = vec![];
    let held = db.lock().unwrap();
    assert_eq!(flush_state_progress(&session, &db, &mut progress, |s| {
        snapshots.push(crate::session_persistence::PersistedEngineState::from_session(s, None)); "saved"
    }), "store=saved database=skipped_db_locked");
    drop(held);
    let after_first_flush = serde_json::to_value(&session.lock().unwrap().state).unwrap();
    session.lock().unwrap().notify_inactivity_fired = true;
    let after_inactivity = serde_json::to_value(&session.lock().unwrap().state).unwrap();
    assert_eq!(after_inactivity["sitting_seconds"], after_first_flush["sitting_seconds"]);
    assert_eq!(after_inactivity["sitting_seconds_total"], after_first_flush["sitting_seconds_total"]);
    assert_eq!(after_inactivity["sitting_started"], after_first_flush["sitting_started"]);
    assert!(session.lock().unwrap().notify_inactivity_fired);
    assert_eq!(flush_state_progress(&session, &db, &mut progress, |s| {
        snapshots.push(crate::session_persistence::PersistedEngineState::from_session(s, Some("reset-1".into()))); "saved"
    }), "store=saved database=saved");
    progress.cancel(&session, &db, |_| "saved").unwrap();
    session.lock().unwrap().notify_posture_balance_fired = true;
    assert!(session.lock().unwrap().pending_history.len() > 0,
        "cancel must return acknowledged history to the retry queue");
    flush_state_progress(&session, &db, &mut progress, |s| {
        snapshots.push(crate::session_persistence::PersistedEngineState::from_session(s, Some("reset-2".into()))); "saved"
    });
    assert_eq!(snapshots.len(), 5, "each successful database acknowledgement is bracketed by Store snapshots");
    assert!(!snapshots[0].notify_inactivity_fired);
    assert!(snapshots[1].notify_inactivity_fired);
    assert!(snapshots[2].notify_inactivity_fired);
    assert!(snapshots[3].notify_posture_balance_fired);
    assert!(snapshots[4].notify_posture_balance_fired);
    assert_eq!(snapshots[4].daily_reset_after.as_deref(), Some("reset-2"));
    let conn = db.lock().unwrap();
    let rows: Vec<(String, Option<String>, String, i64)> = conn.as_ref().unwrap().prepare(
        "SELECT started_at, ended_at, state, duration_seconds FROM sessions ORDER BY started_at, ended_at"
    ).unwrap().query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))
        .unwrap().collect::<Result<_, _>>().unwrap();
    assert_eq!(rows[0], ("2026-10-20T08:00:00Z".into(), Some("2026-10-20T08:00:09Z".into()), "Walking".into(), 9));
    assert!(rows.len() >= 3, "the foreign row and known shutdown history must survive retry/cancel: {rows:?}");
    assert!(rows.iter().skip(1).all(|(_, ended, state, duration)|
        ended.is_some() && state == "Sitting" && *duration >= 0));
    assert_eq!(rows.iter().skip(1).map(|(_, _, _, duration)| duration).sum::<i64>(),
        session.lock().unwrap().state.sitting_seconds_total,
        "persisted shutdown history must account for the manager's known raw sitting tail exactly once");
}

#[test]
fn cancellation_is_retryable_and_never_removes_an_unrelated_or_changed_row() {
    use crate::app_lifecycle_runtime::{FlushProgress, flush_state_progress};
    let session = sitting(); let db = shutdown_database(); let mut progress = FlushProgress::new();
    flush_state_progress(&session, &db, &mut progress, |_| "saved");
    let held = db.lock().unwrap();
    assert!(progress.cancel(&session, &db, |_| "saved").is_err());
    assert!(!progress.complete());
    drop(held);
    session.lock().unwrap().state.sitting_seconds = 333;
    flush_state_progress(&session, &db, &mut progress, |s| { assert_eq!(s.state.sitting_seconds, 333); "saved" });
    {
        let conn = db.lock().unwrap();
        conn.as_ref().unwrap().execute("UPDATE sessions SET duration_seconds=777 WHERE started_at != '2026-10-20T08:00:00Z'", []).unwrap();
    }
    assert!(progress.cancel(&session, &db, |_| "saved").is_err(),
        "identity changed; never delete the replacement");
    let conn = db.lock().unwrap();
    assert_eq!(conn.as_ref().unwrap().query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get::<_, i64>(0)).unwrap(), 3);
}

#[test]
fn failed_history_insert_keeps_store_ack_snapshot_for_retry_and_replay() {
    use crate::app_lifecycle_runtime::{FlushProgress, flush_state_progress};
    let start = chrono::Utc::now() - chrono::Duration::seconds(60);
    let mut manager = SessionManager::new();
    for _ in 0..crate::session_types::DEBOUNCE_COUNT {
        manager.on_reading_at(800, true, start);
    }
    manager.on_reading_at(800, true, start + chrono::Duration::seconds(50));
    let session = Mutex::new(manager);
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    crate::db::init_schema(&conn).unwrap();
    conn.execute_batch("CREATE TRIGGER reject_history BEFORE INSERT ON sessions
        BEGIN SELECT RAISE(FAIL, 'synthetic history failure'); END;").unwrap();
    let db = Mutex::new(Some(conn));
    let mut progress = FlushProgress::new();
    let mut pre_ack = None;

    assert_eq!(flush_state_progress(&session, &db, &mut progress, |manager| {
        pre_ack = Some(serde_json::to_value(
            crate::session_persistence::PersistedEngineState::from_session(manager, None),
        ).unwrap());
        "saved"
    }), "store=saved database=failed");
    assert!(!progress.complete());
    let pending = pre_ack.as_ref().unwrap()["pending_history"].as_array()
        .expect("the Store acknowledgement snapshot must carry pending history");
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0]["state"], "Sitting");

    db.lock().unwrap().as_ref().unwrap()
        .execute_batch("DROP TRIGGER reject_history;").unwrap();
    let snapshot = crate::session_persistence::PersistedEngineState::from_stored_value(
        pre_ack.clone().unwrap(),
    ).expect("pre-ack snapshot must be reloadable");
    let mut restored_manager = SessionManager::new();
    restored_manager.load_persisted_state(&snapshot);
    let restored = Mutex::new(restored_manager);
    let mut retry = FlushProgress::new();
    assert_eq!(flush_state_progress(&restored, &db, &mut retry, |_| "saved"),
        "store=saved database=saved");

    // A crash after the SQLite commit but before the Store acknowledgement replays this snapshot.
    let mut replay_manager = SessionManager::new();
    replay_manager.load_persisted_state(&snapshot);
    let replay = Mutex::new(replay_manager);
    let mut replay_progress = FlushProgress::new();
    assert_eq!(flush_state_progress(&replay, &db, &mut replay_progress, |_| "saved"),
        "store=saved database=saved");
    let guard = db.lock().unwrap();
    let rows: (i64, i64) = guard.as_ref().unwrap().query_row(
        "SELECT COUNT(*), COALESCE(SUM(duration_seconds), 0) FROM sessions", [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).unwrap();
    assert_eq!(rows.0, 1);
    assert!(rows.1 > 0);
}

fn r3_f02_flush_resume_case(state: DeskState, cancel_first_flush: bool) {
    use crate::app_lifecycle_runtime::{flush_state_progress_at, FlushProgress};
    use chrono::Timelike;

    let t100 = chrono::Utc::now().with_nanosecond(0).unwrap();
    let t0 = t100 - chrono::Duration::seconds(100);
    let mut manager = SessionManager::new();
    manager.state.state = state.clone();
    manager.history_span_started = Some(t0);
    manager.history_last_reading = Some(t100);
    manager.state.last_tick_ts = Some(t100);
    manager.state.last_accumulate_ts = Some(t100);
    match state {
        DeskState::Sitting => {
            manager.state.sitting_started = Some(t0);
            manager.state.sitting_seconds = 70;
            manager.state.sitting_seconds_total = 100;
        }
        DeskState::Standing => {
            manager.state.standing_bout_started = Some(t0);
            manager.state.standing_session_started = Some(t0);
            manager.state.standing_session_secs = 100;
            manager.state.standing_seconds = 20;
            manager.state.sitting_seconds = 40;
            manager.state.sitting_seconds_total = 50;
        }
        _ => panic!("the regression covers Sitting and Standing anchors"),
    }
    let session = Mutex::new(manager);
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    crate::db::init_schema(&conn).unwrap();
    if !cancel_first_flush {
        conn.execute_batch("CREATE TRIGGER reject_history BEFORE INSERT ON sessions
            BEGIN SELECT RAISE(FAIL, 'synthetic first shutdown failure'); END;").unwrap();
    }
    let db = Mutex::new(Some(conn));
    let mut progress = FlushProgress::new();
    let first = flush_state_progress_at(&session, &db, &mut progress, t100, |_| "saved");
    assert_eq!(first, if cancel_first_flush {
        "store=saved database=saved"
    } else {
        "store=saved database=failed"
    });
    if cancel_first_flush {
        progress.cancel(&session, &db, |_| "saved").unwrap();
    } else {
        db.lock().unwrap().as_ref().unwrap()
            .execute_batch("DROP TRIGGER reject_history;").unwrap();
    }

    let mm = if state == DeskState::Sitting { 600 } else { 1200 };
    for second in 1..=5 {
        session.lock().unwrap().on_reading_at(
            mm, true, t100 + chrono::Duration::seconds(second));
    }
    for second in 6..=10 {
        session.lock().unwrap().on_reading_at(
            mm, false, t100 + chrono::Duration::seconds(second));
    }
    let t110 = t100 + chrono::Duration::seconds(10);
    let mut snapshots = Vec::new();
    assert_eq!(flush_state_progress_at(&session, &db, &mut progress, t110, |manager| {
        snapshots.push(serde_json::to_value(
            crate::session_persistence::PersistedEngineState::from_session(manager, None),
        ).unwrap());
        "saved"
    }), "store=saved database=saved");

    let manager = session.lock().unwrap();
    assert_eq!(manager.state.state, DeskState::Away);
    match state {
        DeskState::Sitting => {
            assert_eq!(manager.state.sitting_seconds_total, 210,
                "raw Sitting must include the complete known [t100,t110) interval");
            assert_eq!(manager.state.sitting_seconds, 180,
                "credited Sitting advances by the same observed 10 seconds without new credit");
        }
        DeskState::Standing => {
            assert_eq!(manager.state.standing_seconds, 130,
                "raw Standing must include the complete known [t100,t110) interval");
            assert_eq!(manager.state.sitting_seconds, 40,
                "a Standing flush must not change credited Sitting");
        }
        _ => unreachable!(),
    }
    assert!(manager.pending_history.is_empty(), "successful acknowledgement drains pending history");
    drop(manager);

    let rows: Vec<(String, Option<String>, String, i64)> = db.lock().unwrap().as_ref().unwrap()
        .prepare("SELECT started_at, ended_at, state, duration_seconds FROM sessions ORDER BY started_at")
        .unwrap().query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))
        .unwrap().collect::<Result<_, _>>().unwrap();
    assert_eq!(rows.len(), 2, "retry/cancel must persist each known segment exactly once: {rows:?}");
    assert_eq!(rows[0], (t0.to_rfc3339(), Some(t100.to_rfc3339()),
        format!("{:?}", state), 100));
    assert_eq!(rows[1], (t100.to_rfc3339(), Some(t110.to_rfc3339()),
        format!("{:?}", state), 10));

    let pre_ack = crate::session_persistence::PersistedEngineState::from_stored_value(
        snapshots[0].clone()).expect("retry pre-ack snapshot must reload");
    let mut replay_manager = SessionManager::new();
    replay_manager.load_persisted_state(&pre_ack);
    let replay = Mutex::new(replay_manager);
    let mut replay_progress = FlushProgress::new();
    assert_eq!(flush_state_progress_at(&replay, &db, &mut replay_progress, t110, |_| "saved"),
        "store=saved database=saved");
    let count: i64 = db.lock().unwrap().as_ref().unwrap().query_row(
        "SELECT COUNT(*) FROM sessions", [], |row| row.get(0)).unwrap();
    assert_eq!(count, 2, "replaying the pre-ack snapshot must be idempotent");

    let saved = crate::session_persistence::PersistedEngineState::from_stored_value(
        snapshots.last().unwrap().clone()).expect("post-ack snapshot must reload");
    let mut restored = SessionManager::new();
    restored.load_persisted_state(&saved);
    assert!(restored.history_span_started.is_none());
    assert!(restored.state.sitting_started.is_none());
    assert!(restored.state.standing_bout_started.is_none());
    let raw_before_stop = restored.state.sitting_seconds_total;
    let standing_before_stop = restored.state.standing_seconds;
    let resume = t110 + chrono::Duration::hours(2);
    for offset in 1..=5 {
        restored.on_reading_at(mm, false, resume + chrono::Duration::seconds(offset));
    }
    assert_eq!(restored.state.sitting_seconds_total, raw_before_stop,
        "time while the app was stopped remains unknown Sitting time");
    assert_eq!(restored.state.standing_seconds, standing_before_stop,
        "time while the app was stopped remains unknown Standing time");
    assert_eq!(restored.history_span_started, Some(resume + chrono::Duration::seconds(5)),
        "history coverage restarts only when observations confirm the restored posture");
    assert!(restored.pending_history.is_empty());
}

#[test]
fn r3_f02_failed_flush_resume_preserves_sitting_raw_credited_and_history_tail() {
    r3_f02_flush_resume_case(DeskState::Sitting, false);
}

#[test]
fn r3_f02_cancelled_flush_resume_preserves_sitting_raw_credited_and_history_tail() {
    r3_f02_flush_resume_case(DeskState::Sitting, true);
}

#[test]
fn r3_f02_failed_flush_resume_preserves_standing_raw_and_history_tail() {
    r3_f02_flush_resume_case(DeskState::Standing, false);
}

#[test]
fn r3_f02_cancelled_flush_resume_preserves_standing_raw_and_history_tail() {
    r3_f02_flush_resume_case(DeskState::Standing, true);
}
