//! session_tests_props.rs — Property-based tests for session state machine invariants.

#[cfg(test)]
mod property_tests {
    use crate::session_manager::SessionManager;
    use crate::session_types::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_sitting_seconds_never_negative(
            readings in prop::collection::vec(750u32..1100u32, 10..100)
        ) {
            let mut m = SessionManager::new();
            for mm in readings {
                let _ = m.on_reading(mm as i32, true);
                prop_assert!(
                    m.state.sitting_seconds >= 0,
                    "sitting_seconds must never be negative, got {}",
                    m.state.sitting_seconds
                );
            }
        }

        #[test]
        fn prop_standing_seconds_never_negative(
            readings in prop::collection::vec(1000u32..1200u32, 10..100)
        ) {
            let mut m = SessionManager::new();
            for mm in readings {
                let _ = m.on_reading(mm as i32, true);
                prop_assert!(
                    m.state.standing_seconds >= 0,
                    "standing_seconds must never be negative, got {}",
                    m.state.standing_seconds
                );
            }
        }

        #[test]
        fn prop_debounce_count_bounded(
            readings in prop::collection::vec(750u32..1100u32, 20..200)
        ) {
            let mut m = SessionManager::new();
            for mm in readings {
                let _ = m.on_reading(mm as i32, true);
                prop_assert!(
                    m.pending_count < DEBOUNCE_COUNT + 1,
                    "pending_count must be < DEBOUNCE_COUNT + 1, got {}",
                    m.pending_count
                );
            }
        }

        #[test]
        fn prop_break_credit_never_adds_time(
            initial_sitting in 0i64..10000i64,
            break_secs in 0i64..3600i64
        ) {
            let mut m = SessionManager::new();
            m.state.sitting_seconds = initial_sitting;
            m.apply_break_credit(break_secs);
            prop_assert!(
                m.state.sitting_seconds <= initial_sitting,
                "break credit must never increase sitting time. before={}, after={}, break={}s",
                initial_sitting,
                m.state.sitting_seconds,
                break_secs
            );
        }

        #[test]
        fn prop_state_is_always_valid(
            readings in prop::collection::vec(0i32..2000i32, 10..100)
        ) {
            let mut m = SessionManager::new();
            for (idx, mm) in readings.iter().enumerate() {
                let active = idx % 2 == 0;
                let _ = m.on_reading(*mm, active);
                match m.state.state {
                    DeskState::Sitting
                    | DeskState::Standing
                    | DeskState::Walking
                    | DeskState::Away => {}
                }
            }
        }
    }
}
