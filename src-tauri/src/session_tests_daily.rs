//! session_tests_daily.rs — Tests for daily reset and notification conditions.
//!
//! The clock is injected (E020-T01): every reset test states both the instant
//! and the LOCAL calendar day it belongs to (D3). Deriving the day from
//! `Utc::now()` here would make these tests pass or fail depending on the hour
//! the suite runs at, because the local day rolls over before the UTC one.

#[test]
fn e027_local_day_intersection_midnight_dst_and_clipped_raw_totals() {
    use chrono_tz::Europe::Warsaw;
    use crate::db_sessions::{local_date_bounds, clipped_duration};
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    crate::db::init_schema(&conn).unwrap();
    for (date, expected) in [("2026-10-09", 86400), ("2026-03-29", 82800), ("2026-10-25", 90000)] {
        let (start, end) = local_date_bounds(&Warsaw, date, date).unwrap();
        assert_eq!((end - start).num_seconds(), expected);
        let a = (start - chrono::Duration::seconds(60)).to_rfc3339();
        let b = (start + chrono::Duration::seconds(60)).to_rfc3339();
        crate::db_sessions::insert_session(&conn, &a, &b, "Standing", 120).unwrap();
        let before = start.with_timezone(&Warsaw).date_naive().pred_opt().unwrap().to_string();
        let (prior_start, prior_end) = local_date_bounds(&Warsaw, &before, &before).unwrap();
        let prior = crate::db_queries::get_sessions_between(&conn, prior_start, prior_end).unwrap();
        let current = crate::db_queries::get_sessions_between(&conn, start, end).unwrap();
        assert_eq!((prior.len(), current.len()), (1, 1));
        assert_eq!(clipped_duration(&prior[0], prior_start, prior_end), Some(60));
        assert_eq!(clipped_duration(&current[0], start, end), Some(60));
        assert_eq!(current[0].duration_seconds, Some(120)); // Raw wire unchanged.
        assert_eq!(current[0].started_at, a);
        assert_eq!(current[0].ended_at.as_deref(), Some(b.as_str()));
        crate::db_sessions::insert_session(&conn, &start.to_rfc3339(), &b, "Sitting", 60).unwrap();
        assert_eq!(crate::db_queries::get_sessions_between(&conn, prior_start, prior_end).unwrap().len(), 1);
        assert_eq!(crate::db_queries::get_sessions_between(&conn, start, end).unwrap().len(), 2);
    }
    // Public local-date query and totals use the host timezone, not a guessed offset.
    let (start, _) = local_date_bounds(&chrono::Local, "2026-10-09", "2026-10-09").unwrap();
    let isolated = rusqlite::Connection::open_in_memory().unwrap();
    crate::db::init_schema(&isolated).unwrap();
    crate::db_sessions::insert_session(&isolated,
        &(start - chrono::Duration::seconds(60)).to_rfc3339(),
        &(start + chrono::Duration::seconds(60)).to_rfc3339(), "Standing", 120).unwrap();
    for date in ["2026-10-08", "2026-10-09"] {
        assert_eq!(crate::db_queries::get_sessions_range(&isolated, date, date).unwrap().len(), 1);
        assert_eq!(crate::db_sessions::get_totals_for_date(&isolated, date).unwrap(), (0, 60));
    }
    assert!(crate::db_queries::get_sessions_range(&isolated, "invalid", "2026-10-09").is_err());
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};

    use crate::communication_profile::CommunicationProfile;
    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    /// A fixed instant, mid-morning, well away from any day boundary.
    fn base() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 1, 9, 0, 0).unwrap()
    }

    /// The local day `base()` falls on. Boundary cases live in
    /// `session_tests_clock.rs`; these tests only need "today" and "yesterday".
    fn today() -> NaiveDate {
        crate::session_daily::local_date_of(base())
    }

    fn yesterday() -> NaiveDate {
        today() - Duration::days(1)
    }

    fn comm_with(inactivity: bool, posture_balance: bool) -> CommunicationProfile {
        let mut comm = CommunicationProfile::default();
        comm.periodic_notifications.inactivity_enabled = inactivity;
        comm.periodic_notifications.posture_balance_enabled = posture_balance;
        comm
    }

    // ─── Daily Reset Tests ───────────────────────────────────────────────────

    #[test]
    fn check_daily_reset_resets_standing_seconds() {
        let mut m = SessionManager::new();
        m.state.standing_seconds = 3600;
        m.state.sitting_seconds = 2400;
        m.alert_fired = true;
        m.stand_alert_fired = true;
        m.last_reset_date = yesterday();
        m.last_reset_check = base() - Duration::seconds(120);
        let reset_happened = m.check_daily_reset_at(base(), today());
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
        m.last_reset_date = today();
        m.last_reset_check = base() - Duration::seconds(30);
        let reset_happened = m.check_daily_reset_at(base(), today());
        assert!(!reset_happened, "reset should not happen on same day");
        assert_eq!(m.state.sitting_seconds, 1000);
        assert_eq!(m.state.standing_seconds, 500);
    }

    #[test]
    fn check_daily_reset_throttled_every_60_seconds() {
        let mut m = SessionManager::new();
        m.state.sitting_seconds = 1000;
        m.last_reset_date = yesterday();
        m.last_reset_check = base() - Duration::seconds(30);
        let reset_happened = m.check_daily_reset_at(base(), today());
        assert!(!reset_happened, "reset should be throttled if < 60 seconds");
        assert_eq!(m.state.sitting_seconds, 1000);
    }

    #[test]
    fn check_daily_reset_idempotent_same_day() {
        let mut m = SessionManager::new();
        m.state.sitting_seconds = 500;
        m.last_reset_date = today();
        m.last_reset_check = base() - Duration::seconds(120);
        let first = m.check_daily_reset_at(base(), today());
        let second = m.check_daily_reset_at(base(), today());
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
        m.last_reset_date = yesterday();
        m.last_reset_check = base() - Duration::seconds(120);
        let _ = m.check_daily_reset_at(base(), today());
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
        let comm = comm_with(true, false);
        m.state.last_position_change_at = Some(base() - Duration::minutes(61));
        let events = m.check_notification_conditions_at(&comm, base());
        assert!(events.iter().any(|e| matches!(e, NotificationEvent::Inactivity)));
        assert!(m.notify_inactivity_fired);
    }

    #[test]
    fn check_notification_conditions_inactivity_not_disabled() {
        let mut m = SessionManager::new();
        let comm = comm_with(false, false);
        m.state.last_position_change_at = Some(base() - Duration::minutes(61));
        let events = m.check_notification_conditions_at(&comm, base());
        assert!(!events.iter().any(|e| matches!(e, NotificationEvent::Inactivity)));
        assert!(!m.notify_inactivity_fired);
    }

    #[test]
    fn check_notification_conditions_posture_balance_fires() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Sitting;
        let comm = comm_with(false, true);
        m.state.sitting_seconds = 22000;
        m.state.sitting_seconds_total = 22000;
        m.state.standing_seconds = 1000;
        let events = m.check_notification_conditions_at(&comm, base());
        assert!(events.iter().any(|e| matches!(e, NotificationEvent::PostureBalance)));
        assert!(m.notify_posture_balance_fired);
    }

    #[test]
    fn check_notification_conditions_posture_balance_not_disabled() {
        let mut m = SessionManager::new();
        let comm = comm_with(false, false);
        m.state.sitting_seconds = 3600;
        m.state.standing_seconds = 1000;
        let events = m.check_notification_conditions_at(&comm, base());
        assert!(!events.iter().any(|e| matches!(e, NotificationEvent::PostureBalance)));
    }

    #[test]
    fn check_notification_conditions_all_reset_on_daily_reset() {
        let mut m = SessionManager::new();
        let comm = CommunicationProfile::default();
        m.notify_inactivity_fired = true;
        m.notify_posture_balance_fired = true;
        m.praise_halfway_fired_today = true;
        m.standing_target_reached_fired = true;
        m.last_reset_date = yesterday();
        m.last_reset_check = base() - Duration::seconds(120);
        let _ = m.check_daily_reset_at(base(), today());
        assert!(!m.standing_target_reached_fired);
        let events = m.check_notification_conditions_at(&comm, base());
        assert_eq!(events.len(), 0, "no notifications should fire after reset");
    }

    #[test]
    fn standing_target_reached_fires_when_standing_exceeds_limit() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Standing;
        m.state.stand_limit_secs = 900; // 15 min
        m.state.standing_seconds = 900;
        let comm = CommunicationProfile::default();
        let events = m.check_notification_conditions_at(&comm, base());
        assert!(events.iter().any(|e| matches!(e, NotificationEvent::StandingTargetReached)));
        assert!(m.standing_target_reached_fired);
    }

    #[test]
    fn standing_target_reached_does_not_fire_twice() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Standing;
        m.state.stand_limit_secs = 900;
        m.state.standing_seconds = 1000;
        let comm = CommunicationProfile::default();
        let _ = m.check_notification_conditions_at(&comm, base());
        assert!(m.standing_target_reached_fired);
        let events = m.check_notification_conditions_at(&comm, base());
        assert!(!events.iter().any(|e| matches!(e, NotificationEvent::StandingTargetReached)));
    }

    #[test]
    fn standing_target_reached_resets_on_daily_reset() {
        let mut m = SessionManager::new();
        m.standing_target_reached_fired = true;
        m.last_reset_date = yesterday();
        m.last_reset_check = base() - Duration::seconds(120);
        let _ = m.check_daily_reset_at(base(), today());
        assert!(!m.standing_target_reached_fired);
    }
}
