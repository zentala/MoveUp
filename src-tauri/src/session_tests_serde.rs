//! session_tests_serde.rs — Contract tests: Rust↔Frontend serialization agreement.
//!
//! These tests verify that serde output matches what the TypeScript frontend expects.
//! If these fail, the frontend state comparisons (e.g. `state === "Standing"`) will
//! silently break — the UI will render but conditional logic won't fire.

#[cfg(test)]
mod serde_contract_tests {
    use crate::session_types::*;

    // ─── DeskState serialization ────────────────────────────────────────

    #[test]
    fn desk_state_serializes_to_pascal_case() {
        // Frontend compares: state === "Sitting", state === "Standing", etc.
        // If serde renames to snake_case, all frontend conditionals break silently.
        assert_eq!(
            serde_json::to_string(&DeskState::Sitting).unwrap(),
            r#""Sitting""#
        );
        assert_eq!(
            serde_json::to_string(&DeskState::Standing).unwrap(),
            r#""Standing""#
        );
        assert_eq!(
            serde_json::to_string(&DeskState::Walking).unwrap(),
            r#""Walking""#
        );
        assert_eq!(
            serde_json::to_string(&DeskState::Away).unwrap(),
            r#""Away""#
        );
    }

    #[test]
    fn desk_state_deserializes_from_pascal_case() {
        assert_eq!(
            serde_json::from_str::<DeskState>(r#""Sitting""#).unwrap(),
            DeskState::Sitting
        );
        assert_eq!(
            serde_json::from_str::<DeskState>(r#""Standing""#).unwrap(),
            DeskState::Standing
        );
    }

    // ─── SessionStateDto fields ─────────────────────────────────────────

    #[test]
    fn session_state_dto_state_field_is_pascal_case() {
        // Simulates what get_session_state IPC returns to the frontend.
        let dto = SessionStateDto {
            state: DeskState::Standing,
            sitting_seconds: 0,
            standing_seconds: 300,
            break_seconds: 300,
            session_limit_secs: 2700,
            stand_limit_secs: 0,
            desk_height_cm: 110.0,
            position_changes: 1,
            limit_used_secs: 0,
            daily_score: 0.0,
            standing_session_secs: 300,
            current_session_secs: 0,
            continuous_computer_secs: 0,
            longest_computer_session_secs: 0,
            sitting_seconds_total: 0,
            idle_secs: 0,
            away_bout_secs: 0,
            max_continuous_computer_secs: 3600,
        };
        let json = serde_json::to_value(&dto).unwrap();
        assert_eq!(json["state"], "Standing");
    }

    #[test]
    fn state_changed_payload_state_field_is_pascal_case() {
        let payload = StateChangedPayload {
            state: DeskState::Sitting,
            sitting_seconds: 600,
            standing_seconds: 0,
            break_seconds: 0,
            desk_height_cm: 72.0,
            position_changes: 2,
            last_break_secs: 300,
            last_sitting_secs: 600,
            break_credit: BreakCredit::Partial,
            current_session_secs: 0,
        };
        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["state"], "Sitting");
    }

    // ─── Debug format matches serde (DB uses Debug) ─────────────────────

    #[test]
    fn desk_state_debug_matches_serde() {
        // DB saves via format!("{:?}", state) — must match serde output
        // so DB queries and frontend agree on state strings.
        let states = [
            DeskState::Sitting,
            DeskState::Standing,
            DeskState::Walking,
            DeskState::Away,
        ];
        for state in &states {
            let debug_str = format!("{:?}", state);
            let serde_str = serde_json::to_string(state)
                .unwrap()
                .trim_matches('"')
                .to_string();
            assert_eq!(
                debug_str, serde_str,
                "Debug and serde must agree for {:?}: debug={}, serde={}",
                state, debug_str, serde_str
            );
        }
    }
}
