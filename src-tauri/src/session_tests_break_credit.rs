//! session_tests_break_credit.rs — Tests for proportional break credit.

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
        // 12 min break with multiplier 2.0 = 24 min credit.
        // If sitting was < 24 min, should be Full reset.
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_seconds = 2400; // 40 min sitting
        transition_to_standing(&mut m);

        m.state.break_started =
            Some(Utc::now() - chrono::Duration::seconds(20 * 60)); // 20 min break

        let mut result = ReadingResult {
            state_change: None,
            completed_session: None,
            break_credit: None,
        };
        for _ in 0..DEBOUNCE_COUNT {
            result = m.on_reading(800, true);
        }
        let (credit, dur) = result.break_credit
            .expect("should carry break_credit after long break");
        // 20 min break * 2.0 = 40 min credit >= 40 min sitting → Full
        assert_eq!(credit, BreakCredit::Full);
        assert!(dur >= 20 * 60, "duration should be >= 1200s, got {}", dur);
        assert_eq!(m.state.sitting_seconds, 0, "sitting should be reset to 0");
    }

    #[test]
    fn break_credit_none_for_very_short_break() {
        // Break < 1 min (BREAK_MIN_SECS) → no credit
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_seconds = 1200;
        transition_to_standing(&mut m);

        m.state.break_started =
            Some(Utc::now() - chrono::Duration::seconds(30)); // 30 sec break

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
            "Break < 1 min should NOT set break_credit"
        );
    }

    #[test]
    fn break_credit_partial_proportional() {
        // 5 min break with multiplier 2.0 = 10 min (600s) credit.
        // 40 min sitting - 10 min credit = 30 min remaining → Partial
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_seconds = 2400; // 40 min sitting
        transition_to_standing(&mut m);

        m.state.break_started =
            Some(Utc::now() - chrono::Duration::seconds(5 * 60)); // 5 min break

        let mut result = ReadingResult {
            state_change: None,
            completed_session: None,
            break_credit: None,
        };
        for _ in 0..DEBOUNCE_COUNT {
            result = m.on_reading(800, true);
        }
        let (credit, dur) = result.break_credit
            .expect("should carry break_credit for 5 min break");
        assert_eq!(credit, BreakCredit::Partial);
        assert!(dur >= 5 * 60, "duration should be >= 300s, got {}", dur);
        // 2400 - (300 * 2.0) = 2400 - 600 = 1800
        assert_eq!(m.state.sitting_seconds, 1800, "sitting should be 1800s (30 min)");
    }

    #[test]
    fn break_credit_10min_cancels_20min_sitting() {
        // 10 min break * 2.0 = 20 min credit
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_seconds = 2400; // 40 min
        transition_to_standing(&mut m);

        m.state.break_started =
            Some(Utc::now() - chrono::Duration::seconds(10 * 60));

        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(800, true);
        }
        // 2400 - (600 * 2.0) = 2400 - 1200 = 1200 (20 min)
        assert_eq!(m.state.sitting_seconds, 1200);
    }

    #[test]
    fn break_credit_2h_away_resets_fully() {
        // 2 hour break * 2.0 = 4 hours credit → always Full reset
        let mut m = SessionManager::new();
        transition_to_sitting(&mut m);
        m.state.sitting_seconds = 2400; // 40 min
        transition_to_standing(&mut m);

        m.state.break_started =
            Some(Utc::now() - chrono::Duration::seconds(2 * 60 * 60)); // 2h

        for _ in 0..DEBOUNCE_COUNT {
            let _ = m.on_reading(800, true);
        }
        assert_eq!(m.state.sitting_seconds, 0, "2h break should fully reset sitting");
    }

}
