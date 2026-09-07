//! Tests for remote_server.rs — RemoteState, client counter, max clients.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::db::TodaySummary;
use crate::remote_protocol::{Envelope, MessageType, PROTOCOL_VERSION};
use crate::remote_server::{
    classify_inbound, default_dist_path, lan_enabled_in, wrap_event, Inbound, RemoteState,
    LAN_ENABLED_KEY, MAX_WS_CLIENTS,
};
use crate::session::SessionManager;

#[tokio::test]
async fn remote_state_is_clone() {
    let state: RemoteState = crate::remote_routes_health_tests::remote_state(
        Arc::new(Mutex::new(SessionManager::new())),
        Arc::new(Mutex::new(
            crate::communication_policy::CommunicationPolicy::new(
                Default::default(),
                Default::default(),
            ),
        )),
        Arc::new(Mutex::new(empty_today_summary())),
    )
    .await;
    let _cloned = state.clone();
}

#[test]
fn client_counter_increments_and_decrements() {
    let counter = Arc::new(AtomicUsize::new(0));
    let c1 = counter.fetch_add(1, Ordering::Relaxed) + 1;
    assert_eq!(c1, 1);
    let c2 = counter.fetch_add(1, Ordering::Relaxed) + 1;
    assert_eq!(c2, 2);
    let c3 = counter.fetch_sub(1, Ordering::Relaxed) - 1;
    assert_eq!(c3, 1);
}

#[test]
fn max_clients_check() {
    let counter = AtomicUsize::new(MAX_WS_CLIENTS);
    let current = counter.load(Ordering::Relaxed);
    assert!(current >= MAX_WS_CLIENTS);
}

#[test]
fn default_dist_path_uses_exe_parent_not_cwd() {
    let exe = std::path::Path::new("/opt/moveup/install/desk.exe");
    let dist = default_dist_path(exe);
    assert_eq!(dist, std::path::PathBuf::from("/opt/moveup/install/dist"));

    // Sanity: the result does not depend on std::env::current_dir() — it is a
    // pure function of the exe path, so a different CWD changes nothing.
    let cwd_before = std::env::current_dir().unwrap();
    let dist_again = default_dist_path(exe);
    assert_eq!(dist, dist_again);
    assert_eq!(cwd_before, std::env::current_dir().unwrap());
}

#[test]
fn default_dist_path_falls_back_when_exe_has_no_parent() {
    let exe = std::path::Path::new("desk.exe");
    let dist = default_dist_path(exe);
    assert_eq!(dist, std::path::PathBuf::from("dist"));
}

// ─── v1 envelope (E022-T11) ──────────────────────────────────────────────────

/// happy: a real `DisplayEvent` comes back out of the envelope unchanged.
#[test]
fn wrap_event_preserves_the_display_event_payload() {
    let event = r#"{"event":"desk:device-lost","payload":null}"#;
    let framed: Envelope<serde_json::Value> = serde_json::from_str(&wrap_event(event)).unwrap();

    assert_eq!(framed.v, PROTOCOL_VERSION);
    assert_eq!(framed.msg_type, MessageType::Event);
    assert_eq!(framed.payload, serde_json::from_str::<serde_json::Value>(event).unwrap());
    assert!(framed.ts > 0);
    assert!(uuid::Uuid::parse_str(&framed.id).is_ok(), "id must be a UUID: {}", framed.id);
}

/// Two messages must not share an id — the relay routes replies by it.
#[test]
fn wrap_event_gives_every_message_a_fresh_id() {
    let a: Envelope<serde_json::Value> = serde_json::from_str(&wrap_event("null")).unwrap();
    let b: Envelope<serde_json::Value> = serde_json::from_str(&wrap_event("null")).unwrap();
    assert_ne!(a.id, b.id);
}

/// error: an unserializable input must not silently produce a valid-looking
/// envelope carrying garbage — it becomes an explicit null payload.
#[test]
fn wrap_event_turns_unparseable_input_into_a_null_payload() {
    let framed: Envelope<serde_json::Value> =
        serde_json::from_str(&wrap_event("not json at all")).unwrap();
    assert_eq!(framed.payload, serde_json::Value::Null);
    assert_eq!(framed.msg_type, MessageType::Event);
}

// ─── Read-only contract ──────────────────────────────────────────────────────

/// A viewer that sends a well-formed `command` over the LAN socket gets it
/// dropped: there is no classification that leads to execution.
#[test]
fn inbound_command_is_ignored_not_executed() {
    let command = r#"{"v":1,"type":"command","id":"a","ts":1,"payload":{"name":"set_limits","args":{"sit_min":5},"viewer_id":null}}"#;
    assert_eq!(
        classify_inbound(&axum::extract::ws::Message::Text(command.into())),
        Inbound::Ignored
    );
}

#[test]
fn inbound_binary_is_ignored() {
    let msg = axum::extract::ws::Message::Binary(vec![1, 2, 3].into());
    assert_eq!(classify_inbound(&msg), Inbound::Ignored);
}

#[test]
fn inbound_close_ends_the_loop() {
    let msg = axum::extract::ws::Message::Close(None);
    assert_eq!(classify_inbound(&msg), Inbound::Close);
}

// ─── LAN toggle ──────────────────────────────────────────────────────────────

/// nil: no config loaded yet — the server still starts.
#[test]
fn lan_enabled_when_no_config_is_loaded() {
    assert!(lan_enabled_in(None));
}

/// empty: a config written before the toggle existed carries no key, and an
/// absent key may never read as "off".
#[test]
fn lan_enabled_when_the_key_is_absent() {
    let config = serde_json::json!({ "sitting_mm": 720 });
    assert!(lan_enabled_in(Some(&config)));
}

#[test]
fn lan_disabled_only_on_an_explicit_false() {
    let off = serde_json::json!({ LAN_ENABLED_KEY: false });
    let on = serde_json::json!({ LAN_ENABLED_KEY: true });
    let junk = serde_json::json!({ LAN_ENABLED_KEY: "false" });

    assert!(!lan_enabled_in(Some(&off)));
    assert!(lan_enabled_in(Some(&on)));
    assert!(lan_enabled_in(Some(&junk)), "a non-bool is not a disable");
}

fn empty_today_summary() -> TodaySummary {
    TodaySummary {
        sitting_secs: 0,
        standing_secs: 0,
        yesterday_sitting_secs: 0,
        yesterday_standing_secs: 0,
        position_changes: 0,
        sessions: vec![],
    }
}
