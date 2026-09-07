//! commands_relay_tests.rs — the supervisor's decision table and the snapshot
//! the relay opens with (E022-T06).
//!
//! The six `#[tauri::command]` wrappers need a running app to call, so what is
//! tested here is everything they delegate to: when a socket may exist at all,
//! what the desk calls itself, and that the first message the relay receives is
//! a real snapshot rather than an empty object.

use std::sync::{Arc, Mutex};

use crate::commands_relay::{desk_name, plan_action, AppSnapshot, ClientAction};
use crate::health_source::HealthAggregator;
use crate::relay_client::SnapshotSource;

#[test]
fn a_socket_opens_only_when_the_feature_is_on_and_credentials_exist() {
    assert_eq!(plan_action(true, true, true), ClientAction::Start);
}

#[test]
fn every_missing_half_stops_the_client() {
    // "Switched off", "no token" and "not registered" are three different
    // facts; none of them may look like a desk that is merely reconnecting.
    for (enabled, token, desk) in [
        (false, true, true),
        (true, false, true),
        (true, true, false),
        (false, false, false),
    ] {
        assert_eq!(
            plan_action(enabled, token, desk),
            ClientAction::Stop,
            "enabled={enabled} token={token} desk={desk}"
        );
    }
}

#[test]
fn the_desk_names_itself_after_the_machine_or_falls_back() {
    let name = desk_name();
    assert!(!name.trim().is_empty(), "a desk name is always sent to the relay");
}

#[test]
fn the_opening_snapshot_is_a_real_snapshot_event() {
    let source = AppSnapshot {
        session: Arc::new(Mutex::new(crate::session::SessionManager::new())),
        comm_policy: Arc::new(Mutex::new(crate::communication_policy::CommunicationPolicy::new(
            Default::default(),
            Default::default(),
        ))),
        today_cache: Arc::new(Mutex::new(crate::db::TodaySummary::default())),
        health: Arc::new(HealthAggregator::new(vec![])),
    };

    let value = source.snapshot();
    assert_eq!(
        value["event"], "snapshot",
        "the relay's first message must be the same event the LAN path sends"
    );
    assert!(
        value["payload"]["session"].is_object(),
        "an empty payload would leave the phone with nothing to render: {value}"
    );
    assert!(
        value["payload"]["metrics"].as_array().is_some_and(|m| !m.is_empty()),
        "metrics must be computed, not left empty: {value}"
    );
}
