//! session_tests_floating.rs — Tests for floating window spec scenarios (T029).
//! Tests that expose bugs use #[ignore] with a comment explaining the issue.

#[cfg(test)]
mod floating_window_tests {
    use chrono::Utc;

    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    /// Helper: transition manager to Sitting by feeding DEBOUNCE_COUNT low readings.
    fn transition_to_sitting(m: &mut SessionManager) {
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(800, true);
        }
        assert_eq!(m.state.state, DeskState::Sitting);
    }

    /// Helper: transition manager to Standing by feeding DEBOUNCE_COUNT high readings.
    fn transition_to_standing(m: &mut SessionManager) {
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(1200, true);
        }
        assert_eq!(m.state.state, DeskState::Standing);
    }

    // ── Scenario A: Fresh start ────────────────────────────────────────────

    #[test]
    fn scenario_a_fresh_start_sitting_seconds_is_zero() {
        // SPEC: On first Sitting transition with no prior history, sitting_seconds = 0
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        // sitting_seconds should be 0 at the moment of transition (no prior sitting)
        assert_eq!(
            m.state.sitting_seconds, 0,
            "Scenario A: fresh start should have sitting_seconds = 0"
        );
    }

    // ── Scenario B: Short break (no credit, < 5 min) ──────────────────────

    #[test]
    fn scenario_b_short_break_no_credit() {
        // SPEC: After standing < 5 min, sitting_seconds = pre-break value
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);

        // Simulate 30 min of sitting by setting committed seconds
        m.state.sitting_seconds = 1800;
        m.state.sitting_started = Some(Utc::now());

        // Stand for 3 min (< 5 min threshold)
        transition_to_standing(&mut m);
        // Manually set break_started to 3 min ago to simulate time passing
        m.state.break_started =
            Some(Utc::now() - chrono::Duration::seconds(3 * 60));

        // Sit back down
        transition_to_sitting(&mut m);

        // Break credit: < 5 min = None, session continues
        // sitting_seconds should be >= 1800 (the pre-break value, plus small elapsed)
        assert!(
            m.state.sitting_seconds >= 1800,
            "Scenario B: after short break, sitting_seconds ({}) should be >= 1800",
            m.state.sitting_seconds
        );
        assert_eq!(
            m.state.last_break_credit,
            BreakCredit::None,
            "Scenario B: short break should yield BreakCredit::None"
        );
    }

    // ── Scenario C: Medium break (partial credit, 5-9 min) ────────────────

    #[test]
    fn scenario_c_partial_credit_7min_break() {
        // SPEC: sitting_seconds after 7-min break = max(0, pre_break - 1200)
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);

        m.state.sitting_seconds = 1800;
        m.state.sitting_started = Some(Utc::now());

        transition_to_standing(&mut m);
        m.state.break_started =
            Some(Utc::now() - chrono::Duration::seconds(7 * 60));

        transition_to_sitting(&mut m);

        // 1800 - 1200 = 600 (but there's small elapsed from the transition)
        // The key assertion: sitting_seconds should be close to 600
        // We allow small margin for elapsed time during transition
        assert!(
            m.state.sitting_seconds < 700,
            "Scenario C: after 7-min break, sitting_seconds ({}) should be ~600",
            m.state.sitting_seconds
        );
        assert_eq!(
            m.state.last_break_credit,
            BreakCredit::Partial,
            "Scenario C: 7-min break should yield BreakCredit::Partial"
        );
    }

    // ── Scenario D: Full break (>= 10 min standing) ───────────────────────

    #[test]
    fn scenario_d_full_reset_after_12min_break() {
        // SPEC: sitting_seconds = 0 after standing >= 10 min
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);

        m.state.sitting_seconds = 2100;
        m.state.sitting_started = Some(Utc::now());

        transition_to_standing(&mut m);
        m.state.break_started =
            Some(Utc::now() - chrono::Duration::seconds(12 * 60));

        transition_to_sitting(&mut m);

        assert_eq!(
            m.state.sitting_seconds, 0,
            "Scenario D: after 12-min break, sitting_seconds should be 0"
        );
        assert_eq!(
            m.state.last_break_credit,
            BreakCredit::Full,
            "Scenario D: 12-min break should yield BreakCredit::Full"
        );
    }

    // ── Scenario E: App restart mid-session ────────────────────────────────

    #[test]
    #[ignore] // BUG: load_today_totals() seeds sitting_seconds from DB total,
              // which means after restart the session timer shows total-today
              // instead of 0 for a fresh session. See T029 Scenario F.
    fn scenario_e_restart_should_not_seed_session_timer() {
        // SPEC (recommended): After restart, sitting_seconds = 0
        let mut m = SessionManager::new();
        // Simulate loading DB totals (e.g., user sat 20 min before restart)
        m.load_today_totals(1200, 600);

        // After load, sitting_seconds should still be 0 for the session timer
        // EXPECTED: 0 (fresh session after restart)
        // ACTUAL: 1200 (DB total seeded into sitting_seconds)
        assert_eq!(
            m.state.sitting_seconds, 0,
            "Scenario E: after restart, sitting_seconds should be 0 for session timer"
        );
    }

    // ── Scenario F: Sitting timer shows total today (KNOWN BUG) ────────────

    #[test]
    #[ignore] // BUG: After sit(30m) -> stand(10m, full reset) -> sit(5m),
              // sitting_seconds should be ~5m but load_today_totals seeds from DB.
              // The issue is that sitting_seconds conflates current session with
              // total-today after restart. Fix in T030.
    fn scenario_f_timer_shows_current_session_not_total() {
        // SPEC: After full reset + 5 min sitting, timer shows ~5:00
        let mut m = SessionManager::new();

        // Simulate: user sat 30m, stood 10m (full reset), then sat 5m,
        // then app restarted and DB says total sitting = 35 min today.
        m.load_today_totals(35 * 60, 10 * 60);

        // Now user starts a new sitting session — timer should show 0:00
        // EXPECTED: sitting_seconds = 0 (current session)
        // ACTUAL: sitting_seconds = 2100 (total today from DB)
        assert_eq!(
            m.state.sitting_seconds, 0,
            "Scenario F: session timer should show current session, not total today"
        );
    }

    // ── Scenario G: Last standing duration in payload ──────────────────────

    #[test]
    fn scenario_g_payload_includes_last_break_secs() {
        // SPEC: StateChangedPayload includes last_break_secs after transition
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_started = Some(Utc::now());

        transition_to_standing(&mut m);
        m.state.break_started =
            Some(Utc::now() - chrono::Duration::seconds(12 * 60));

        // Transition back to sitting — payload should have last_break_secs
        let result = {
            let mut r = ReadingResult {
                state_change: None,
                completed_session: None,
            };
            for _ in 0..DEBOUNCE_COUNT {
                r = m.on_reading(800, true);
            }
            r
        };

        let payload = result
            .state_change
            .expect("Scenario G: should have state_change payload");
        assert!(
            payload.last_break_secs >= 12 * 60,
            "Scenario G: last_break_secs ({}) should be >= 720",
            payload.last_break_secs
        );
        assert_eq!(
            payload.break_credit,
            BreakCredit::Full,
            "Scenario G: 12-min break should report Full credit in payload"
        );
    }

    // ── Scenario H: Position changes count ─────────────────────────────────

    #[test]
    fn scenario_h_position_changes_only_sit_stand() {
        // SPEC: Only Sitting<->Standing transitions increment position_changes
        let mut m = SessionManager::new();
        assert_eq!(m.state.position_changes, 0, "starts at 0");

        // Sit -> no change
        transition_to_sitting(&mut m);
        assert_eq!(m.state.position_changes, 0, "initial sit = 0 changes");

        // Sit -> Stand = +1
        transition_to_standing(&mut m);
        assert_eq!(m.state.position_changes, 1, "sit->stand = 1");

        // Stand -> Sit = +1
        m.state.break_started =
            Some(Utc::now() - chrono::Duration::seconds(600));
        transition_to_sitting(&mut m);
        assert_eq!(m.state.position_changes, 2, "stand->sit = 2");

        // Sit -> Walk (high reading, inactive) = NOT a position change
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(1200, false);
        }
        assert_eq!(m.state.state, DeskState::Walking);
        assert_eq!(m.state.position_changes, 2, "sit->walk = still 2");
    }
}
