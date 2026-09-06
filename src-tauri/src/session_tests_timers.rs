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
    fn e015_t042_04_restart_from_db_seeds_credited_counter() {
        let mut m = SessionManager::new();
        m.load_today_totals(&crate::db_sessions::TodayTotals::from_secs(1800, 600));
        assert_eq!(m.state.sitting_seconds, 1800);
        assert_eq!(m.snapshot().limit_used_secs, 1800);
    }

    #[test]
    fn t042_05_restart_from_db_seeds_standing_break_zero() {
        let mut m = SessionManager::new();
        m.load_today_totals(&crate::db_sessions::TodayTotals::from_secs(1200, 900));
        assert_eq!(m.state.standing_seconds, 900);
        assert_eq!(m.state.break_seconds, 0);
    }

    // ─── State Transitions (tests 6–11) ──────────────────────────────────

    #[test]
    fn e015_t042_06_sitting_to_standing_keeps_credited_secs_starts_break() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_seconds = 1200;
        m.state.sitting_started = Some(Utc::now());
        transition_to_standing(&mut m);
        // Standing up does not zero the counter — only break credit reduces it,
        // and that is applied when the user sits back down.
        assert!(
            m.state.sitting_seconds >= 1200,
            "expected >= 1200, got {}",
            m.state.sitting_seconds
        );
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
    fn e015_t042_08_short_standing_no_credit_keeps_sitting_secs() {
        // Break must be < BREAK_MIN_SECS (60s) to get None. Use 30s.
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_seconds = 1500;
        m.state.sitting_started = Some(Utc::now());
        transition_to_standing(&mut m);
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(30));
        transition_to_sitting(&mut m);
        // A 30 s break is under BREAK_MIN_SECS: no credit, and nothing resets.
        assert!(
            m.state.sitting_seconds >= 1500,
            "expected >= 1500, got {}",
            m.state.sitting_seconds
        );
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
    fn t042_10b_standing_away_sitting_only_counts_standing_time() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_started = Some(Utc::now());
        let before = m.state.standing_seconds;
        transition_to_standing(&mut m);
        // Fake: user stood for 5 min, then went away
        m.state.standing_bout_started = Some(Utc::now() - chrono::Duration::seconds(5 * 60));
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(5 * 60));
        transition_to_away(&mut m);
        // After Standing→Away: standing time (5 min) should be committed
        assert!(
            m.state.standing_seconds >= before + 299,
            "standing committed on Standing→Away: got {}",
            m.state.standing_seconds
        );
        let after_away_entry = m.state.standing_seconds;
        // Fake: user was away for another 10 min
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(15 * 60));
        transition_to_sitting(&mut m);
        // Away time should NOT increase standing_seconds
        assert_eq!(
            m.state.standing_seconds, after_away_entry,
            "away time must not count as standing"
        );
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
        let t = Utc::now() - chrono::Duration::seconds(600);
        m.state.break_started = Some(t);
        m.state.standing_bout_started = Some(t);
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
