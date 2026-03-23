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

    // ─── continuous_computer_secs ───────────────────────────────────────────

    #[test]
    fn continuous_computer_secs_increments_while_sitting() {
        let mut mgr = SessionManager::new();
        feed_sitting(&mut mgr, DEBOUNCE_COUNT as usize);
        assert_eq!(mgr.state.state, DeskState::Sitting);
        let before = mgr.state.continuous_computer_secs;
        feed_sitting(&mut mgr, 10);
        assert_eq!(mgr.state.continuous_computer_secs, before + 10);
    }

    #[test]
    fn continuous_computer_secs_increments_while_standing() {
        let mut mgr = SessionManager::new();
        feed_standing(&mut mgr, DEBOUNCE_COUNT as usize);
        assert_eq!(mgr.state.state, DeskState::Standing);
        let before = mgr.state.continuous_computer_secs;
        feed_standing(&mut mgr, 10);
        assert_eq!(mgr.state.continuous_computer_secs, before + 10);
    }

    #[test]
    fn continuous_computer_secs_resets_after_5min_away() {
        let mut mgr = SessionManager::new();
        feed_sitting(&mut mgr, DEBOUNCE_COUNT as usize + 50);
        assert!(mgr.state.continuous_computer_secs > 0);
        // Manually set to Away (sensor can't produce Away directly).
        mgr.state.state = DeskState::Away;
        let now = chrono::Utc::now();
        for _ in 0..305 { mgr.accumulate_ongoing(now); }
        assert_eq!(mgr.state.continuous_computer_secs, 0);
    }

    #[test]
    fn four_min_59s_away_does_not_reset_continuous_timer() {
        let mut mgr = SessionManager::new();
        feed_sitting(&mut mgr, DEBOUNCE_COUNT as usize + 50);
        let before = mgr.state.continuous_computer_secs;
        assert!(before > 0);
        mgr.state.state = DeskState::Away;
        let now = chrono::Utc::now();
        for _ in 0..299 { mgr.accumulate_ongoing(now); }
        assert_eq!(mgr.state.continuous_computer_secs, before);
        // Return to Sitting: timer continues.
        mgr.state.state = DeskState::Sitting;
        mgr.state.away_bout_secs = 0;
        for _ in 0..5 { mgr.accumulate_ongoing(now); }
        assert!(mgr.state.continuous_computer_secs > before);
    }

    // ─── longest_computer_session_secs ──────────────────────────────────────

    #[test]
    fn longest_computer_session_tracks_daily_max() {
        let mut mgr = SessionManager::new();
        let now = chrono::Utc::now();
        mgr.state.state = DeskState::Sitting;
        for _ in 0..100 { mgr.accumulate_ongoing(now); }
        assert_eq!(mgr.state.longest_computer_session_secs, 100);
        // Away 5+ min resets continuous but longest stays.
        mgr.state.state = DeskState::Away;
        for _ in 0..305 { mgr.accumulate_ongoing(now); }
        assert_eq!(mgr.state.continuous_computer_secs, 0);
        assert_eq!(mgr.state.longest_computer_session_secs, 100);
        // Second shorter session doesn't change longest.
        mgr.state.state = DeskState::Sitting;
        mgr.state.away_bout_secs = 0;
        for _ in 0..50 { mgr.accumulate_ongoing(now); }
        assert_eq!(mgr.state.longest_computer_session_secs, 100);
    }

    // ─── away_bout_secs ─────────────────────────────────────────────────────

    #[test]
    fn away_bout_secs_tracks_continuous_away() {
        let mut mgr = SessionManager::new();
        mgr.state.state = DeskState::Away;
        let now = chrono::Utc::now();
        for _ in 0..60 { mgr.accumulate_ongoing(now); }
        assert_eq!(mgr.state.away_bout_secs, 60);
    }

    // ─── position_changes from 5min Away ────────────────────────────────────

    #[test]
    fn position_changes_incremented_on_5min_away() {
        let mut mgr = SessionManager::new();
        let initial = mgr.state.position_changes;
        mgr.state.state = DeskState::Away;
        let now = chrono::Utc::now();
        for _ in 0..305 { mgr.accumulate_ongoing(now); }
        assert_eq!(mgr.state.position_changes, initial + 1);
    }

    #[test]
    fn position_changes_not_incremented_twice_for_10min_away() {
        let mut mgr = SessionManager::new();
        let initial = mgr.state.position_changes;
        mgr.state.state = DeskState::Away;
        let now = chrono::Utc::now();
        for _ in 0..600 { mgr.accumulate_ongoing(now); }
        assert_eq!(mgr.state.position_changes, initial + 1);
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
        mgr.last_reset_date = now.date_naive() - chrono::Duration::days(1);
        mgr.last_reset_check = now - chrono::Duration::seconds(120);
        assert!(mgr.check_daily_reset());
        assert_eq!(mgr.state.continuous_computer_secs, 0);
        assert_eq!(mgr.state.longest_computer_session_secs, 0);
        assert_eq!(mgr.state.away_bout_secs, 0);
        assert!(mgr.state.first_reading_at.is_none());
        assert!(mgr.state.last_tick_ts.is_none());
    }
}
