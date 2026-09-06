//! session_dto_fixture_tests.rs — E015: emits the JSON fixture the TypeScript
//! drift test reads.
//!
//! Rust owns the wire format; TypeScript only mirrors it. The fixture is the
//! handshake between the two: this test serialises the real structs and
//! compares the result with the committed file, so a field added, renamed or
//! removed on the Rust side fails here, and `src/test/dto-drift.test.ts` fails
//! on the TypeScript side until both agree again.
//!
//! Regenerate after an intentional change:
//! `UPDATE_FIXTURES=1 cargo test --lib -- e015_dto_fixture`.

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use crate::session_types::*;

    /// Path of the committed fixture, relative to `src-tauri/`.
    const FIXTURE: &str = "../src/test/fixtures/session-dto.json";

    fn fixture_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE)
    }

    /// A DTO with a distinct value per field, so a silent field swap in the
    /// wire format shows up as a changed fixture rather than as equal numbers.
    fn sample_dto() -> SessionStateDto {
        SessionStateDto {
            state: DeskState::Sitting,
            sitting_seconds: 1500,
            standing_seconds: 600,
            break_seconds: 120,
            session_limit_secs: 2700,
            stand_limit_secs: 900,
            desk_height_cm: 72.5,
            position_changes: 3,
            limit_used_secs: 1260,
            daily_score: 12.5,
            standing_session_secs: 240,
            secs_since_last_break: 300,
            continuous_computer_secs: 3600,
            longest_computer_session_secs: 5400,
            sitting_seconds_total: 1800,
            idle_secs: 30,
            away_bout_secs: 60,
            max_continuous_computer_secs: 7200,
        }
    }

    fn sample_payload() -> StateChangedPayload {
        StateChangedPayload {
            state: DeskState::Standing,
            standing_seconds: 600,
            break_seconds: 120,
            desk_height_cm: 110.0,
            position_changes: 3,
            last_break_secs: 300,
            last_sitting_secs: 1500,
            break_credit: BreakCredit::Partial,
            limit_used_secs: 1260,
        }
    }

    /// Serialises both wire types and renders the fixture body.
    fn render_fixture() -> String {
        let value = serde_json::json!({
            "session_state_dto": sample_dto(),
            "state_changed_payload": sample_payload(),
        });
        format!("{}\n", serde_json::to_string_pretty(&value).expect("fixture serialises"))
    }

    /// The committed fixture must equal what the current structs serialise to.
    #[test]
    fn e015_dto_fixture_matches_committed_json() {
        let generated = render_fixture();
        let path = fixture_path();

        if std::env::var("UPDATE_FIXTURES").is_ok() {
            fs::create_dir_all(path.parent().expect("fixture has a parent"))
                .expect("fixture directory is writable");
            fs::write(&path, &generated).expect("fixture is writable");
            return;
        }

        let committed = fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!("fixture {} unreadable: {e} — run with UPDATE_FIXTURES=1", path.display())
        });

        assert_eq!(
            committed.replace("\r\n", "\n"),
            generated,
            "the DTO wire format changed; regenerate with UPDATE_FIXTURES=1 \
             and update the key list in src/test/dto-drift.test.ts"
        );
    }
}
