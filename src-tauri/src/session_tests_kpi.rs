//! session_tests_kpi.rs — Tests for KPI-related SessionManager extensions.

#[cfg(test)]
mod tests {
    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    const SITTING_MM: i32 = 720;
    const STANDING_MM: i32 = 1050;

    fn feed_sitting(mgr: &mut SessionManager, n: usize) {
        for _ in 0..n { mgr.on_reading(SITTING_MM, true); }
    }

    fn feed_standing(mgr: &mut SessionManager, n: usize) {
        for _ in 0..n { mgr.on_reading(STANDING_MM, true); }
    }

    /// Simulates `secs` seconds of accumulation with advancing timestamps.
    fn tick_seconds(mgr: &mut SessionManager, start: chrono::DateTime<chrono::Utc>, secs: i64) {
        for i in 0..secs {
            mgr.accumulate_ongoing(start + chrono::Duration::seconds(i));
        }
    }

    // ─── continuous_computer_secs ───────────────────────────────────────────

    #[test]
    fn continuous_computer_secs_increments_while_sitting() {
        let mut mgr = SessionManager::new();
        feed_sitting(&mut mgr, DEBOUNCE_COUNT as usize);
        assert_eq!(mgr.state.state, DeskState::Sitting);
        let now = chrono::Utc::now() + chrono::Duration::seconds(10);
        let before = mgr.state.continuous_computer_secs;
        tick_seconds(&mut mgr, now, 10);
        assert!(
            mgr.state.continuous_computer_secs >= before + 9,
            "expected at least {} more, got {}",
            9, mgr.state.continuous_computer_secs - before
        );
    }

    #[test]
    fn continuous_computer_secs_increments_while_standing() {
        let mut mgr = SessionManager::new();
        feed_standing(&mut mgr, DEBOUNCE_COUNT as usize);
        assert_eq!(mgr.state.state, DeskState::Standing);
        let now = chrono::Utc::now() + chrono::Duration::seconds(10);
        let before = mgr.state.continuous_computer_secs;
        tick_seconds(&mut mgr, now, 10);
        assert!(
            mgr.state.continuous_computer_secs >= before + 9,
            "expected at least {} more, got {}",
            9, mgr.state.continuous_computer_secs - before
        );
    }

    #[test]
    fn continuous_computer_secs_resets_after_5min_away() {
        let mut mgr = SessionManager::new();
        feed_sitting(&mut mgr, DEBOUNCE_COUNT as usize);
        assert_eq!(mgr.state.state, DeskState::Sitting);
        // Simulate accumulated sitting time (feed_sitting runs too fast for throttle)
        mgr.state.continuous_computer_secs = 50;
        // Manually set to Away (sensor can't produce Away directly).
        mgr.state.state = DeskState::Away;
        let now = chrono::Utc::now() + chrono::Duration::seconds(100);
        tick_seconds(&mut mgr, now, 305);
        assert_eq!(mgr.state.continuous_computer_secs, 0);
    }

    #[test]
    fn four_min_59s_away_does_not_reset_continuous_timer() {
        let mut mgr = SessionManager::new();
        feed_sitting(&mut mgr, DEBOUNCE_COUNT as usize);
        assert_eq!(mgr.state.state, DeskState::Sitting);
        // Simulate accumulated sitting time (feed_sitting runs too fast for throttle)
        mgr.state.continuous_computer_secs = 50;
        let before = mgr.state.continuous_computer_secs;
        mgr.state.state = DeskState::Away;
        let now = chrono::Utc::now() + chrono::Duration::seconds(100);
        tick_seconds(&mut mgr, now, 299);
        // Away < 5 min: continuous timer keeps its value (not reset).
        assert_eq!(mgr.state.continuous_computer_secs, before);
        // Return to Sitting: timer continues.
        mgr.state.state = DeskState::Sitting;
        mgr.state.away_bout_secs = 0;
        let after_away = now + chrono::Duration::seconds(300);
        tick_seconds(&mut mgr, after_away, 5);
        assert!(mgr.state.continuous_computer_secs > before);
    }

    // ─── longest_computer_session_secs ──────────────────────────────────────

    #[test]
    fn longest_computer_session_tracks_daily_max() {
        let mut mgr = SessionManager::new();
        let now = chrono::Utc::now();
        mgr.state.state = DeskState::Sitting;
        tick_seconds(&mut mgr, now, 100);
        assert_eq!(mgr.state.longest_computer_session_secs, 100);
        // Away 5+ min resets continuous but longest stays.
        mgr.state.state = DeskState::Away;
        let t2 = now + chrono::Duration::seconds(200);
        tick_seconds(&mut mgr, t2, 305);
        assert_eq!(mgr.state.continuous_computer_secs, 0);
        assert_eq!(mgr.state.longest_computer_session_secs, 100);
        // Second shorter session doesn't change longest.
        mgr.state.state = DeskState::Sitting;
        mgr.state.away_bout_secs = 0;
        let t3 = t2 + chrono::Duration::seconds(400);
        tick_seconds(&mut mgr, t3, 50);
        assert_eq!(mgr.state.longest_computer_session_secs, 100);
    }

    // ─── away_bout_secs ─────────────────────────────────────────────────────

    #[test]
    fn away_bout_secs_tracks_continuous_away() {
        let mut mgr = SessionManager::new();
        mgr.state.state = DeskState::Away;
        let now = chrono::Utc::now();
        tick_seconds(&mut mgr, now, 60);
        assert_eq!(mgr.state.away_bout_secs, 60);
    }

    // ─── Away DOES count as position change ──────────────────────────────────

    #[test]
    fn position_changes_incremented_on_5min_away() {
        let mut mgr = SessionManager::new();
        mgr.state.continuous_computer_secs = 4500;
        let initial = mgr.state.position_changes;
        mgr.state.state = DeskState::Away;
        let now = chrono::Utc::now();
        tick_seconds(&mut mgr, now, 305);
        assert_eq!(mgr.state.position_changes, initial + 1, "away reset IS a posture change");
    }

    #[test]
    fn position_changes_incremented_for_10min_away() {
        let mut mgr = SessionManager::new();
        mgr.state.continuous_computer_secs = 4500;
        let initial = mgr.state.position_changes;
        mgr.state.state = DeskState::Away;
        let now = chrono::Utc::now();
        tick_seconds(&mut mgr, now, 605);
        assert_eq!(mgr.state.position_changes, initial + 1, "away reset IS a posture change");
    }

    // ─── first_reading_at ───────────────────────────────────────────────────

    #[test]
    fn first_reading_at_set_on_first_reading_only() {
        let mut mgr = SessionManager::new();
        assert!(mgr.state.first_reading_at.is_none());
        mgr.on_reading(SITTING_MM, true);
        let first = mgr.state.first_reading_at.unwrap();
        for _ in 0..10 { mgr.on_reading(SITTING_MM, true); }
        assert_eq!(mgr.state.first_reading_at.unwrap(), first);
    }

    // ─── daily reset ────────────────────────────────────────────────────────

    #[test]
    fn daily_reset_clears_new_kpi_fields() {
        let mut mgr = SessionManager::new();
        let now = chrono::Utc::now();
        mgr.state.continuous_computer_secs = 500;
        mgr.state.longest_computer_session_secs = 1000;
        mgr.state.away_bout_secs = 42;
        mgr.state.first_reading_at = Some(now);
        mgr.state.last_tick_ts = Some(now);
        mgr.state.last_accumulate_ts = Some(now);
        mgr.last_reset_date = now.date_naive() - chrono::Duration::days(1);
        mgr.last_reset_check = now - chrono::Duration::seconds(120);
        assert!(mgr.check_daily_reset());
        assert_eq!(mgr.state.continuous_computer_secs, 0);
        assert_eq!(mgr.state.longest_computer_session_secs, 0);
        assert_eq!(mgr.state.away_bout_secs, 0);
        assert!(mgr.state.first_reading_at.is_none());
        assert!(mgr.state.last_tick_ts.is_none());
        assert!(mgr.state.last_accumulate_ts.is_none());
    }
}
