//! session_tests_serde.rs — Contract tests: Rust↔Frontend serialization agreement.
//!
//! These tests verify that serde output matches what the TypeScript frontend expects.
//! If these fail, the frontend state comparisons (e.g. `state === "Standing"`) will
//! silently break — the UI will render but conditional logic won't fire.

#[test]
fn e027_pending_history_survives_failure_json_restore_and_commit_replay() {
    use crate::session_types::DEBOUNCE_COUNT;
    use crate::session_manager::SessionManager;
    use crate::session_persistence::PersistedEngineState;
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    crate::db::init_schema(&conn).unwrap();
    conn.execute_batch("CREATE TRIGGER reject_history BEFORE INSERT ON sessions
        BEGIN SELECT RAISE(FAIL, 'synthetic failure'); END;").unwrap();
    let mut m = SessionManager::new();
    m.state.sitting_seconds = 500;
    let at = chrono::DateTime::parse_from_rfc3339("2026-10-09T10:00:00Z").unwrap()
        .with_timezone(&chrono::Utc);
    for (seconds, mm, active) in [(0, 1200, true), (60, 1200, false), (120, 800, true)] {
        for offset in 0..=DEBOUNCE_COUNT {
            m.on_reading_at(mm, active, at + chrono::Duration::seconds(seconds - DEBOUNCE_COUNT as i64 + offset as i64));
        }
    }
    assert!(crate::serial_periodic_reset::persist_pending_history(&mut m, &conn, |_, _| {}).is_err());
    assert_eq!(m.pending_history.len(), 2);
    let json = serde_json::to_value(PersistedEngineState::from_session(&m, None)).unwrap();
    let snapshot = PersistedEngineState::from_stored_value(json).unwrap();
    let mut restored = SessionManager::new();
    restored.load_persisted_state(&snapshot);
    assert_eq!(restored.pending_history.len(), 2);
    assert!(restored.history_span_started.is_none());
    conn.execute_batch("DROP TRIGGER reject_history;").unwrap();
    crate::serial_periodic_reset::persist_pending_history(&mut restored, &conn, |_, _| {}).unwrap();
    assert!(restored.pending_history.is_empty());
    // Simulate a crash after SQLite commit but before the store ack save.
    restored.load_persisted_state(&snapshot);
    crate::serial_periodic_reset::persist_pending_history(&mut restored, &conn, |_, _| {}).unwrap();
    let (count, sum): (i64, i64) = conn.query_row(
        "SELECT COUNT(*), SUM(duration_seconds) FROM sessions", [],
        |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
    assert_eq!((count, sum), (2, 120));
    assert!(restored.pending_history.is_empty());
    let credit: String = conn.query_row("SELECT break_credit FROM sessions WHERE state='Away'",
        [], |r| r.get(0)).unwrap();
    assert_eq!(credit, "partial");
    let mut stale = snapshot.clone();
    stale.date_local = "2020-01-01".into();
    let mut next_day = SessionManager::new();
    next_day.load_today_totals(&crate::db_sessions::TodayTotals::from_secs(1000, 500));
    next_day.load_persisted_state(&stale);
    assert_eq!(next_day.state.sitting_seconds, 1000); // Stale counters cannot overwrite today.
    assert_eq!(next_day.state.standing_seconds, 500);
    assert_eq!(next_day.pending_history.len(), 2); // Retry envelope crosses midnight.
    crate::serial_periodic_reset::persist_pending_history(&mut next_day, &conn, |_, _| {}).unwrap();
    assert!(next_day.pending_history.is_empty());
}

#[cfg(test)]
mod serde_contract_tests {
    use crate::session_types::*;

    // ─── DeskState serialization ────────────────────────────────────────

    #[test]
    fn desk_state_serializes_to_pascal_case() {
        // Frontend compares: state === "Sitting", state === "Standing", etc.
        // If serde renames to snake_case, all frontend conditionals break silently.
        assert_eq!(
            serde_json::to_string(&DeskState::Sitting).unwrap(),
            r#""Sitting""#
        );
        assert_eq!(
            serde_json::to_string(&DeskState::Standing).unwrap(),
            r#""Standing""#
        );
        assert_eq!(
            serde_json::to_string(&DeskState::Walking).unwrap(),
            r#""Walking""#
        );
        assert_eq!(
            serde_json::to_string(&DeskState::Away).unwrap(),
            r#""Away""#
        );
    }

    #[test]
    fn desk_state_deserializes_from_pascal_case() {
        assert_eq!(
            serde_json::from_str::<DeskState>(r#""Sitting""#).unwrap(),
            DeskState::Sitting
        );
        assert_eq!(
            serde_json::from_str::<DeskState>(r#""Standing""#).unwrap(),
            DeskState::Standing
        );
    }

    // ─── SessionStateDto fields ─────────────────────────────────────────

    #[test]
    fn session_state_dto_state_field_is_pascal_case() {
        // Simulates what get_session_state IPC returns to the frontend.
        let dto = SessionStateDto {
            state: DeskState::Standing,
            sitting_seconds: 0,
            standing_seconds: 300,
            break_seconds: 300,
            session_limit_secs: 2700,
            stand_limit_secs: 0,
            desk_height_cm: 110.0,
            position_changes: 1,
            limit_used_secs: 0,
            daily_score: 0.0,
            standing_session_secs: 300,
            secs_since_last_break: 0,
            continuous_computer_secs: 0,
            longest_computer_session_secs: 0,
            sitting_seconds_total: 0,
            idle_secs: 0,
            away_bout_secs: 0,
            max_continuous_computer_secs: 3600,
        };
        let json = serde_json::to_value(&dto).unwrap();
        assert_eq!(json["state"], "Standing");
    }

    #[test]
    fn state_changed_payload_state_field_is_pascal_case() {
        let payload = StateChangedPayload {
            state: DeskState::Sitting,
            standing_seconds: 0,
            break_seconds: 0,
            desk_height_cm: 72.0,
            position_changes: 2,
            last_break_secs: 300,
            last_sitting_secs: 600,
            break_credit: BreakCredit::Partial,
            limit_used_secs: 600,
        };
        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["state"], "Sitting");
    }

    // ─── Debug format matches serde (DB uses Debug) ─────────────────────

    #[test]
    fn desk_state_debug_matches_serde() {
        // DB saves via format!("{:?}", state) — must match serde output
        // so DB queries and frontend agree on state strings.
        let states = [
            DeskState::Sitting,
            DeskState::Standing,
            DeskState::Walking,
            DeskState::Away,
        ];
        for state in &states {
            let debug_str = format!("{:?}", state);
            let serde_str = serde_json::to_string(state)
                .unwrap()
                .trim_matches('"')
                .to_string();
            assert_eq!(
                debug_str, serde_str,
                "Debug and serde must agree for {:?}: debug={}, serde={}",
                state, debug_str, serde_str
            );
        }
    }
}
