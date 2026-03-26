//! session_tests_sleep.rs — Sleep/suspend gap detection tests.

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

    /// Regression: computer sleep inflates sitting_seconds.
    #[test]
    fn sleep_does_not_inflate_sitting_seconds() {
        let mut m = SessionManager::new();
        let now = Utc::now();

        advance_ticks(&mut m, 800, true, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Sitting);

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

        advance_ticks(&mut m, 800, true, DEBOUNCE_COUNT as usize);
        m.state.sitting_started = Some(now);
        advance_ticks(&mut m, 1200, true, DEBOUNCE_COUNT as usize);
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
}
