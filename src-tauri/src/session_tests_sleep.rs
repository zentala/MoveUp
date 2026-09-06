//! session_tests_sleep.rs — Sleep/suspend gap detection and break credit tests.
//!
//! Readings are injected at explicit instants (E020-T01), so the gap the engine
//! sees is exactly the gap the test intends. The old form backdated
//! `last_tick_ts` against one `Utc::now()` and then let `on_reading` read a
//! second, later one, which is why some assertions here were inequalities.

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Duration, TimeZone, Utc};

    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    /// A fixed instant, so nothing here depends on when the suite runs.
    fn base() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 1, 9, 0, 0).unwrap()
    }

    /// Feeds `count` readings one second apart from `at`; returns the instant
    /// after the last one.
    fn advance_ticks(
        m: &mut SessionManager,
        mm: i32,
        active: bool,
        at: DateTime<Utc>,
        count: i64,
    ) -> DateTime<Utc> {
        for i in 0..count {
            let _ = m.on_reading_at(mm, active, at + Duration::seconds(i));
        }
        at + Duration::seconds(count)
    }

    /// Transitions manager to Sitting state via debounced readings.
    fn transition_to_sitting(m: &mut SessionManager, at: DateTime<Utc>) -> DateTime<Utc> {
        let next = advance_ticks(m, 800, true, at, DEBOUNCE_COUNT as i64);
        assert_eq!(m.state.state, DeskState::Sitting);
        next
    }

    /// Transitions manager to Standing state via debounced readings.
    fn transition_to_standing(m: &mut SessionManager, at: DateTime<Utc>) -> DateTime<Utc> {
        let next = advance_ticks(m, 1200, true, at, DEBOUNCE_COUNT as i64);
        assert_eq!(m.state.state, DeskState::Standing);
        next
    }

    // ── Inflation-prevention (original tests) ──────────────────────────────

    /// Regression: computer sleep inflates sitting_seconds.
    #[test]
    fn sleep_does_not_inflate_sitting_seconds() {
        let mut m = SessionManager::new();
        let now = base();

        transition_to_sitting(&mut m, now - Duration::hours(9));

        m.state.sitting_started = Some(now - Duration::hours(8));
        m.state.sitting_seconds = 600;
        m.state.sitting_seconds_total = 600;
        m.state.last_tick_ts = Some(now - Duration::hours(8));

        advance_ticks(&mut m, 800, false, now, DEBOUNCE_COUNT as i64);
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
        let now = base();

        let t = transition_to_sitting(&mut m, now - Duration::hours(9));
        m.state.sitting_started = Some(now);
        transition_to_standing(&mut m, t);
        assert_eq!(m.state.state, DeskState::Standing);

        m.state.standing_bout_started = Some(now - Duration::hours(8));
        m.state.standing_seconds = 300;
        m.state.last_tick_ts = Some(now - Duration::hours(8));

        advance_ticks(&mut m, 1200, false, now, DEBOUNCE_COUNT as i64);
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
        let now = base();
        transition_to_sitting(&mut m, now - Duration::hours(3));

        m.state.sitting_seconds = 2400; // 40 min sitting
        m.state.sitting_seconds_total = 2400;
        // Simulate 2-hour gap between last tick and now.
        m.state.last_tick_ts = Some(now - Duration::hours(2));

        // The next reading triggers sleep gap detection → apply_break_credit(7200).
        let _ = m.on_reading_at(800, true, now);

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
        let now = base();
        transition_to_sitting(&mut m, now - Duration::hours(1));

        m.state.sitting_seconds = 2400; // 40 min sitting
        m.state.sitting_seconds_total = 2400;
        // Simulate 10-minute gap.
        m.state.last_tick_ts = Some(now - Duration::minutes(10));

        let _ = m.on_reading_at(800, true, now);

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
        let now = base();
        transition_to_sitting(&mut m, now - Duration::hours(1));

        m.state.sitting_seconds = 2400; // 40 min sitting
        m.state.sitting_seconds_total = 2400;
        m.state.break_credit_multiplier = 1.0; // slower reset
        // Simulate 10-minute gap.
        m.state.last_tick_ts = Some(now - Duration::minutes(10));

        let _ = m.on_reading_at(800, true, now);

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
        let now = base();
        let t = transition_to_sitting(&mut m, now - Duration::hours(1));
        m.state.sitting_seconds = 0; // just transitioned from credit
        transition_to_standing(&mut m, t);

        m.state.standing_bout_started = Some(now - Duration::minutes(5));
        // Simulate 30-minute sleep gap while standing.
        m.state.last_tick_ts = Some(now - Duration::minutes(30));

        // Should not panic; apply_break_credit(0) is called but credit is harmless.
        let _ = m.on_reading_at(1200, true, now);

        assert_eq!(
            m.state.sitting_seconds, 0,
            "sitting_seconds must stay 0 after sleep gap in Standing state"
        );
        // standing_bout_started must have been rewound to prevent standing inflation.
        assert_eq!(
            m.state.standing_bout_started,
            Some(now - Duration::seconds(1)),
            "standing_bout_started should be rewound to just before the reading"
        );
    }

    /// Sleep gap must NOT reduce sitting_seconds_total — that counter tracks raw
    /// historical accumulation and must never be reduced by break credit.
    #[test]
    fn sleep_gap_preserves_sitting_seconds_total() {
        let mut m = SessionManager::new();
        let now = base();
        transition_to_sitting(&mut m, now - Duration::hours(3));

        m.state.sitting_seconds = 2400;
        m.state.sitting_seconds_total = 2400;
        // 2-hour sleep gap → full credit, sitting_seconds → 0.
        m.state.last_tick_ts = Some(now - Duration::hours(2));

        let _ = m.on_reading_at(800, true, now);

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
