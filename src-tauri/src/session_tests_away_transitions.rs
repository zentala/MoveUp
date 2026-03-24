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
