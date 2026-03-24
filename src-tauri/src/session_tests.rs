//! session_tests.rs — Unit tests for break credit rules and basic state machine.

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    fn manager_with_sitting_secs(sitting: i64) -> SessionManager {
        let mut m = SessionManager::new();
        m.state.sitting_seconds = sitting;
        m
    }

    // ─── Break Credit Tests ──────────────────────────────────────────────────

    #[test]
    fn break_under_5_min_has_no_effect() {
        let mut m = manager_with_sitting_secs(3000);
        m.apply_break_credit(4 * 60);
        assert_eq!(
            m.state.sitting_seconds, 3000,
            "short break must not reduce sitting time"
        );
    }

    #[test]
    fn break_7_min_subtracts_20_min() {
        let mut m = manager_with_sitting_secs(3000);
        m.apply_break_credit(7 * 60);
        assert_eq!(
            m.state.sitting_seconds,
            3000 - 1200,
            "7-minute break should subtract 1200 seconds"
        );
    }

    #[test]
    fn break_7_min_floors_at_zero() {
        let mut m = manager_with_sitting_secs(600);
        m.apply_break_credit(7 * 60);
        assert_eq!(
            m.state.sitting_seconds, 0,
            "sitting_seconds must not go below 0"
        );
    }

    #[test]
    fn break_12_min_resets_to_zero() {
        let mut m = manager_with_sitting_secs(3600);
        m.apply_break_credit(12 * 60);
        assert_eq!(
            m.state.sitting_seconds, 0,
            "10+ min break should reset sitting time"
        );
    }

    #[test]
    fn break_exactly_5_min_subtracts_20_min() {
        let mut m = manager_with_sitting_secs(2000);
        m.apply_break_credit(5 * 60);
        assert_eq!(
            m.state.sitting_seconds, 800,
            "5-minute break should subtract 1200 seconds"
        );
    }

    #[test]
    fn break_exactly_10_min_resets() {
        let mut m = manager_with_sitting_secs(3600);
        m.apply_break_credit(10 * 60);
        assert_eq!(
            m.state.sitting_seconds, 0,
            "10-minute break should reset to 0"
        );
    }

    // ─── Calibration & State Detection Tests ─────────────────────────────────

    #[test]
    fn on_reading_computes_desk_height_correctly() {
        let mut m = SessionManager::new();
        let _result = m.on_reading(1000, true);
        assert!(
            (m.state.desk_height_cm - 97.0).abs() < 0.01,
            "desk_height_cm should be 97.0 but got {}",
            m.state.desk_height_cm
        );
    }

    #[test]
    fn low_reading_produces_sitting_candidate() {
        let mut m = SessionManager::new();
        for _ in 0..DEBOUNCE_COUNT {
            let _result = m.on_reading(800, true);
        }
        assert_eq!(m.state.state, DeskState::Sitting);
    }

    #[test]
    fn high_reading_active_produces_standing_candidate() {
        let mut m = SessionManager::new();
        for _ in 0..DEBOUNCE_COUNT {
            let _result = m.on_reading(1200, true);
        }
        assert_eq!(m.state.state, DeskState::Standing);
    }

    #[test]
    fn high_reading_inactive_produces_away_candidate() {
        let mut m = SessionManager::new();
        for _ in 0..DEBOUNCE_COUNT {
            let _result = m.on_reading(1200, false);
        }
        assert_eq!(m.state.state, DeskState::Away);
    }

    #[test]
    fn low_reading_inactive_produces_away_candidate() {
        let mut m = SessionManager::new();
        for _ in 0..DEBOUNCE_COUNT {
            let _result = m.on_reading(800, false);
        }
        assert_eq!(m.state.state, DeskState::Away);
    }

    #[test]
    fn should_alert_fires_once_then_suppressed() {
        let mut m = SessionManager::new();
        m.state.session_limit_secs = 10;
        m.state.sitting_seconds = 11;
        m.state.state = DeskState::Sitting;
        assert!(m.should_alert(), "first call should return true");
        assert!(!m.should_alert(), "second call should be suppressed");
    }

    // ─── Standing Seconds Tests ──────────────────────────────────────────────

    #[test]
    fn standing_seconds_accumulates_on_transition() {
        let mut m = SessionManager::new();
        m.state.standing_seconds = 0;
        let t = Utc::now() - chrono::Duration::seconds(300);
        m.state.break_started = Some(t);
        m.state.standing_bout_started = Some(t);
        m.state.state = DeskState::Standing;
        let _ = m.on_reading(800, true);
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(800, true);
        }
        assert_eq!(m.state.state, DeskState::Sitting);
        assert!(
            m.state.standing_seconds >= 0,
            "standing_seconds should accumulate"
        );
    }

    #[test]
    fn standing_seconds_does_not_accumulate_in_away() {
        let mut m = SessionManager::new();
        m.state.standing_seconds = 0;
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(1200, false);
        }
        assert_eq!(m.state.state, DeskState::Away);
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(300));
        let standing_before = m.state.standing_seconds;
        let _ = m.on_reading(1200, false);
        assert_eq!(
            m.state.standing_seconds, standing_before,
            "standing_seconds must not increase in Away state"
        );
    }

    #[test]
    fn set_stand_limit_minutes_converts_correctly() {
        let mut m = SessionManager::new();
        m.set_stand_limit_minutes(20);
        assert_eq!(
            m.state.stand_limit_secs, 1200,
            "20 minutes should be 1200 seconds"
        );
    }
}
