//! session_tests_sleep.rs — Sleep/suspend gap detection and break credit tests.

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    fn advance_ticks(m: &mut SessionManager, mm: i32, active: bool, count: usize) {
        for _ in 0..count {
            let _ = m.on_reading(mm, active);
        }
    }

    /// Transitions manager to Sitting state via debounced readings.
    fn transition_to_sitting(m: &mut SessionManager) {
        advance_ticks(m, 800, true, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Sitting);
    }

    /// Transitions manager to Standing state via debounced readings.
    fn transition_to_standing(m: &mut SessionManager) {
        advance_ticks(m, 1200, true, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Standing);
    }

    // ── Inflation-prevention (original tests) ──────────────────────────────

    /// Regression: computer sleep inflates sitting_seconds.
    #[test]
    fn sleep_does_not_inflate_sitting_seconds() {
        let mut m = SessionManager::new();
        let now = Utc::now();

        transition_to_sitting(&mut m);

        m.state.sitting_started = Some(now - chrono::Duration::hours(8));
        m.state.sitting_seconds = 600;
        m.state.sitting_seconds_total = 600;
        m.state.last_tick_ts = Some(now - chrono::Duration::hours(8));

        advance_ticks(&mut m, 800, false, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Away);

        assert!(
            m.state.sitting_seconds < 700,
            "sitting_seconds should be capped, got {} (sleep inflation!)",
            m.state.sitting_seconds
        );
    }

    /// Same test but for standing: sleep shouldn't inflate standing_seconds.
    #[test]
    fn sleep_does_not_inflate_standing_seconds() {
        let mut m = SessionManager::new();
        let now = Utc::now();

        transition_to_sitting(&mut m);
        m.state.sitting_started = Some(now);
        transition_to_standing(&mut m);
        assert_eq!(m.state.state, DeskState::Standing);

        m.state.standing_bout_started = Some(now - chrono::Duration::hours(8));
        m.state.standing_seconds = 300;
        m.state.last_tick_ts = Some(now - chrono::Duration::hours(8));

        advance_ticks(&mut m, 1200, false, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Away);

        assert!(
            m.state.standing_seconds < 400,
            "standing_seconds should be capped, got {} (sleep inflation!)",
            m.state.standing_seconds
        );
    }

    // ── Break credit applied during sleep gap ──────────────────────────────

    /// A 2-hour sleep gap while sitting should fully reset sitting_seconds
    /// (2h × 2.0 multiplier = 4h credit > any reasonable sitting duration).
    #[test]
    fn sleep_gap_full_credit_resets_sitting() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);

        m.state.sitting_seconds = 2400; // 40 min sitting
        m.state.sitting_seconds_total = 2400;
        // Simulate 2-hour gap between last tick and now.
        m.state.last_tick_ts = Some(Utc::now() - chrono::Duration::hours(2));

        // The next reading triggers sleep gap detection → apply_break_credit(7200).
        let _ = m.on_reading(800, true);

        assert_eq!(
            m.state.sitting_seconds, 0,
            "2h gap × 2.0 multiplier should fully reset sitting_seconds (got {})",
            m.state.sitting_seconds
        );
        assert_eq!(
            m.state.last_break_credit,
            BreakCredit::Full,
            "break credit should be Full after oversized sleep gap"
        );
    }

    /// A 10-minute sleep gap while sitting should give partial credit:
    /// 2400 - (600 × 2.0) = 1200 remaining.
    #[test]
    fn sleep_gap_partial_credit_reduces_sitting() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);

        m.state.sitting_seconds = 2400; // 40 min sitting
        m.state.sitting_seconds_total = 2400;
        // Simulate 10-minute gap.
        m.state.last_tick_ts = Some(Utc::now() - chrono::Duration::minutes(10));

        let _ = m.on_reading(800, true);

        // 600s × 2.0 = 1200s credit → 2400 - 1200 = 1200 remaining.
        assert_eq!(
            m.state.sitting_seconds, 1200,
            "10-min gap × 2.0 should reduce sitting to 1200s (got {})",
            m.state.sitting_seconds
        );
        assert_eq!(
            m.state.last_break_credit,
            BreakCredit::Partial,
            "break credit should be Partial for 10-min gap"
        );
    }

    /// Break credit multiplier of 1.0 should halve the reduction rate.
    #[test]
    fn sleep_gap_respects_custom_multiplier() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);

        m.state.sitting_seconds = 2400; // 40 min sitting
        m.state.sitting_seconds_total = 2400;
        m.state.break_credit_multiplier = 1.0; // slower reset
        // Simulate 10-minute gap.
        m.state.last_tick_ts = Some(Utc::now() - chrono::Duration::minutes(10));

        let _ = m.on_reading(800, true);

        // 600s × 1.0 = 600s credit → 2400 - 600 = 1800 remaining.
        assert_eq!(
            m.state.sitting_seconds, 1800,
            "10-min gap × 1.0 should reduce sitting to 1800s (got {})",
            m.state.sitting_seconds
        );
    }

    /// Sleep gap while in Standing state should not crash and should not corrupt
    /// sitting_seconds (which is already 0 in a fresh standing session).
    #[test]
    fn sleep_gap_while_standing_is_harmless() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_seconds = 0; // just transitioned from credit
        transition_to_standing(&mut m);

        let bout_start = Utc::now() - chrono::Duration::minutes(5);
        m.state.standing_bout_started = Some(bout_start);
        // Simulate 30-minute sleep gap while standing.
        m.state.last_tick_ts = Some(Utc::now() - chrono::Duration::minutes(30));

        // Should not panic; apply_break_credit(0) is called but credit is harmless.
        let _ = m.on_reading(1200, true);

        assert_eq!(
            m.state.sitting_seconds, 0,
            "sitting_seconds must stay 0 after sleep gap in Standing state"
        );
        // standing_bout_started must have been rewound to prevent standing inflation.
        let rewound = m.state.standing_bout_started.expect("should still be set");
        let age_secs = (Utc::now() - rewound).num_seconds();
        assert!(
            age_secs < 5,
            "standing_bout_started should be rewound to ~now, but is {}s old",
            age_secs
        );
    }

    /// Sleep gap must NOT reduce sitting_seconds_total — that counter tracks raw
    /// historical accumulation and must never be reduced by break credit.
    #[test]
    fn sleep_gap_preserves_sitting_seconds_total() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);

        m.state.sitting_seconds = 2400;
        m.state.sitting_seconds_total = 2400;
        // 2-hour sleep gap → full credit, sitting_seconds → 0.
        m.state.last_tick_ts = Some(Utc::now() - chrono::Duration::hours(2));

        let _ = m.on_reading(800, true);

        assert_eq!(
            m.state.sitting_seconds, 0,
            "sitting_seconds should be reset (got {})",
            m.state.sitting_seconds
        );
        assert_eq!(
            m.state.sitting_seconds_total, 2400,
            "sitting_seconds_total must not be affected by break credit (got {})",
            m.state.sitting_seconds_total
        );
    }
}
