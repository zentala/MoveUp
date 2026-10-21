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
    crate::db_sessions::insert_session(&conn, "prior-start", "prior-end", "Walking", 9).unwrap();
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
    assert_eq!(conn.as_ref().unwrap().query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get::<_, i64>(0)).unwrap(), 2);
    assert_eq!(conn.as_ref().unwrap().query_row("SELECT duration_seconds FROM sessions WHERE started_at='prior-start'", [], |r| r.get::<_, i64>(0)).unwrap(), 9);
}

#[test]
fn unfinished_and_cancelled_attempts_resave_manager_flags_and_reset_with_same_state() {
    use crate::app_lifecycle_runtime::{FlushProgress, flush_state_progress};
    let session = sitting(); let db = shutdown_database(); let mut progress = FlushProgress::new();
    let before = serde_json::to_value(&session.lock().unwrap().state).unwrap();
    let mut snapshots = vec![];
    let held = db.lock().unwrap();
    assert_eq!(flush_state_progress(&session, &db, &mut progress, |s| {
        snapshots.push(crate::session_persistence::PersistedEngineState::from_session(s, None)); "saved"
    }), "store=saved database=skipped_db_locked");
    drop(held);
    session.lock().unwrap().notify_inactivity_fired = true;
    assert_eq!(serde_json::to_value(&session.lock().unwrap().state).unwrap(), before);
    assert_eq!(flush_state_progress(&session, &db, &mut progress, |s| {
        snapshots.push(crate::session_persistence::PersistedEngineState::from_session(s, Some("reset-1".into()))); "saved"
    }), "store=saved database=saved");
    progress.cancel(&db).unwrap();
    session.lock().unwrap().notify_posture_balance_fired = true;
    assert_eq!(serde_json::to_value(&session.lock().unwrap().state).unwrap(), before);
    flush_state_progress(&session, &db, &mut progress, |s| {
        snapshots.push(crate::session_persistence::PersistedEngineState::from_session(s, Some("reset-2".into()))); "saved"
    });
    assert_eq!(snapshots.len(), 3);
    assert!(!snapshots[0].notify_inactivity_fired);
    assert!(snapshots[1].notify_inactivity_fired);
    assert!(snapshots[2].notify_posture_balance_fired);
    assert_eq!(snapshots[2].daily_reset_after.as_deref(), Some("reset-2"));
    let conn = db.lock().unwrap();
    assert_eq!(conn.as_ref().unwrap().query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get::<_, i64>(0)).unwrap(), 2);
}

#[test]
fn cancellation_is_retryable_and_never_removes_an_unrelated_or_changed_row() {
    use crate::app_lifecycle_runtime::{FlushProgress, flush_state_progress};
    let session = sitting(); let db = shutdown_database(); let mut progress = FlushProgress::new();
    flush_state_progress(&session, &db, &mut progress, |_| "saved");
    let held = db.lock().unwrap();
    assert!(progress.cancel(&db).is_err());
    assert!(!progress.complete());
    drop(held);
    session.lock().unwrap().state.sitting_seconds = 333;
    flush_state_progress(&session, &db, &mut progress, |s| { assert_eq!(s.state.sitting_seconds, 333); "saved" });
    {
        let conn = db.lock().unwrap();
        conn.as_ref().unwrap().execute("UPDATE sessions SET duration_seconds=777 WHERE started_at != 'prior-start'", []).unwrap();
    }
    assert!(progress.cancel(&db).is_err(), "identity changed; never delete the replacement");
    let conn = db.lock().unwrap();
    assert_eq!(conn.as_ref().unwrap().query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get::<_, i64>(0)).unwrap(), 2);
}
