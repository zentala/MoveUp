//! session_tests_day_break.rs — Tests for Day Break Credit (ADR 009).
//!
//! `apply_break_credit` never read the clock; the notification checks did, so
//! they now take an explicit instant (E020-T01).

#[cfg(test)]
mod day_break_tests {
    use chrono::{DateTime, TimeZone, Utc};

    use crate::communication_profile::CommunicationProfile;
    use crate::notification_service::NotificationService;
    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    /// A fixed instant, so nothing here depends on when the suite runs.
    fn base() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 1, 9, 0, 0).unwrap()
    }

    #[test]
    fn day_break_credit_resets_notification_flags() {
        let mut m = SessionManager::new();
        m.notify_inactivity_fired = true;
        m.notify_posture_balance_fired = true;
        m.praise_halfway_fired_today = true;
        m.standing_target_reached_fired = true;
        m.state.daily_score = 42.0;

        m.apply_break_credit(21600); // 6h break

        assert!(!m.notify_inactivity_fired);
        assert!(!m.notify_posture_balance_fired);
        assert!(!m.praise_halfway_fired_today);
        assert!(!m.standing_target_reached_fired);
        assert_eq!(m.state.daily_score, 0.0);
    }

    #[test]
    fn day_break_credit_preserves_kpi_counters() {
        let mut m = SessionManager::new();
        m.state.sitting_seconds_total = 10000;
        m.state.standing_seconds = 5000;
        m.state.position_changes = 8;
        m.state.hourly_breaks_covered = 3;
        m.state.hourly_breaks_active = 5;

        m.apply_break_credit(21600); // 6h break

        assert_eq!(m.state.sitting_seconds_total, 10000);
        assert_eq!(m.state.standing_seconds, 5000);
        assert_eq!(m.state.position_changes, 8);
        assert_eq!(m.state.hourly_breaks_covered, 3);
        assert_eq!(m.state.hourly_breaks_active, 5);
    }

    #[test]
    fn day_break_credit_does_not_fire_below_threshold() {
        let mut m = SessionManager::new();
        m.notify_posture_balance_fired = true;
        m.state.daily_score = 42.0;

        m.apply_break_credit(21599); // 1 second short

        assert!(m.notify_posture_balance_fired, "flags should stay set");
        assert_eq!(m.state.daily_score, 42.0, "score should stay");
    }

    #[test]
    fn day_break_credit_disabled_when_zero() {
        let mut m = SessionManager::new();
        m.limits.day_break_min_secs = 0; // disabled
        m.notify_posture_balance_fired = true;
        m.state.daily_score = 42.0;

        m.apply_break_credit(100_000); // huge break

        assert!(m.notify_posture_balance_fired, "flags unchanged when disabled");
        assert_eq!(m.state.daily_score, 42.0);
    }

    #[test]
    fn day_break_reset_is_callable_on_its_own() {
        // The reset was extracted out of `apply_break_credit` (E020-T03); it
        // must stand alone, and it must NOT touch the sitting countdown that
        // Session Break Credit owns.
        let mut m = SessionManager::new();
        m.state.sitting_seconds = 1234;
        m.notify_posture_balance_fired = true;
        m.state.daily_score = 42.0;

        m.apply_day_break_reset(21600);

        assert!(!m.notify_posture_balance_fired);
        assert_eq!(m.state.daily_score, 0.0);
        assert!(m.day_break_applied);
        assert_eq!(
            m.state.sitting_seconds, 1234,
            "day break reset must not subtract sitting seconds"
        );
        assert_eq!(
            m.state.last_break_credit,
            BreakCredit::None,
            "day break reset must not set a session credit verdict"
        );
    }

    #[test]
    fn day_break_reset_below_threshold_is_a_no_op() {
        let mut m = SessionManager::new();
        m.notify_posture_balance_fired = true;
        m.state.daily_score = 42.0;

        m.apply_day_break_reset(21599);

        assert!(m.notify_posture_balance_fired);
        assert_eq!(m.state.daily_score, 42.0);
        assert!(!m.day_break_applied);
    }

    #[test]
    fn session_credit_still_applies_alongside_day_break_reset() {
        // Both must run for one long break: seconds subtracted AND flags reset.
        let mut m = SessionManager::new();
        m.state.sitting_seconds = 1800;
        m.notify_posture_balance_fired = true;

        m.apply_break_credit(21600);

        assert_eq!(m.state.sitting_seconds, 0);
        assert_eq!(m.state.last_break_credit, BreakCredit::Full);
        assert!(!m.notify_posture_balance_fired);
        assert!(m.day_break_applied);
    }

    #[test]
    fn short_break_reaches_neither_credit_nor_day_reset() {
        let mut m = SessionManager::new();
        m.state.sitting_seconds = 1800;
        m.notify_posture_balance_fired = true;

        m.apply_break_credit(1); // below break_min_secs

        assert_eq!(m.state.sitting_seconds, 1800);
        assert_eq!(m.state.last_break_credit, BreakCredit::None);
        assert!(m.notify_posture_balance_fired);
        assert!(!m.day_break_applied);
    }

    #[test]
    fn posture_balance_requires_minimum_sitting_total() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Sitting;
        m.state.sitting_seconds = 3600;
        m.state.sitting_seconds_total = 3600; // below 6h threshold
        m.state.standing_seconds = 0;

        let mut comm = CommunicationProfile::default();
        comm.periodic_notifications.posture_balance_enabled = true;

        let events = m.check_notification_conditions_at(&comm, base());
        assert!(
            !events.iter().any(|e| matches!(e, NotificationEvent::PostureBalance)),
            "PostureBalance should NOT fire with only 1h total sitting"
        );
    }

    #[test]
    fn posture_balance_fires_after_minimum_sitting_total() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Sitting;
        m.state.sitting_seconds = 22000;
        m.state.sitting_seconds_total = 22000;
        m.state.standing_seconds = 1000;

        let mut comm = CommunicationProfile::default();
        comm.periodic_notifications.posture_balance_enabled = true;

        let events = m.check_notification_conditions_at(&comm, base());
        assert!(
            events.iter().any(|e| matches!(e, NotificationEvent::PostureBalance)),
            "PostureBalance should fire with 6h+ total sitting"
        );
    }

    #[test]
    fn posture_balance_refires_after_day_break_reset() {
        let mut m = SessionManager::new();
        m.notify_posture_balance_fired = true;
        m.state.sitting_seconds = 22000;
        m.state.sitting_seconds_total = 22000;
        m.state.standing_seconds = 1000;

        // 6h break resets the flag
        m.apply_break_credit(21600);
        assert!(!m.notify_posture_balance_fired);

        // New sitting session after break
        m.state.state = DeskState::Sitting;
        m.state.sitting_seconds = 2000;
        m.state.standing_seconds = 500;

        let mut comm = CommunicationProfile::default();
        comm.periodic_notifications.posture_balance_enabled = true;

        let events = m.check_notification_conditions_at(&comm, base());
        // sitting_seconds_total still 22000 (above threshold)
        // sitting_seconds 2000 > standing_seconds * 2 (1000)
        assert!(
            events.iter().any(|e| matches!(e, NotificationEvent::PostureBalance)),
            "PostureBalance should fire again after day break reset"
        );
    }

    #[test]
    fn notification_message_includes_actual_numbers() {
        let events = vec![NotificationEvent::PostureBalance];
        let intents = NotificationService::build_intents(&events, 7200, 1800);
        assert_eq!(intents.len(), 1);
        assert!(
            intents[0].title.contains("2h 0m"),
            "title should contain sitting hours: {}",
            intents[0].title
        );
        assert!(
            intents[0].title.contains("0h 30m"),
            "title should contain standing hours: {}",
            intents[0].title
        );
    }
}
