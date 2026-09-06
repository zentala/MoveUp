//! session_tests_limits.rs — Ergonomic limits are config, not state (E020-T02).
//!
//! The six threshold fields used to be copied into `SessionState` at
//! construction and never refreshed, so editing an ergonomic profile on disk
//! changed nothing until restart. They now live on `SessionManager::limits`
//! and every engine function reads them from there.

#[cfg(test)]
mod limits_tests {
    use chrono::{DateTime, Duration, TimeZone, Utc};

    use crate::config::AppConfig;
    use crate::ergonomic_profile::ErgonomicProfile;
    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    /// A fixed instant, so nothing here depends on when the suite runs.
    fn base() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 1, 9, 0, 0).unwrap()
    }

    /// happy — a profile swapped mid-session changes the NEXT credit, with no
    /// restart. Before this task the multiplier was frozen at construction.
    #[test]
    fn reloaded_profile_changes_the_next_break_credit() {
        let mut m = SessionManager::new();
        m.state.sitting_seconds = 1800;

        let mut ergo = ErgonomicProfile::default();
        ergo.limits.break_credit_multiplier = 4.0;
        m.set_ergo_profile(&ergo);

        m.apply_break_credit(120);

        assert_eq!(
            m.state.sitting_seconds,
            1800 - 480,
            "the reloaded multiplier (4.0), not the construction-time one, must apply"
        );
    }

    /// happy — the same for a threshold that gates a notification.
    #[test]
    fn reloaded_profile_changes_posture_balance_threshold() {
        let comm = crate::communication_profile::CommunicationProfile::default();
        let mut m = SessionManager::new();
        m.state.state = DeskState::Sitting;
        m.state.sitting_seconds_total = 3600;
        m.state.standing_seconds = 0;

        let events = m.check_notification_conditions_at(&comm, base());
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, NotificationEvent::PostureBalance)),
            "1 h sitting is below the default 6 h floor"
        );

        let mut ergo = ErgonomicProfile::default();
        ergo.limits.posture_balance_min_sitting_secs = 1800;
        m.set_limits(&ergo.limits);

        let events = m.check_notification_conditions_at(&comm, base());
        assert!(
            events
                .iter()
                .any(|e| matches!(e, NotificationEvent::PostureBalance)),
            "lowering the floor to 30 min must make it reachable on the next check"
        );
    }

    /// nil — a profile left at its serde defaults behaves like the constants
    /// the manager used before the fields moved out of state.
    #[test]
    fn default_profile_matches_previous_hardcoded_defaults() {
        let m = SessionManager::new();
        assert_eq!(m.limits.break_min_secs, 60);
        assert_eq!(m.limits.break_credit_multiplier, 2.0);
        assert_eq!(m.limits.day_break_min_secs, 21600);
        assert_eq!(m.limits.posture_balance_min_sitting_secs, 21600);
        assert_eq!(m.limits.max_continuous_computer_secs, 3600);
        assert_eq!(m.limits.computer_break_reset_secs, 300);
    }

    /// nil — `new_from_config` seeds the limits from the profile it is given.
    #[test]
    fn new_from_config_seeds_limits_from_the_profile() {
        let mut ergo = ErgonomicProfile::default();
        ergo.limits.break_min_secs = 15;
        ergo.limits.computer_break_reset_secs = 42;
        let m = SessionManager::new_from_config(&AppConfig::default(), &ergo);

        assert_eq!(m.limits.break_min_secs, 15);
        assert_eq!(m.limits.computer_break_reset_secs, 42);
    }

    /// empty — a zeroed day-break threshold still disables the day reset when
    /// the value is read from the profile instead of from state.
    #[test]
    fn zero_day_break_threshold_still_disables_the_day_reset() {
        let mut m = SessionManager::new();
        m.limits.day_break_min_secs = 0;
        m.notify_posture_balance_fired = true;
        m.state.daily_score = 42.0;

        m.apply_break_credit(100_000);

        assert!(m.notify_posture_balance_fired, "flags unchanged when disabled");
        assert_eq!(m.state.daily_score, 42.0);
    }

    /// error — an out-of-range multiplier is still clamped to 10.0.
    #[test]
    fn out_of_range_multiplier_is_still_clamped() {
        let mut m = SessionManager::new();
        m.state.sitting_seconds = 1000;
        m.limits.break_credit_multiplier = 99.0;

        m.apply_break_credit(60);

        assert_eq!(
            m.state.sitting_seconds, 400,
            "60 s x clamp(99.0) = 600 s of credit, not 5940"
        );
    }

    /// The DTO reports the live limit, so the UI follows a reloaded profile.
    #[test]
    fn snapshot_reports_the_live_computer_limit() {
        let mut m = SessionManager::new();
        m.limits.max_continuous_computer_secs = 1234;

        assert_eq!(m.snapshot().max_continuous_computer_secs, 1234);
    }

    /// The Away-reset threshold is read per tick from the profile.
    #[test]
    fn computer_reset_threshold_follows_the_live_profile() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Away;
        m.state.continuous_computer_secs = 4500;
        m.limits.computer_break_reset_secs = 3;

        for i in 0..3 {
            let now = base() + Duration::seconds(i);
            m.state.last_accumulate_ts =
                if i == 0 { None } else { Some(now - Duration::seconds(1)) };
            m.accumulate_ongoing(now);
        }

        assert_eq!(m.state.continuous_computer_secs, 0);
    }
}
