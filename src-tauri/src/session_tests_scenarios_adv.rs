//! session_tests_scenarios_adv.rs — Advanced multi-cycle scenarios (5-8).
//!
//! Continuation of session_tests_scenarios.rs: restart, rapid changes,
//! user's exact bug, and daily reset.

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    const SIT_MM: i32 = 800;
    const STAND_MM: i32 = 1200;

    fn feed(m: &mut SessionManager, mm: i32, active: bool, count: usize) {
        for _ in 0..count {
            let _ = m.on_reading(mm, active);
            if let Some(ts) = m.state.last_accumulate_ts {
                m.state.last_accumulate_ts = Some(ts + Duration::seconds(1));
            }
        }
    }

    fn transition_to(m: &mut SessionManager, state: DeskState, mm: i32, active: bool) {
        feed(m, mm, active, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, state, "expected {:?}", state);
    }

    fn simulate_elapsed(m: &mut SessionManager, secs: i64) {
        let offset = Duration::seconds(secs);
        if let Some(ref mut ts) = m.state.sitting_started { *ts = *ts - offset; }
        if let Some(ref mut ts) = m.state.standing_bout_started { *ts = *ts - offset; }
        if let Some(ref mut ts) = m.state.standing_session_started { *ts = *ts - offset; }
        if let Some(ref mut ts) = m.state.break_started { *ts = *ts - offset; }
        m.state.last_tick_ts = Some(Utc::now());
    }

    // ─── Scenario 5: App restart mid-session ─────────────────────────────

    #[test]
    fn scenario_app_restart_preserves_daily_totals() {
        let mut m = SessionManager::new();
        let totals = crate::db_sessions::TodayTotals {
            sitting_secs: 1800,
            standing_secs: 600,
            position_changes: 3,
        };
        m.load_today_totals(&totals);

        assert_eq!(m.state.sitting_seconds, 1800);
        assert_eq!(m.state.standing_seconds, 600);
        assert_eq!(m.state.position_changes, 3);

        transition_to(&mut m, DeskState::Sitting, SIT_MM, true);
        simulate_elapsed(&mut m, 10 * 60);
        transition_to(&mut m, DeskState::Standing, STAND_MM, true);
        assert!(m.state.sitting_seconds >= 2350,
            "daily total should include DB + new: got {}", m.state.sitting_seconds);
        assert_eq!(m.state.position_changes, 4);
    }

    // ─── Scenario 6: Rapid sit/stand oscillation ─────────────────────────

    #[test]
    fn scenario_rapid_position_changes() {
        // Breaks are 30s each (< BREAK_MIN_SECS=60) → BreakCredit::None.
        let mut m = SessionManager::new();
        transition_to(&mut m, DeskState::Sitting, SIT_MM, true);
        simulate_elapsed(&mut m, 2 * 60);
        transition_to(&mut m, DeskState::Standing, STAND_MM, true);
        simulate_elapsed(&mut m, 30); // 30s break < 60s threshold → None
        transition_to(&mut m, DeskState::Sitting, SIT_MM, true);
        assert_eq!(m.state.last_break_credit, BreakCredit::None);
        simulate_elapsed(&mut m, 1 * 60);
        transition_to(&mut m, DeskState::Standing, STAND_MM, true);
        simulate_elapsed(&mut m, 30); // 30s break < 60s threshold → None
        transition_to(&mut m, DeskState::Sitting, SIT_MM, true);
        assert_eq!(m.state.last_break_credit, BreakCredit::None);
        assert_eq!(m.state.position_changes, 4);
        assert!(m.state.sitting_seconds >= 150 && m.state.sitting_seconds < 500);
        assert!(m.state.standing_seconds >= 50 && m.state.standing_seconds < 200);
    }

    // ─── Scenario 7: Stand 18min → Away → Sit (user's exact bug) ────────

    #[test]
    fn scenario_stand_18min_away_then_sit() {
        let mut m = SessionManager::new();
        transition_to(&mut m, DeskState::Sitting, SIT_MM, true);
        simulate_elapsed(&mut m, 20 * 60);

        transition_to(&mut m, DeskState::Standing, STAND_MM, true);
        let sitting_committed = m.state.sitting_seconds;
        assert!(sitting_committed >= 1180);
        simulate_elapsed(&mut m, 18 * 60);

        let mut last = ReadingResult::default();
        for _ in 0..DEBOUNCE_COUNT { last = m.on_reading(STAND_MM, false); }
        assert_eq!(m.state.state, DeskState::Away);
        assert!(m.state.standing_seconds >= 1070,
            "standing must be committed: got {}", m.state.standing_seconds);

        simulate_elapsed(&mut m, 8 * 60);
        for _ in 0..DEBOUNCE_COUNT { last = m.on_reading(SIT_MM, true); }
        assert_eq!(m.state.state, DeskState::Sitting);
        assert_eq!(m.state.last_break_credit, BreakCredit::Full);
        assert_eq!(m.state.sitting_seconds, 0);
        assert!(m.state.standing_seconds >= 1070,
            "standing_seconds must NOT be lost: got {}", m.state.standing_seconds);
        assert!(last.completed_session.is_some(),
            "Away→Sit must create CompletedSession");
    }

    // ─── Scenario 8: Daily reset mid-use ─────────────────────────────────

    #[test]
    fn scenario_daily_reset_clears_everything() {
        let mut m = SessionManager::new();
        transition_to(&mut m, DeskState::Sitting, SIT_MM, true);
        m.state.sitting_seconds = 3600;
        m.state.standing_seconds = 1200;
        m.state.position_changes = 5;
        m.state.daily_score = 15.0;
        m.state.continuous_computer_secs = 5400;
        m.state.longest_computer_session_secs = 5400;

        m.last_reset_date = (Utc::now() - Duration::days(1)).date_naive();
        m.last_reset_check = Utc::now() - Duration::seconds(120);
        assert!(m.check_daily_reset());
        assert_eq!(m.state.sitting_seconds, 0);
        assert_eq!(m.state.standing_seconds, 0);
        assert_eq!(m.state.position_changes, 0);
        assert_eq!(m.state.daily_score, 0.0);
        assert_eq!(m.state.continuous_computer_secs, 0);
        assert_eq!(m.state.longest_computer_session_secs, 0);
    }
}
