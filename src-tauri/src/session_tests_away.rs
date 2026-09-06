//! session_tests_away.rs — Away state detection and accumulation tests (E002-T04).
//!
//! The accumulation tests already drove `accumulate_ongoing` from an explicit
//! instant; E020-T01 extends that to the readings themselves, so no test here
//! reads the system clock.

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Duration, TimeZone, Utc};

    use crate::communication_profile::CommunicationProfile;
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

    // ─── State Detection ────────────────────────────────────────────────────

    #[test]
    fn low_desk_inactive_produces_away() {
        let mut m = SessionManager::new();
        advance_ticks(&mut m, 800, false, base(), DEBOUNCE_COUNT as i64);
        assert_eq!(m.state.state, DeskState::Away, "low desk + inactive must be Away");
    }

    #[test]
    fn high_desk_inactive_produces_away() {
        let mut m = SessionManager::new();
        advance_ticks(&mut m, 1200, false, base(), DEBOUNCE_COUNT as i64);
        assert_eq!(m.state.state, DeskState::Away, "high desk + inactive must be Away");
    }

    #[test]
    fn low_desk_active_produces_sitting() {
        let mut m = SessionManager::new();
        advance_ticks(&mut m, 800, true, base(), DEBOUNCE_COUNT as i64);
        assert_eq!(m.state.state, DeskState::Sitting);
    }

    #[test]
    fn high_desk_active_produces_standing() {
        let mut m = SessionManager::new();
        advance_ticks(&mut m, 1200, true, base(), DEBOUNCE_COUNT as i64);
        assert_eq!(m.state.state, DeskState::Standing);
    }

    // ─── Away Bout Accumulation ─────────────────────────────────────────────

    #[test]
    fn away_bout_secs_increments_during_away() {
        let mut m = SessionManager::new();
        advance_ticks(&mut m, 800, false, base(), DEBOUNCE_COUNT as i64);
        assert_eq!(m.state.state, DeskState::Away);

        let before = m.state.away_bout_secs;
        let start = base() + Duration::minutes(1);
        for i in 1..=3 {
            let t = start + Duration::seconds(i);
            m.state.last_accumulate_ts = Some(t - Duration::seconds(1));
            m.accumulate_ongoing(t);
        }
        assert!(
            m.state.away_bout_secs > before,
            "away_bout_secs should grow: {} -> {}",
            before, m.state.away_bout_secs
        );
    }

    #[test]
    fn away_bout_resets_on_return_to_sitting() {
        let mut m = SessionManager::new();
        let t = advance_ticks(&mut m, 800, false, base(), DEBOUNCE_COUNT as i64);
        let t = advance_ticks(&mut m, 800, false, t, 10);
        assert!(m.state.away_bout_secs > 0);
        advance_ticks(&mut m, 800, true, t, DEBOUNCE_COUNT as i64);
        assert_eq!(m.state.state, DeskState::Sitting);
        assert_eq!(m.state.away_bout_secs, 0, "away_bout must reset on return");
    }

    // ─── 5-Min Triggers ────────────────────────────────────────────────────

    #[test]
    fn five_min_away_counts_as_position_change() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Away;
        m.state.position_changes = 0;
        m.state.continuous_computer_secs = 4500;
        m.state.last_accumulate_ts = None;

        for i in 0..300 {
            let now = base() + Duration::seconds(i);
            m.state.last_accumulate_ts =
                if i == 0 { None } else { Some(now - Duration::seconds(1)) };
            m.accumulate_ongoing(now);
        }

        assert_eq!(m.state.away_bout_secs, 300);
        assert_eq!(m.state.position_changes, 1, "away reset IS a posture change");
    }

    #[test]
    fn five_min_away_resets_continuous_computer_secs() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Away;
        m.state.continuous_computer_secs = 4500;
        m.state.last_accumulate_ts = None;

        for i in 0..300 {
            let now = base() + Duration::seconds(i);
            m.state.last_accumulate_ts =
                if i == 0 { None } else { Some(now - Duration::seconds(1)) };
            m.accumulate_ongoing(now);
        }

        assert_eq!(m.state.continuous_computer_secs, 0, "5 min away resets computer time");
    }

    #[test]
    fn longest_session_preserved_after_away_reset() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Sitting;
        m.state.continuous_computer_secs = 4500;
        m.state.longest_computer_session_secs = 4500;
        m.state.last_accumulate_ts = None;

        let now = base();
        m.accumulate_ongoing(now);
        assert_eq!(m.state.longest_computer_session_secs, 4501);

        m.state.state = DeskState::Away;
        for i in 0..300 {
            let t = now + Duration::seconds(i + 1);
            m.state.last_accumulate_ts = Some(t - Duration::seconds(1));
            m.accumulate_ongoing(t);
        }

        assert_eq!(m.state.continuous_computer_secs, 0);
        assert_eq!(m.state.longest_computer_session_secs, 4501, "daily max never decreases");
    }

    // ─── Notification Suppression ───────────────────────────────────────────

    #[test]
    fn notifications_suppressed_during_away() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Away;
        m.state.sitting_seconds = 5000;
        m.state.standing_seconds = 100;
        m.state.last_position_change_at = Some(base() - Duration::minutes(120));

        let mut comm = CommunicationProfile::default();
        comm.periodic_notifications.inactivity_enabled = true;
        comm.periodic_notifications.posture_balance_enabled = true;

        let events = m.check_notification_conditions_at(&comm, base());
        assert!(events.is_empty(), "no notifications during Away, got {:?}", events);
    }

    #[test]
    fn notifications_fire_after_returning_from_away() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Sitting;
        m.state.sitting_seconds = 22000;
        m.state.sitting_seconds_total = 22000;
        m.state.standing_seconds = 100;
        m.state.last_position_change_at = Some(base() - Duration::minutes(120));

        let mut comm = CommunicationProfile::default();
        comm.periodic_notifications.inactivity_enabled = true;
        comm.periodic_notifications.posture_balance_enabled = true;

        let events = m.check_notification_conditions_at(&comm, base());
        assert!(events.len() >= 2, "should fire when sitting, got {:?}", events);
    }

    // ─── Configurable Computer Break Reset ──────────────────────────────

    #[test]
    fn computer_timer_resets_at_configurable_threshold() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Away;
        m.state.continuous_computer_secs = 4500;
        m.state.computer_break_reset_secs = 5; // 5s instead of default 300s
        m.state.last_accumulate_ts = None;

        for i in 0..5 {
            let now = base() + Duration::seconds(i);
            m.state.last_accumulate_ts =
                if i == 0 { None } else { Some(now - Duration::seconds(1)) };
            m.accumulate_ongoing(now);
        }

        assert_eq!(
            m.state.continuous_computer_secs, 0,
            "computer timer should reset at configurable threshold (5s)"
        );
        assert_eq!(
            m.state.position_changes, 1,
            "away reset IS a posture change"
        );
    }

    #[test]
    fn computer_timer_does_not_reset_before_threshold() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Away;
        m.state.continuous_computer_secs = 4500;
        m.state.computer_break_reset_secs = 10;
        m.state.last_accumulate_ts = None;

        for i in 0..5 {
            let now = base() + Duration::seconds(i);
            m.state.last_accumulate_ts =
                if i == 0 { None } else { Some(now - Duration::seconds(1)) };
            m.accumulate_ongoing(now);
        }

        assert_eq!(
            m.state.continuous_computer_secs, 4500,
            "computer timer must not reset before reaching threshold"
        );
    }
}
