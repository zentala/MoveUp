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
    fn scenario_e_restart_should_not_seed_session_timer() {
        // SPEC: After restart, current_session_secs = 0 (fresh session)
        // sitting_seconds is the daily accumulator and WILL be seeded from DB.
        let mut m = SessionManager::new();
        m.load_today_totals(1200, 600);

        // sitting_seconds holds the daily total (expected: 1200)
        assert_eq!(m.state.sitting_seconds, 1200);
        // current_session_secs must be 0 (no active session after restart)
        assert_eq!(
            m.state.current_session_secs, 0,
            "Scenario E: current_session_secs should be 0 after restart"
        );
    }

    // ── Scenario F: Sitting timer shows total today (FIXED) ─────────────

    #[test]
    fn scenario_f_timer_shows_current_session_not_total() {
        // SPEC: After full reset + restart, current_session_secs = 0
        let mut m = SessionManager::new();
        m.load_today_totals(35 * 60, 10 * 60);

        // Daily accumulator is seeded
        assert_eq!(m.state.sitting_seconds, 35 * 60);
        // But session timer starts at 0
        assert_eq!(
            m.state.current_session_secs, 0,
            "Scenario F: current_session_secs should be 0, not total today"
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
                break_credit: None,
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
