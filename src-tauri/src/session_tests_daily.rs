//! session_tests_daily.rs — Tests for daily reset and notification conditions.

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    // ─── Daily Reset Tests ───────────────────────────────────────────────────

    #[test]
    fn check_daily_reset_resets_standing_seconds() {
        let mut m = SessionManager::new();
        m.state.standing_seconds = 3600;
        m.state.sitting_seconds = 2400;
        m.alert_fired = true;
        m.stand_alert_fired = true;
        m.last_reset_date = Utc::now().date_naive() - chrono::Duration::days(1);
        m.last_reset_check = Utc::now() - chrono::Duration::seconds(120);
        let reset_happened = m.check_daily_reset();
        assert!(reset_happened, "daily reset should have triggered");
        assert_eq!(m.state.standing_seconds, 0);
        assert_eq!(m.state.sitting_seconds, 0);
        assert!(!m.alert_fired);
        assert!(!m.stand_alert_fired);
    }

    #[test]
    fn check_daily_reset_does_not_reset_same_day() {
        let mut m = SessionManager::new();
        m.state.sitting_seconds = 1000;
        m.state.standing_seconds = 500;
        m.last_reset_date = Utc::now().date_naive();
        m.last_reset_check = Utc::now() - chrono::Duration::seconds(30);
        let reset_happened = m.check_daily_reset();
        assert!(!reset_happened, "reset should not happen on same day");
        assert_eq!(m.state.sitting_seconds, 1000);
        assert_eq!(m.state.standing_seconds, 500);
    }

    #[test]
    fn check_daily_reset_throttled_every_60_seconds() {
        let mut m = SessionManager::new();
        m.state.sitting_seconds = 1000;
        m.last_reset_date = Utc::now().date_naive() - chrono::Duration::days(1);
        m.last_reset_check = Utc::now() - chrono::Duration::seconds(30);
        let reset_happened = m.check_daily_reset();
        assert!(!reset_happened, "reset should be throttled if < 60 seconds");
        assert_eq!(m.state.sitting_seconds, 1000);
    }

    #[test]
    fn check_daily_reset_idempotent_same_day() {
        let mut m = SessionManager::new();
        m.state.sitting_seconds = 500;
        m.last_reset_date = Utc::now().date_naive();
        let first = m.check_daily_reset();
        let second = m.check_daily_reset();
        assert!(!first && !second, "same day should return false");
        assert_eq!(m.state.sitting_seconds, 500);
    }

    #[test]
    fn check_daily_reset_clears_all_notification_flags() {
        let mut m = SessionManager::new();
        m.notify_inactivity_fired = true;
        m.notify_posture_balance_fired = true;
        m.praise_halfway_fired_today = true;
        m.standing_target_reached_fired = true;
        m.alert_fired = true;
        m.stand_alert_fired = true;
        m.last_reset_date = Utc::now().date_naive() - chrono::Duration::days(1);
        m.last_reset_check = Utc::now() - chrono::Duration::seconds(120);
        let _ = m.check_daily_reset();
        assert!(!m.notify_inactivity_fired);
        assert!(!m.notify_posture_balance_fired);
        assert!(!m.praise_halfway_fired_today);
        assert!(!m.standing_target_reached_fired);
        assert!(!m.alert_fired);
        assert!(!m.stand_alert_fired);
    }

    // ─── Notification Condition Tests ────────────────────────────────────────

    #[test]
    fn check_notification_conditions_inactivity_fires_after_60min() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Sitting;
        let config = crate::config::AppConfig {
            notify_inactivity: true,
            ..Default::default()
        };
        m.state.last_position_change_at =
            Some(Utc::now() - chrono::Duration::minutes(61));
        let events = m.check_notification_conditions(&config);
        assert!(events.iter().any(|e| matches!(e, NotificationEvent::Inactivity)));
        assert!(m.notify_inactivity_fired);
    }

    #[test]
    fn check_notification_conditions_inactivity_not_disabled() {
        let mut m = SessionManager::new();
        let config = crate::config::AppConfig {
            notify_inactivity: false,
            ..Default::default()
        };
        m.state.last_position_change_at =
            Some(Utc::now() - chrono::Duration::minutes(61));
        let events = m.check_notification_conditions(&config);
        assert!(!events.iter().any(|e| matches!(e, NotificationEvent::Inactivity)));
        assert!(!m.notify_inactivity_fired);
    }

    #[test]
    fn check_notification_conditions_posture_balance_fires() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Sitting;
        let config = crate::config::AppConfig {
            notify_daily_posture_balance: true,
            ..Default::default()
        };
        m.state.sitting_seconds = 3600;
        m.state.standing_seconds = 1000;
        let events = m.check_notification_conditions(&config);
        assert!(events.iter().any(|e| matches!(e, NotificationEvent::PostureBalance)));
        assert!(m.notify_posture_balance_fired);
    }

    #[test]
    fn check_notification_conditions_posture_balance_not_disabled() {
        let mut m = SessionManager::new();
        let config = crate::config::AppConfig {
            notify_daily_posture_balance: false,
            ..Default::default()
        };
        m.state.sitting_seconds = 3600;
        m.state.standing_seconds = 1000;
        let events = m.check_notification_conditions(&config);
        assert!(!events.iter().any(|e| matches!(e, NotificationEvent::PostureBalance)));
    }

    #[test]
    fn check_notification_conditions_all_reset_on_daily_reset() {
        let mut m = SessionManager::new();
        let config = crate::config::AppConfig::default();
        m.notify_inactivity_fired = true;
        m.notify_posture_balance_fired = true;
        m.praise_halfway_fired_today = true;
        m.standing_target_reached_fired = true;
        m.last_reset_date = Utc::now().date_naive() - chrono::Duration::days(1);
        m.last_reset_check = Utc::now() - chrono::Duration::seconds(120);
        let _ = m.check_daily_reset();
        assert!(!m.standing_target_reached_fired);
        let events = m.check_notification_conditions(&config);
        assert_eq!(events.len(), 0, "no notifications should fire after reset");
    }

    #[test]
    fn standing_target_reached_fires_when_standing_exceeds_limit() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Standing;
        m.state.stand_limit_secs = 900; // 15 min
        m.state.standing_seconds = 900;
        let config = crate::config::AppConfig::default();
        let events = m.check_notification_conditions(&config);
        assert!(events.iter().any(|e| matches!(e, NotificationEvent::StandingTargetReached)));
        assert!(m.standing_target_reached_fired);
    }

    #[test]
    fn standing_target_reached_does_not_fire_twice() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Standing;
        m.state.stand_limit_secs = 900;
        m.state.standing_seconds = 1000;
        let config = crate::config::AppConfig::default();
        let _ = m.check_notification_conditions(&config);
        assert!(m.standing_target_reached_fired);
        let events = m.check_notification_conditions(&config);
        assert!(!events.iter().any(|e| matches!(e, NotificationEvent::StandingTargetReached)));
    }

    #[test]
    fn standing_target_reached_resets_on_daily_reset() {
        let mut m = SessionManager::new();
        m.standing_target_reached_fired = true;
        m.last_reset_date = Utc::now().date_naive() - chrono::Duration::days(1);
        m.last_reset_check = Utc::now() - chrono::Duration::seconds(120);
        let _ = m.check_daily_reset();
        assert!(!m.standing_target_reached_fired);
    }
}
