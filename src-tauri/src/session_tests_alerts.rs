//! session_tests_alerts.rs — Tests for stand alerts, praise, and position changes.

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    // ─── Stand Limit Alert Tests ─────────────────────────────────────────────

    #[test]
    fn should_stand_alert_fires_once_per_standing_stint() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Standing;
        m.state.stand_limit_secs = 900;
        m.state.break_seconds = 901;
        assert!(m.should_stand_alert());
        assert!(m.stand_alert_fired);
        assert!(!m.should_stand_alert());
    }

    #[test]
    fn should_stand_alert_disabled_when_stand_limit_is_zero() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Standing;
        m.state.stand_limit_secs = 0;
        m.state.break_seconds = 1000;
        assert!(!m.should_stand_alert());
    }

    #[test]
    fn should_stand_alert_reset_on_state_change() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Standing;
        m.state.stand_limit_secs = 900;
        m.state.break_seconds = 901;
        assert!(m.should_stand_alert());

        // Transition Standing -> Sitting
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(901));
        let _ = m.on_reading(800, true);
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(800, true);
        }

        // Back to Standing
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(1200, true);
        }

        m.state.break_seconds = 901;
        assert!(m.should_stand_alert(), "should fire again in new stint");
    }

    // ─── Praise Halfway Tests ────────────────────────────────────────────────

    #[test]
    fn should_send_praise_halfway_fires_at_50_percent() {
        let mut m = SessionManager::new();
        let config = crate::config::AppConfig {
            notify_praise_halfway: true,
            standing_target_mins: 20,
            ..Default::default()
        };
        m.state.stand_limit_secs = 1200;
        m.state.standing_seconds = 600;
        assert!(m.should_send_praise_halfway(&config));
        assert!(m.praise_halfway_fired_today);
    }

    #[test]
    fn should_send_praise_halfway_not_disabled() {
        let mut m = SessionManager::new();
        let config = crate::config::AppConfig {
            notify_praise_halfway: false,
            standing_target_mins: 20,
            ..Default::default()
        };
        m.state.stand_limit_secs = 1200;
        m.state.standing_seconds = 600;
        assert!(!m.should_send_praise_halfway(&config));
    }

    #[test]
    fn should_send_praise_halfway_disabled_when_stand_limit_zero() {
        let mut m = SessionManager::new();
        let config = crate::config::AppConfig {
            notify_praise_halfway: true,
            ..Default::default()
        };
        m.state.stand_limit_secs = 0;
        m.state.standing_seconds = 600;
        assert!(!m.should_send_praise_halfway(&config));
    }

    #[test]
    fn should_send_praise_halfway_fires_once_per_day() {
        let mut m = SessionManager::new();
        let config = crate::config::AppConfig {
            notify_praise_halfway: true,
            standing_target_mins: 20,
            ..Default::default()
        };
        m.state.stand_limit_secs = 1200;
        m.state.standing_seconds = 600;
        m.state.state = DeskState::Standing;
        assert!(m.should_send_praise_halfway(&config));
        assert!(!m.should_send_praise_halfway(&config), "should not fire twice");
    }

    // ─── Position Changes Tests ──────────────────────────────────────────────

    #[test]
    fn position_changes_increments_sitting_to_standing() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Sitting;
        m.state.sitting_started = Some(Utc::now() - chrono::Duration::seconds(100));
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(1200, true);
        }
        assert_eq!(m.state.state, DeskState::Standing);
        assert_eq!(m.state.position_changes, 1);
    }

    #[test]
    fn position_changes_increments_standing_to_sitting() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Standing;
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(300));
        m.state.position_changes = 1;
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(800, true);
        }
        assert_eq!(m.state.state, DeskState::Sitting);
        assert_eq!(m.state.position_changes, 2);
    }

    #[test]
    fn position_changes_does_not_increment_standing_to_walking() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Standing;
        m.state.break_started = Some(Utc::now());
        m.state.position_changes = 5;
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(1200, false);
        }
        assert_eq!(m.state.state, DeskState::Walking);
        assert_eq!(m.state.position_changes, 5);
    }

    #[test]
    fn position_changes_does_not_increment_walking_to_standing() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Walking;
        m.state.break_started = Some(Utc::now());
        m.state.position_changes = 3;
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(1200, true);
        }
        assert_eq!(m.state.state, DeskState::Standing);
        assert_eq!(m.state.position_changes, 3);
    }

    #[test]
    fn position_changes_resets_on_daily_reset() {
        let mut m = SessionManager::new();
        m.state.position_changes = 12;
        m.state.sitting_seconds = 2400;
        m.last_reset_date = Utc::now().date_naive() - chrono::Duration::days(1);
        m.last_reset_check = Utc::now() - chrono::Duration::seconds(120);
        let reset_happened = m.check_daily_reset();
        assert!(reset_happened);
        assert_eq!(m.state.position_changes, 0);
    }

    #[test]
    fn state_changed_payload_includes_position_changes() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Sitting;
        m.state.sitting_started = Some(Utc::now() - chrono::Duration::seconds(100));
        m.state.position_changes = 2;
        for _ in 0..DEBOUNCE_COUNT {
            let result = m.on_reading(1200, true);
            if let Some(payload) = result.state_change {
                assert_eq!(payload.position_changes, 3);
                return;
            }
        }
        panic!("expected state change payload");
    }
}
