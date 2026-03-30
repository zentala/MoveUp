//! session_tests_away.rs — Away state detection and accumulation tests (E002-T04).

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::communication_profile::CommunicationProfile;
    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    fn advance_ticks(m: &mut SessionManager, mm: i32, active: bool, count: usize) {
        for _ in 0..count {
            let _ = m.on_reading(mm, active);
        }
    }

    // ─── State Detection ────────────────────────────────────────────────────

    #[test]
    fn low_desk_inactive_produces_away() {
        let mut m = SessionManager::new();
        advance_ticks(&mut m, 800, false, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Away, "low desk + inactive must be Away");
    }

    #[test]
    fn high_desk_inactive_produces_away() {
        let mut m = SessionManager::new();
        advance_ticks(&mut m, 1200, false, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Away, "high desk + inactive must be Away");
    }

    #[test]
    fn low_desk_active_produces_sitting() {
        let mut m = SessionManager::new();
        advance_ticks(&mut m, 800, true, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Sitting);
    }

    #[test]
    fn high_desk_active_produces_standing() {
        let mut m = SessionManager::new();
        advance_ticks(&mut m, 1200, true, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Standing);
    }

    // ─── Away Bout Accumulation ─────────────────────────────────────────────

    #[test]
    fn away_bout_secs_increments_during_away() {
        let mut m = SessionManager::new();
        advance_ticks(&mut m, 800, false, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Away);

        let before = m.state.away_bout_secs;
        let base = Utc::now();
        for i in 1..=3 {
            let t = base + chrono::Duration::seconds(i);
            m.state.last_accumulate_ts = Some(t - chrono::Duration::seconds(1));
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
        advance_ticks(&mut m, 800, false, DEBOUNCE_COUNT as usize);
        advance_ticks(&mut m, 800, false, 10);
        assert!(m.state.away_bout_secs > 0);
        advance_ticks(&mut m, 800, true, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Sitting);
        assert_eq!(m.state.away_bout_secs, 0, "away_bout must reset on return");
    }

    // ─── 5-Min Triggers ────────────────────────────────────────────────────

    #[test]
    fn five_min_away_triggers_position_change() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Away;
        m.state.position_changes = 0;
        m.state.last_accumulate_ts = None;

        for i in 0..300 {
            let now = Utc::now() + chrono::Duration::seconds(i);
            m.state.last_accumulate_ts =
                if i == 0 { None } else { Some(now - chrono::Duration::seconds(1)) };
            m.accumulate_ongoing(now);
        }

        assert_eq!(m.state.away_bout_secs, 300);
        assert_eq!(m.state.position_changes, 1, "5 min away = 1 position change");
    }

    #[test]
    fn five_min_away_resets_continuous_computer_secs() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Away;
        m.state.continuous_computer_secs = 4500;
        m.state.last_accumulate_ts = None;

        for i in 0..300 {
            let now = Utc::now() + chrono::Duration::seconds(i);
            m.state.last_accumulate_ts =
                if i == 0 { None } else { Some(now - chrono::Duration::seconds(1)) };
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

        let now = Utc::now();
        m.accumulate_ongoing(now);
        assert_eq!(m.state.longest_computer_session_secs, 4501);

        m.state.state = DeskState::Away;
        for i in 0..300 {
            let t = now + chrono::Duration::seconds(i + 1);
            m.state.last_accumulate_ts = Some(t - chrono::Duration::seconds(1));
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
        m.state.last_position_change_at =
            Some(Utc::now() - chrono::Duration::minutes(120));

        let mut comm = CommunicationProfile::default();
        comm.periodic_notifications.inactivity_enabled = true;
        comm.periodic_notifications.posture_balance_enabled = true;

        let events = m.check_notification_conditions(&comm);
        assert!(events.is_empty(), "no notifications during Away, got {:?}", events);
    }

    #[test]
    fn notifications_fire_after_returning_from_away() {
        let mut m = SessionManager::new();
        m.state.state = DeskState::Sitting;
        m.state.sitting_seconds = 22000;
        m.state.sitting_seconds_total = 22000;
        m.state.standing_seconds = 100;
        m.state.last_position_change_at =
            Some(Utc::now() - chrono::Duration::minutes(120));

        let mut comm = CommunicationProfile::default();
        comm.periodic_notifications.inactivity_enabled = true;
        comm.periodic_notifications.posture_balance_enabled = true;

        let events = m.check_notification_conditions(&comm);
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
            let now = Utc::now() + chrono::Duration::seconds(i);
            m.state.last_accumulate_ts =
                if i == 0 { None } else { Some(now - chrono::Duration::seconds(1)) };
            m.accumulate_ongoing(now);
        }

        assert_eq!(
            m.state.continuous_computer_secs, 0,
            "computer timer should reset at configurable threshold (5s)"
        );
        assert_eq!(
            m.state.position_changes, 1,
            "position change should fire at configurable threshold"
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
            let now = Utc::now() + chrono::Duration::seconds(i);
            m.state.last_accumulate_ts =
                if i == 0 { None } else { Some(now - chrono::Duration::seconds(1)) };
            m.accumulate_ongoing(now);
        }

        assert_eq!(
            m.state.continuous_computer_secs, 4500,
            "computer timer must not reset before reaching threshold"
        );
    }
}
