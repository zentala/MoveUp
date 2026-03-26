//! session_tests_away_transitions.rs — Away state transition + credit tests (E002-T04).

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::session_manager::SessionManager;
    use crate::session_types::*;

    fn advance_ticks(m: &mut SessionManager, mm: i32, active: bool, count: usize) {
        for _ in 0..count {
            let _ = m.on_reading(mm, active);
        }
    }

    // ─── Away → Sitting = New Session ───────────────────────────────────────

    #[test]
    fn away_to_sitting_full_credit() {
        let mut m = SessionManager::new();
        advance_ticks(&mut m, 800, true, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Sitting);
        m.state.sitting_seconds = 2000;
        m.state.sitting_started = Some(Utc::now() - chrono::Duration::seconds(100));

        advance_ticks(&mut m, 800, false, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Away);

        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(700));
        advance_ticks(&mut m, 800, true, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Sitting);

        assert_eq!(m.state.sitting_seconds, 0, ">=10 min away = full reset");
        assert!(m.state.sitting_started.is_some(), "new session started");
    }

    #[test]
    fn away_to_sitting_partial_credit() {
        let mut m = SessionManager::new();
        advance_ticks(&mut m, 800, true, DEBOUNCE_COUNT as usize);
        m.state.sitting_seconds = 2000;
        m.state.sitting_started = Some(Utc::now() - chrono::Duration::seconds(100));

        advance_ticks(&mut m, 800, false, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Away);

        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(420));
        advance_ticks(&mut m, 800, true, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Sitting);
        assert_eq!(m.state.last_break_credit, BreakCredit::Partial, "7 min = partial");
    }

    #[test]
    fn away_to_sitting_no_credit() {
        let mut m = SessionManager::new();
        advance_ticks(&mut m, 800, true, DEBOUNCE_COUNT as usize);
        m.state.sitting_seconds = 2000;
        m.state.sitting_started = Some(Utc::now() - chrono::Duration::seconds(100));

        advance_ticks(&mut m, 800, false, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Away);

        // 2 min away (< 5 min = no credit)
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(120));
        advance_ticks(&mut m, 800, true, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Sitting);
        assert_eq!(m.state.last_break_credit, BreakCredit::None, "<5 min = no credit");
    }

    #[test]
    fn away_to_standing_resets_away_bout() {
        let mut m = SessionManager::new();
        advance_ticks(&mut m, 800, true, DEBOUNCE_COUNT as usize);
        m.state.sitting_started = Some(Utc::now());

        advance_ticks(&mut m, 800, false, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Away);
        m.state.away_bout_secs = 120;

        advance_ticks(&mut m, 1200, true, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Standing);
        assert_eq!(m.state.away_bout_secs, 0, "away_bout must reset on standing");
    }

    #[test]
    fn sitting_to_away_creates_completed_session() {
        let mut m = SessionManager::new();
        advance_ticks(&mut m, 800, true, DEBOUNCE_COUNT as usize);
        m.state.sitting_started = Some(Utc::now() - chrono::Duration::seconds(300));

        let mut last = ReadingResult {
            state_change: None,
            completed_session: None,
            break_credit: None,
        };
        for _ in 0..DEBOUNCE_COUNT {
            last = m.on_reading(800, false);
        }
        assert_eq!(m.state.state, DeskState::Away);
        assert!(
            last.completed_session.is_some(),
            "Sitting→Away must create CompletedSession for timeline"
        );
    }

    /// Regression test: Standing + inactive MUST transition to Away.
    /// Bug report: user was standing, walked away, app stayed in Standing.
    /// Verifies: (1) state transitions to Away, (2) break_started preserved,
    /// (3) state_change event is emitted, (4) break credit applied on return.
    #[test]
    fn standing_to_away_on_inactivity() {
        let mut m = SessionManager::new();

        // 1. Start Sitting
        advance_ticks(&mut m, 800, true, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Sitting);
        m.state.sitting_started = Some(Utc::now() - chrono::Duration::seconds(600));
        m.state.sitting_seconds = 600;
        m.state.sitting_seconds_total = 600;

        // 2. Stand up (active) — should transition to Standing
        advance_ticks(&mut m, 1200, true, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Standing, "should be Standing when desk high + active");
        assert!(m.state.break_started.is_some(), "break_started must be set on Sitting→Standing");
        let break_start = m.state.break_started.unwrap();

        // 3. Walk away (inactive, desk still high) — MUST transition to Away
        let mut last = ReadingResult {
            state_change: None,
            completed_session: None,
            break_credit: None,
        };
        for _ in 0..DEBOUNCE_COUNT {
            last = m.on_reading(1200, false);  // high desk + inactive
        }
        assert_eq!(
            m.state.state, DeskState::Away,
            "Standing + inactive MUST transition to Away (not stay Standing)"
        );
        assert!(
            last.state_change.is_some(),
            "state_change event must be emitted for Standing→Away"
        );
        assert_eq!(
            last.state_change.as_ref().unwrap().state,
            DeskState::Away,
            "emitted state must be Away"
        );
        // break_started must be preserved (break continues through Away)
        assert_eq!(
            m.state.break_started,
            Some(break_start),
            "break_started must be preserved on Standing→Away"
        );

        // 4. Stay away for 10+ minutes — simulate with fake break_started
        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(700));

        // 5. Return and sit down — break credit should apply
        advance_ticks(&mut m, 800, true, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Sitting);
        assert_eq!(
            m.state.last_break_credit, BreakCredit::Full,
            ">=10 min break (standing+away) = full credit"
        );
        assert_eq!(m.state.sitting_seconds, 0, "full credit resets sitting to 0");
    }

    // Sleep inflation tests moved to session_tests_sleep.rs

    #[test]
    fn standing_away_standing_sitting_only_counts_standing() {
        let mut m = SessionManager::new();
        // Start sitting
        advance_ticks(&mut m, 800, true, DEBOUNCE_COUNT as usize);
        m.state.sitting_started = Some(Utc::now());

        // Stand for ~5 min (faked)
        advance_ticks(&mut m, 1200, true, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Standing);
        let t1 = Utc::now() - chrono::Duration::seconds(300);
        m.state.break_started = Some(t1);
        m.state.standing_bout_started = Some(t1);

        // Go away for ~10 min
        advance_ticks(&mut m, 1200, false, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Away);
        // standing_seconds should now have ~300s from the standing bout
        let after_first_stand = m.state.standing_seconds;
        assert!(after_first_stand >= 299, "got {}", after_first_stand);

        // Come back standing for ~3 min
        advance_ticks(&mut m, 1200, true, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Standing);
        let t3 = Utc::now() - chrono::Duration::seconds(180);
        m.state.standing_bout_started = Some(t3);

        // Sit down
        advance_ticks(&mut m, 800, true, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Sitting);

        // Total standing should be ~300 + ~180 = ~480, NOT 300+600+180
        let total = m.state.standing_seconds;
        assert!(
            total >= 478 && total <= 482,
            "expected ~480 (300+180), got {} — away time must not count",
            total
        );
    }

    #[test]
    fn away_to_sitting_creates_completed_session() {
        let mut m = SessionManager::new();
        advance_ticks(&mut m, 800, true, DEBOUNCE_COUNT as usize);
        m.state.sitting_started = Some(Utc::now());

        advance_ticks(&mut m, 800, false, DEBOUNCE_COUNT as usize);
        assert_eq!(m.state.state, DeskState::Away);

        m.state.break_started = Some(Utc::now() - chrono::Duration::seconds(600));
        let mut last = ReadingResult {
            state_change: None,
            completed_session: None,
            break_credit: None,
        };
        for _ in 0..DEBOUNCE_COUNT {
            last = m.on_reading(800, true);
        }
        assert_eq!(m.state.state, DeskState::Sitting);
        assert!(
            last.completed_session.is_some(),
            "Away→Sitting must create CompletedSession for timeline"
        );
    }
}
