//! Tests for `flush_current_session()` — verifies that shutdown persistence
//! captures the correct state label and duration for all DeskState variants.

use chrono::{Duration, Utc};

use crate::session_manager::SessionManager;
use crate::session_types::DeskState;

#[test]
fn flush_during_sitting() {
    let mut mgr = SessionManager::new();
    let now = Utc::now();
    mgr.state.state = DeskState::Sitting;
    mgr.state.sitting_started = Some(now - Duration::seconds(300));

    let result = mgr.flush_current_session();
    assert!(result.is_some(), "Should produce a CompletedSession");
    let session = result.unwrap();
    assert!(session.duration_secs >= 299 && session.duration_secs <= 301);
}

#[test]
fn flush_during_standing() {
    let mut mgr = SessionManager::new();
    let now = Utc::now();
    mgr.state.state = DeskState::Standing;
    mgr.state.standing_bout_started = Some(now - Duration::seconds(120));

    let result = mgr.flush_current_session();
    assert!(result.is_some(), "Should produce a CompletedSession");
    let session = result.unwrap();
    assert!(session.duration_secs >= 119 && session.duration_secs <= 121);
}

#[test]
fn flush_during_walking() {
    let mut mgr = SessionManager::new();
    let now = Utc::now();
    mgr.state.state = DeskState::Walking;
    mgr.state.standing_bout_started = Some(now - Duration::seconds(60));

    let result = mgr.flush_current_session();
    assert!(result.is_some(), "Walking uses standing_bout_started");
    let session = result.unwrap();
    assert!(session.duration_secs >= 59 && session.duration_secs <= 61);
}

#[test]
fn flush_during_away_returns_none() {
    let mgr = SessionManager::new();
    // Default state is Away with no timestamps
    assert_eq!(mgr.state.state, DeskState::Away);
    let result = mgr.flush_current_session();
    assert!(result.is_none(), "Away state should not produce a session");
}

fn instant(offset: i64) -> chrono::DateTime<Utc> {
    chrono::DateTime::parse_from_rfc3339("2026-10-09T10:00:00Z")
        .unwrap().with_timezone(&Utc) + Duration::seconds(offset)
}

fn confirm(m: &mut SessionManager, state: DeskState, offset: i64) -> crate::session::ReadingResult {
    let (mm, active) = match state {
        DeskState::Sitting => (800, true),
        DeskState::Standing => (1200, true),
        DeskState::Away => (1200, false),
        DeskState::Walking => unreachable!(),
    };
    let requested_start = instant(offset - crate::session_types::DEBOUNCE_COUNT as i64);
    let previous = m.history_last_reading.into_iter().chain(m.state.last_tick_ts).max();
    let sample_start = previous.map_or(requested_start, |last| {
        if (requested_start - last).num_seconds() > crate::session_types::SLEEP_GAP_THRESHOLD_SECS {
            instant(offset)
        } else {
            requested_start
        }
    });
    let mut result = crate::session::ReadingResult::default();
    for second in 0..=crate::session::DEBOUNCE_COUNT {
        result = m.on_reading_at(mm, active, sample_start + Duration::seconds(second as i64));
    }
    result
}

fn memory_db() -> std::sync::Arc<std::sync::Mutex<Option<rusqlite::Connection>>> {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    crate::db::init_schema(&conn).unwrap();
    std::sync::Arc::new(std::sync::Mutex::new(Some(conn)))
}

#[test]
fn e027_mixed_break_records_exact_states_and_whole_break_credit() {
    use crate::session::BreakCredit;
    let db = memory_db();
    let mut m = SessionManager::new();
    confirm(&mut m, DeskState::Sitting, 0);
    m.state.sitting_seconds = 2000;
    m.state.daily_score = 17.0;
    let mut rows = Vec::new();
    for (end, after) in [(200, DeskState::Standing), (220, DeskState::Away),
        (250, DeskState::Standing), (290, DeskState::Away), (340, DeskState::Sitting)] {
        let before = m.current_state();
        let result = confirm(&mut m, after, end);
        let span = result.completed_session.unwrap();
        crate::serial_periodic_reset::persist_completed_session(&db, &span, &before,
            result.break_credit.as_ref().map(|(credit, _)| credit));
        if end == 340 {
            assert_eq!(result.break_credit, Some((BreakCredit::Partial, 140)));
        } else {
            assert!(result.break_credit.is_none());
        }
        rows.push(span);
    }
    assert_eq!(rows.iter().map(|s| s.duration_secs).collect::<Vec<_>>(), [200, 20, 30, 40, 50]);
    assert_eq!(rows.iter().map(|s| s.duration_secs).sum::<i64>(), 340);
    for pair in rows.windows(2) { assert_eq!(pair[0].ended_at, pair[1].started_at); }
    assert_eq!(m.state.sitting_seconds, 1920); // 2000 + 200 - 140 * 2
    assert_eq!(m.state.sitting_seconds_total, 200);
    assert_eq!(m.state.standing_seconds, 60);
    assert_eq!(m.state.daily_score, 17.0);
    let guard = db.lock().unwrap();
    let mut stmt = guard.as_ref().unwrap().prepare(
        "SELECT state, duration_seconds, break_credit FROM sessions ORDER BY id").unwrap();
    let saved = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?,
        r.get::<_, Option<String>>(2)?))).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(saved.iter().map(|r| r.0.as_str()).collect::<Vec<_>>(),
        ["Sitting", "Standing", "Away", "Standing", "Away"]);
    assert_eq!(saved.iter().map(|r| r.1).collect::<Vec<_>>(), [200, 20, 30, 40, 50]);
    assert!(saved[..4].iter().all(|r| r.2.is_none()));
    assert_eq!(saved[4].2.as_deref(), Some("partial"));
}

#[test]
fn e027_observed_away_shutdown_uses_only_final_individual_span() {
    let mut m = SessionManager::new();
    confirm(&mut m, DeskState::Standing, 0);
    confirm(&mut m, DeskState::Away, 100);
    confirm(&mut m, DeskState::Away, 150);
    let span = m.flush_current_session_at(instant(150)).unwrap();
    assert_eq!(span.started_at, instant(100).to_rfc3339());
    assert_eq!(span.ended_at, instant(150).to_rfc3339());
    assert_eq!(span.duration_secs, 50);
    assert_eq!(m.state.break_started, Some(instant(0)));
    let db = memory_db();
    let session = std::sync::Mutex::new(m);
    let mut progress = crate::app_lifecycle_runtime::FlushProgress::new();
    assert_eq!(crate::app_lifecycle_runtime::flush_state_progress_at(&session, &db,
        &mut progress, instant(150), |_| "saved"), "store=saved database=saved");
    let guard = db.lock().unwrap();
    let row: (String, String, String, i64) = guard.as_ref().unwrap().query_row(
        "SELECT state, started_at, ended_at, duration_seconds FROM sessions ORDER BY id DESC LIMIT 1", [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))).unwrap();
    assert_eq!(row, ("Away".into(), span.started_at, span.ended_at, 50));
    let count: i64 = guard.as_ref().unwrap().query_row("SELECT COUNT(*) FROM sessions", [],
        |r| r.get(0)).unwrap();
    assert_eq!(count, 2); // Earlier Standing plus the shutdown Away tail.
}

#[test]
fn e027_sleep_closes_coverage_and_keeps_gap_unobserved() {
    let db = memory_db();
    let mut m = SessionManager::new();
    confirm(&mut m, DeskState::Standing, 0);
    confirm(&mut m, DeskState::Standing, 100);
    let result = confirm(&mut m, DeskState::Away, 800);
    let interrupted = std::mem::take(&mut m.interrupted_history);
    assert_eq!(interrupted.len(), 1);
    assert_eq!(interrupted[0].0, DeskState::Standing);
    assert_eq!(interrupted[0].1.duration_secs, 100);
    assert_eq!(interrupted[0].1.ended_at, instant(100).to_rfc3339());
    crate::serial_periodic_reset::persist_completed_session(&db, &interrupted[0].1,
        &interrupted[0].0, None);
    assert!(result.completed_session.is_none()); // no zero-duration resumed span
    let final_span = confirm(&mut m, DeskState::Sitting, 850).completed_session.unwrap();
    assert_eq!(final_span.started_at, instant(800).to_rfc3339());
    assert_eq!(final_span.duration_secs, 50);
    assert_eq!(m.state.last_break_secs, 150,
        "only the observed 100-second prefix and 50-second resumed tail count as break");
    assert!(m.interrupted_history.is_empty());

    crate::serial_periodic_reset::persist_pending_history(&mut m,
        db.lock().unwrap().as_ref().unwrap(), |_, _| {}).unwrap();
    let guard = db.lock().unwrap();
    let conn = guard.as_ref().unwrap();
    let mut stmt = conn.prepare(
        "SELECT state, started_at, ended_at, duration_seconds FROM sessions ORDER BY id"
    ).unwrap();
    let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?,
        row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, i64>(3)?)))
        .unwrap().collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(rows, vec![
        ("Standing".into(), instant(0).to_rfc3339(), instant(100).to_rfc3339(), 100),
        ("Away".into(), instant(800).to_rfc3339(), instant(850).to_rfc3339(), 50),
    ], "the unknown 700-second interval must not become a persisted row");
}

#[test]
fn e027_shutdown_after_missing_readings_stops_at_last_observation() {
    let mut m = SessionManager::new();
    confirm(&mut m, DeskState::Away, 0);
    confirm(&mut m, DeskState::Away, 100);
    let span = m.flush_current_session_at(instant(800)).unwrap();
    assert_eq!(span.duration_secs, 100);
    assert_eq!(span.ended_at, instant(100).to_rfc3339());
}

#[test]
fn e027_shutdown_after_fractional_gap_stops_at_last_observation() {
    let mut m = SessionManager::new();
    let started = instant(0);
    let last = instant(10);
    m.state.state = DeskState::Standing;
    m.state.standing_bout_started = Some(started);
    m.history_span_started = Some(started);
    m.history_last_reading = Some(last);

    let span = m.flush_current_session_at(last + Duration::milliseconds(300_500)).unwrap();
    assert_eq!(span.started_at, started.to_rfc3339());
    assert_eq!(span.ended_at, last.to_rfc3339(),
        "a 300.5-second shutdown gap must stop at the last observation");
    assert_eq!(span.duration_secs, 10,
        "time after the final observation remains unknown, not session coverage");
}

#[test]
fn e027_restart_does_not_replay_already_flushed_history() {
    let mut m = SessionManager::new();
    confirm(&mut m, DeskState::Standing, 0);
    confirm(&mut m, DeskState::Away, 100);
    confirm(&mut m, DeskState::Away, 150);
    let old = m.flush_current_session_at(instant(150)).unwrap();
    let snapshot = crate::session_persistence::PersistedEngineState::from_session(&m, None);
    let mut restored = SessionManager::new();
    restored.load_persisted_state(&snapshot);
    assert!(restored.flush_current_session_at(instant(180)).is_none());
    confirm(&mut restored, DeskState::Away, 200);
    let next = confirm(&mut restored, DeskState::Sitting, 250).completed_session.unwrap();
    assert_eq!(next.started_at, instant(200).to_rfc3339());
    assert_eq!(next.duration_secs, 50);
    assert!(old.ended_at < next.started_at);
}

#[test]
fn e027_day_rollover_preserves_history_anchor_separate_from_raw_counters() {
    use chrono_tz::Europe::Warsaw;
    let mut m = SessionManager::new();
    confirm(&mut m, DeskState::Standing, 43100);
    confirm(&mut m, DeskState::Standing, 43150);
    let boundary = instant(43200);
    let previous_day = instant(43100).with_timezone(&Warsaw).date_naive();
    let today = boundary.with_timezone(&Warsaw).date_naive();
    confirm(&mut m, DeskState::Standing, 43200);
    m.last_reset_date = previous_day;
    m.last_reset_check = boundary - Duration::seconds(120);
    assert!(m.check_daily_reset_at_boundary(boundary, today, boundary));
    assert_eq!(m.state.standing_seconds, 0, "the first local day's raw count resets");
    assert_eq!(m.state.daily_score, 0.0);
    let db = memory_db();
    crate::serial_periodic_reset::persist_pending_history(&mut m,
        db.lock().unwrap().as_ref().unwrap(), |_, _| {}).unwrap();
    confirm(&mut m, DeskState::Away, 43250);
    assert_eq!(m.state.standing_seconds, 50,
        "the next local day counts only the post-midnight observed Standing tail");
    crate::serial_periodic_reset::persist_pending_history(&mut m,
        db.lock().unwrap().as_ref().unwrap(), |_, _| {}).unwrap();

    let guard = db.lock().unwrap();
    let conn = guard.as_ref().unwrap();
    let mut stmt = conn.prepare(
        "SELECT state, started_at, ended_at, duration_seconds FROM sessions ORDER BY id"
    ).unwrap();
    let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?,
        row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, i64>(3)?)))
        .unwrap().collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(rows.len(), 2, "neither local-day portion may disappear or duplicate");
    assert_eq!(rows[0], ("Standing".into(), instant(43100).to_rfc3339(), boundary.to_rfc3339(), 100));
    assert_eq!(rows[1], ("Standing".into(), boundary.to_rfc3339(), instant(43250).to_rfc3339(), 50));
    assert_eq!(rows.iter().map(|row| row.3).sum::<i64>(), 150);
}

#[test]
fn e027_delayed_reset_after_postmidnight_transition_splits_rows_and_keeps_day_raw_totals() {
    use chrono_tz::Europe::Warsaw;
    let mut m = SessionManager::new();
    confirm(&mut m, DeskState::Sitting, 43_100);
    confirm(&mut m, DeskState::Sitting, 43_150);

    let boundary = instant(43_200);
    let previous_day = boundary.with_timezone(&Warsaw).date_naive() - Duration::days(1);
    let today = boundary.with_timezone(&Warsaw).date_naive();
    m.last_reset_date = previous_day;
    m.last_reset_check = boundary - Duration::seconds(120);
    assert!(m.check_daily_reset_at_observation(instant(43_215), today, boundary));
    for second in 0..=crate::session_types::DEBOUNCE_COUNT {
        m.on_reading_at(1200, true, instant(43_215 + second as i64));
    }

    assert_eq!(m.state.sitting_seconds_total, 20,
        "only the confirmed post-midnight Sitting portion belongs to today");
    assert_eq!(m.state.standing_seconds, 0,
        "the transition sample starts Standing; it does not backdate that posture");
    assert_eq!(m.history_span_started, Some(instant(43_220)),
        "the reset must not rewind the current Standing anchor to midnight");

    let db = memory_db();
    crate::serial_periodic_reset::persist_pending_history(&mut m,
        db.lock().unwrap().as_ref().unwrap(), |_, _| {}).unwrap();
    let guard = db.lock().unwrap();
    let conn = guard.as_ref().unwrap();
    let mut stmt = conn.prepare(
        "SELECT state, started_at, ended_at, duration_seconds FROM sessions ORDER BY id"
    ).unwrap();
    let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?,
        row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, i64>(3)?)))
        .unwrap().collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(rows, vec![
        ("Sitting".into(), instant(43_100).to_rfc3339(), boundary.to_rfc3339(), 100),
        ("Sitting".into(), boundary.to_rfc3339(), instant(43_220).to_rfc3339(), 20),
    ], "history crossing the local midnight is split without inventing posture time");
}

#[test]
fn e027_long_observed_span_keeps_consistent_endpoints_without_cap() {
    let mut m = SessionManager::new();
    confirm(&mut m, DeskState::Away, 0);
    for seconds in (100..=12000).step_by(100) { confirm(&mut m, DeskState::Away, seconds); }
    confirm(&mut m, DeskState::Away, 12050);
    let span = m.flush_current_session_at(instant(12050)).unwrap();
    assert_eq!(span.duration_secs, 12050);
    let start = chrono::DateTime::parse_from_rfc3339(&span.started_at).unwrap();
    let end = chrono::DateTime::parse_from_rfc3339(&span.ended_at).unwrap();
    assert_eq!((end - start).num_seconds(), span.duration_secs);
}

#[test]
fn e027_sleep_across_daily_reset_still_closes_only_observed_coverage() {
    use chrono_tz::Europe::Warsaw;
    let mut m = SessionManager::new();
    let boundary = instant(43200);
    confirm(&mut m, DeskState::Standing, 43100);
    confirm(&mut m, DeskState::Standing, 43150);
    let previous_day = instant(43100).with_timezone(&Warsaw).date_naive();
    let today = boundary.with_timezone(&Warsaw).date_naive();
    m.last_reset_date = previous_day;
    m.last_reset_check = boundary - Duration::seconds(120);
    assert!(m.check_daily_reset_at_boundary(boundary + Duration::seconds(700), today, boundary));
    assert!(m.state.last_tick_ts.is_none());
    let db = memory_db();
    let snapshot = crate::session_persistence::PersistedEngineState::from_session(&m, None);
    crate::serial_periodic_reset::persist_pending_history(&mut m,
        db.lock().unwrap().as_ref().unwrap(), |_, _| {}).unwrap();
    m.load_persisted_state(&snapshot);
    crate::serial_periodic_reset::persist_pending_history(&mut m,
        db.lock().unwrap().as_ref().unwrap(), |_, _| {}).unwrap();

    let resume = boundary + Duration::seconds(800);
    for second in 0..=crate::session_types::DEBOUNCE_COUNT {
        m.on_reading_at(1200, true, resume + Duration::seconds(second as i64));
    }
    let tail = confirm(&mut m, DeskState::Away, 44_050).completed_session.unwrap();
    assert_eq!(tail.started_at, resume.to_rfc3339());
    assert_eq!(tail.ended_at, (resume + Duration::seconds(50)).to_rfc3339());
    assert_eq!(tail.duration_secs, 50);
    assert_eq!(m.state.standing_seconds, 50,
        "the combined gap/reset/replay path retains only the resumed day's known Standing tail");
    crate::serial_periodic_reset::persist_pending_history(&mut m,
        db.lock().unwrap().as_ref().unwrap(), |_, _| {}).unwrap();

    let guard = db.lock().unwrap();
    let conn = guard.as_ref().unwrap();
    let mut stmt = conn.prepare(
        "SELECT state, started_at, ended_at, duration_seconds FROM sessions ORDER BY id"
    ).unwrap();
    let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?,
        row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, i64>(3)?)))
        .unwrap().collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(rows, vec![
        ("Standing".into(), instant(43100).to_rfc3339(), instant(43150).to_rfc3339(), 50),
        ("Standing".into(), resume.to_rfc3339(), (resume + Duration::seconds(50)).to_rfc3339(), 50),
    ], "the known pre-reset prefix is durable once, the gap absent, and resumed tail distinct");
}

#[test]
fn e027_backwards_clock_cannot_move_state_or_history_anchor() {
    let mut m = SessionManager::new();
    confirm(&mut m, DeskState::Sitting, 0);
    let first = confirm(&mut m, DeskState::Standing, 200).completed_session.unwrap();
    assert!(confirm(&mut m, DeskState::Away, 100).state_change.is_none());
    assert_eq!(m.current_state(), DeskState::Standing);
    assert_eq!(m.history_last_reading, Some(instant(200)));
    assert_eq!(m.history_span_started, Some(instant(200)));
    let second = confirm(&mut m, DeskState::Away, 250).completed_session.unwrap();
    assert_eq!(first.ended_at, second.started_at);
    assert_eq!(first.duration_secs + second.duration_secs, 250);
    assert_eq!(second.duration_secs, 50);
    assert_eq!(m.state.standing_seconds, 50);
}
