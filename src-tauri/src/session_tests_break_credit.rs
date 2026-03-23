//! session_tests_break_credit.rs — Tests for break_credit field in ReadingResult.

#[cfg(test)]
mod break_credit_tests {
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

    #[test]
    fn break_credit_full_after_long_break() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        transition_to_standing(&mut m);

        m.state.break_started =
            Some(Utc::now() - chrono::Duration::seconds(12 * 60));

        let mut result = ReadingResult {
            state_change: None,
            completed_session: None,
            break_credit: None,
        };
        for _ in 0..DEBOUNCE_COUNT {
            result = m.on_reading(800, true);
        }
        let (credit, dur) = result.break_credit
            .expect("should carry break_credit after Standing→Sitting with >=10min");
        assert_eq!(credit, BreakCredit::Full);
        assert!(dur >= 12 * 60, "duration should be >= 720s, got {}", dur);
    }

    #[test]
    fn break_credit_none_for_short_break() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        transition_to_standing(&mut m);

        m.state.break_started =
            Some(Utc::now() - chrono::Duration::seconds(2 * 60));

        let mut result = ReadingResult {
            state_change: None,
            completed_session: None,
            break_credit: None,
        };
        for _ in 0..DEBOUNCE_COUNT {
            result = m.on_reading(800, true);
        }
        assert!(
            result.break_credit.is_none(),
            "Short break (<5min) should NOT set break_credit"
        );
    }

    #[test]
    fn break_credit_partial_for_medium_break() {
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        transition_to_standing(&mut m);

        m.state.break_started =
            Some(Utc::now() - chrono::Duration::seconds(7 * 60));

        let mut result = ReadingResult {
            state_change: None,
            completed_session: None,
            break_credit: None,
        };
        for _ in 0..DEBOUNCE_COUNT {
            result = m.on_reading(800, true);
        }
        let (credit, dur) = result.break_credit
            .expect("should carry break_credit for 7min break");
        assert_eq!(credit, BreakCredit::Partial);
        assert!(dur >= 7 * 60, "duration should be >= 420s, got {}", dur);
    }
}
