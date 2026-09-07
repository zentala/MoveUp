//! Contract tests for [`crate::remote_protocol`].
//!
//! Every file in `tests/fixtures/relay-protocol/` is parsed into its typed
//! payload and re-serialized; the result must equal the input. The same
//! fixtures are re-parsed by `src/remote/protocol.test.ts`, so a field renamed
//! on one side fails on both.

use super::remote_protocol::*;
use serde_json::Value;
use std::path::PathBuf;

/// Named in the epic handoff. An empty glob is a failure, never a pass.
const MIN_FIXTURES: usize = 12;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("tests")
        .join("fixtures")
        .join("relay-protocol")
}

fn fixtures() -> Vec<(String, Value)> {
    let dir = fixture_dir();
    let mut out: Vec<(String, Value)> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|e| e.expect("dir entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .map(|p| {
            let name = p.file_stem().expect("stem").to_string_lossy().into_owned();
            let text = std::fs::read_to_string(&p).expect("readable fixture");
            let value = serde_json::from_str(&text)
                .unwrap_or_else(|e| panic!("{} is not valid JSON: {e}", p.display()));
            (name, value)
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Re-encodes an envelope through its typed payload. `Value` equality is
/// order-insensitive, which is exactly "byte-equal after key sorting".
fn round_trip(raw: &Value) -> Value {
    let env: Envelope<Value> = serde_json::from_value(raw.clone()).expect("envelope parses");
    check_version(env.v).expect("fixture speaks the current protocol version");

    let payload = match env.msg_type {
        MessageType::Hello => {
            let p: Hello = serde_json::from_value(env.payload).expect("hello payload");
            serde_json::to_value(p).expect("hello re-encodes")
        }
        MessageType::Welcome => {
            let p: Welcome = serde_json::from_value(env.payload).expect("welcome payload");
            serde_json::to_value(p).expect("welcome re-encodes")
        }
        MessageType::DeskStatus => {
            let p: DeskStatus = serde_json::from_value(env.payload).expect("desk_status payload");
            serde_json::to_value(p).expect("desk_status re-encodes")
        }
        MessageType::Command => {
            let p: Command = serde_json::from_value(env.payload).expect("command payload");
            validate_command(&p.name, &p.args).expect("fixture command is allowlisted");
            serde_json::to_value(p).expect("command re-encodes")
        }
        MessageType::CommandResult => {
            let p: CommandResult =
                serde_json::from_value(env.payload).expect("command_result payload");
            serde_json::to_value(p).expect("command_result re-encodes")
        }
        MessageType::Error => {
            let p: ErrorBody = serde_json::from_value(env.payload).expect("error payload");
            serde_json::to_value(p).expect("error re-encodes")
        }
        // `event` stays opaque on purpose: a DisplayEvent variant this build
        // does not know must survive the envelope layer and be rejected (or
        // ignored) by the reducer, not by the transport.
        MessageType::Event | MessageType::Ping | MessageType::Pong | MessageType::Unknown(_) => {
            env.payload
        }
    };

    serde_json::to_value(Envelope {
        v: env.v,
        msg_type: env.msg_type,
        id: env.id,
        ts: env.ts,
        payload,
    })
    .expect("envelope re-encodes")
}

#[test]
fn fixture_directory_is_not_empty() {
    let found = fixtures().len();
    assert!(
        found >= MIN_FIXTURES,
        "expected at least {MIN_FIXTURES} protocol fixtures in {}, found {found}",
        fixture_dir().display()
    );
}

#[test]
fn every_fixture_round_trips() {
    let all = fixtures();
    assert!(!all.is_empty(), "no fixtures were processed");
    for (name, raw) in &all {
        assert_eq!(&round_trip(raw), raw, "fixture {name} did not round-trip");
    }
}

#[test]
fn every_message_type_has_a_fixture() {
    let seen: Vec<String> = fixtures()
        .iter()
        .map(|(_, v)| v["type"].as_str().expect("type is a string").to_string())
        .collect();
    for want in [
        "hello",
        "welcome",
        "event",
        "desk_status",
        "command",
        "command_result",
        "ping",
        "error",
    ] {
        assert!(seen.iter().any(|s| s == want), "no fixture of type {want}");
    }
}

#[test]
fn ping_payload_is_null_but_the_key_is_required() {
    let ok: Envelope<Value> = serde_json::from_str(
        r#"{"v":1,"type":"ping","id":"a","ts":1,"payload":null}"#,
    )
    .expect("null payload is valid");
    assert!(ok.payload.is_null());

    let err = serde_json::from_str::<Envelope<Value>>(r#"{"v":1,"type":"ping","id":"a","ts":1}"#);
    assert!(err.is_err(), "a missing payload key must not parse");
}

#[test]
fn unsupported_version_is_a_typed_error() {
    let env: Envelope<Value> =
        serde_json::from_str(r#"{"v":2,"type":"ping","id":"a","ts":1,"payload":null}"#)
            .expect("v2 still parses as an envelope");
    let err = check_version(env.v).expect_err("v2 must be rejected");
    assert_eq!(err.code, "unsupported_version");
}

#[test]
fn unknown_message_type_survives_the_envelope_layer() {
    let env: Envelope<Value> =
        serde_json::from_str(r#"{"v":1,"type":"tomorrow","id":"a","ts":1,"payload":{}}"#)
            .expect("unknown type parses");
    assert_eq!(env.msg_type, MessageType::Unknown("tomorrow".into()));
    assert_eq!(env.msg_type.as_str(), "tomorrow");
}

#[test]
fn message_type_round_trips_through_json() {
    for t in [
        MessageType::Hello,
        MessageType::Welcome,
        MessageType::Event,
        MessageType::DeskStatus,
        MessageType::Command,
        MessageType::CommandResult,
        MessageType::Ping,
        MessageType::Pong,
        MessageType::Error,
        MessageType::Unknown("later".into()),
    ] {
        let json = serde_json::to_string(&t).expect("serializes");
        let back: MessageType = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(t, back);
    }
}

#[test]
fn close_codes_match_the_protocol_table() {
    assert_eq!(
        [
            CLOSE_PROTOCOL_ERROR,
            CLOSE_UNAUTHENTICATED,
            CLOSE_UNENTITLED,
            CLOSE_REVOKED,
            CLOSE_REPLACED,
            CLOSE_RATE_LIMITED,
            CLOSE_SHEDDING,
        ],
        [4400, 4401, 4402, 4403, 4409, 4429, 4503]
    );
}

#[test]
fn allowlist_holds_exactly_the_three_v1_commands() {
    let names: Vec<&str> = COMMAND_ALLOWLIST.iter().map(|s| s.name).collect();
    assert_eq!(names, ["ack_alert", "set_limits", "switch_profile"]);
}

#[test]
fn empty_args_are_valid_for_ack_alert_and_invalid_for_set_limits() {
    let empty = serde_json::json!({});
    assert!(validate_command("ack_alert", &empty).is_ok());
    let err = validate_command("set_limits", &empty).expect_err("set_limits needs an argument");
    assert_eq!(err.code, "bad_args");
}

#[test]
fn set_limits_enforces_its_bounds() {
    assert!(validate_command("set_limits", &serde_json::json!({"sit_min": 30})).is_ok());
    assert!(validate_command("set_limits", &serde_json::json!({"stand_min": 1})).is_ok());
    assert!(
        validate_command("set_limits", &serde_json::json!({"sit_min": 4})).is_err(),
        "4 is below the 5-minute floor"
    );
    assert!(validate_command("set_limits", &serde_json::json!({"sit_min": 241})).is_err());
    assert!(validate_command("set_limits", &serde_json::json!({"stand_min": 121})).is_err());
    assert!(validate_command("set_limits", &serde_json::json!({"sit_min": "30"})).is_err());
}

#[test]
fn switch_profile_checks_kind_and_name_shape() {
    let ok = serde_json::json!({"kind": "communication", "name": "gentle"});
    assert!(validate_command("switch_profile", &ok).is_ok());
    for bad in [
        serde_json::json!({"kind": "ergonomic"}),
        serde_json::json!({"kind": "sideways", "name": "gentle"}),
        serde_json::json!({"kind": "ergonomic", "name": "Strict"}),
        serde_json::json!({"kind": "ergonomic", "name": ""}),
        serde_json::json!({"kind": "ergonomic", "name": "../../etc/passwd"}),
    ] {
        let err = validate_command("switch_profile", &bad).expect_err("must be rejected");
        assert_eq!(err.code, "bad_args", "for {bad}");
    }
}

#[test]
fn unknown_arguments_and_unknown_names_are_rejected() {
    let err = validate_command("ack_alert", &serde_json::json!({"force": true}))
        .expect_err("no extra arguments");
    assert_eq!(err.code, "bad_args");

    let err = validate_command("save_settings", &serde_json::json!({}))
        .expect_err("there is no generic write path");
    assert_eq!(err.code, "unknown_command");

    let err =
        validate_command("ack_alert", &serde_json::json!([])).expect_err("args must be an object");
    assert_eq!(err.code, "bad_args");
}
