//! session_tests_timers.rs — T042: Initialization and state transition tests.

#[cfg(test)]
mod timer_tests {
    use chrono::Utc;

    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    fn transition_to_sitting(m: &mut SessionManager) {
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(800, true);
        }
        assert_eq!(m.state.state, DeskState::Sitting);
    }

    fn transition_to_standing(m: &mut SessionManager) {
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(1200, true);
        }
        assert_eq!(m.state.state, DeskState::Standing);
    }

    fn transition_to_away(m: &mut SessionManager) {
        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(1200, false);
        }
        assert_eq!(m.state.state, DeskState::Away);
    }

    // ─── Initialization (tests 1–5) ──────────────────────────────────────

    #[test]
    fn t042_01_start_standing_sets_break_started() {
        let mut m = SessionManager::new();
        transition_to_standing(&mut m);
        assert!(m.state.break_started.is_some());
    }

    #[test]
    fn t042_02_start_standing_break_seconds_grows() {
        let mut m = SessionManager::new();
        transition_to_standing(&mut m);
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(60));
        let live = m.get_live_break_seconds(Utc::now());
        assert!(live >= 59 && live <= 61, "expected ~60, got {}", live);
    }

    #[test]
    fn t042_03_start_sitting_sets_sitting_started() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        assert!(m.state.sitting_started.is_some());
    }

    #[test]
    fn t042_04_restart_from_db_seeds_sitting_session_zero() {
        let mut m = SessionManager::new();
        m.load_today_totals(1800, 600);
        assert_eq!(m.state.sitting_seconds, 1800);
        assert_eq!(m.state.current_session_secs, 0);
    }

    #[test]
    fn t042_05_restart_from_db_seeds_standing_break_zero() {
        let mut m = SessionManager::new();
        m.load_today_totals(1200, 900);
        assert_eq!(m.state.standing_seconds, 900);
        assert_eq!(m.state.break_seconds, 0);
    }

    // ─── State Transitions (tests 6–11) ──────────────────────────────────

    #[test]
    fn t042_06_sitting_to_standing_resets_session_starts_break() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_seconds = 1200;
        m.state.current_session_secs = 1200;
        m.state.sitting_started = Some(Utc::now());
        transition_to_standing(&mut m);
        assert_eq!(m.state.current_session_secs, 0);
        assert!(m.state.break_started.is_some());
    }

    #[test]
    fn t042_07_sitting_to_standing_creates_completed_session() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_started = Some(Utc::now() - chrono::Duration::seconds(300));
        let mut last = ReadingResult { state_change: None, completed_session: None, break_credit: None };
        for _ in 0..DEBOUNCE_COUNT {
            last = m.on_reading(1200, true);
        }
        let session = last.completed_session.expect("should produce CompletedSession");
        assert!(session.duration_secs >= 299 && session.duration_secs <= 301);
    }

    #[test]
    fn t042_08_short_standing_no_credit_session_zero() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_seconds = 1500;
        m.state.sitting_started = Some(Utc::now());
        transition_to_standing(&mut m);
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(3 * 60));
        transition_to_sitting(&mut m);
        assert_eq!(m.state.current_session_secs, 0);
        assert_eq!(m.state.last_break_credit, BreakCredit::None);
    }

    #[test]
    fn t042_09_standing_to_away_break_continues() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_started = Some(Utc::now());
        transition_to_standing(&mut m);
        let break_start = m.state.break_started;
        transition_to_away(&mut m);
        assert_eq!(m.state.break_started, break_start, "timestamp must not change");
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(60));
        let live = m.get_live_break_seconds(Utc::now());
        assert!(live >= 59 && live <= 61, "break should grow in Away, got {}", live);
    }

    #[test]
    fn t042_10_away_to_sitting_applies_credit() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_seconds = 2000;
        m.state.sitting_started = Some(Utc::now());
        transition_to_away(&mut m);
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(7 * 60));
        transition_to_sitting(&mut m);
        assert!(m.state.sitting_started.is_some());
        assert_eq!(m.state.last_break_credit, BreakCredit::Partial);
    }

    #[test]
    fn t042_10b_standing_away_sitting_accumulates_standing() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_started = Some(Utc::now());
        let before = m.state.standing_seconds;
        transition_to_standing(&mut m);
        transition_to_away(&mut m);
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(10 * 60));
        transition_to_sitting(&mut m);
        assert!(m.state.standing_seconds >= before + 599, "got {}", m.state.standing_seconds);
    }

    #[test]
    fn t042_11_away_to_standing_starts_break() {
        let mut m = SessionManager::new();
        assert_eq!(m.state.state, DeskState::Away);
        transition_to_standing(&mut m);
        assert!(m.state.break_started.is_some());
        assert_eq!(m.state.break_seconds, 0);
    }

    #[test]
    fn t042_12b_standing_to_sitting_produces_completed_session() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_started = Some(Utc::now());
        transition_to_standing(&mut m);
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(600));
        let mut last = ReadingResult { state_change: None, completed_session: None, break_credit: None };
        for _ in 0..DEBOUNCE_COUNT {
            last = m.on_reading(800, true);
        }
        let session = last.completed_session.expect("standing->sitting should produce CompletedSession");
        assert!(session.duration_secs >= 599 && session.duration_secs <= 601,
            "expected ~600, got {}", session.duration_secs);
    }

    #[test]
    fn t042_12c_away_to_sitting_produces_completed_session() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_started = Some(Utc::now());
        transition_to_standing(&mut m);
        transition_to_away(&mut m);
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(900));
        let mut last = ReadingResult { state_change: None, completed_session: None, break_credit: None };
        for _ in 0..DEBOUNCE_COUNT {
            last = m.on_reading(800, true);
        }
        let session = last.completed_session.expect("away->sitting should produce CompletedSession");
        assert!(session.duration_secs >= 899 && session.duration_secs <= 901,
            "expected ~900, got {}", session.duration_secs);
    }
}
