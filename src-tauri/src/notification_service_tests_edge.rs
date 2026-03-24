//! notification_service_tests_edge.rs — Backend selection and edge case tests.

use crate::notification_service::NotificationService;
use crate::session_types::NotificationEvent;

// ─── Backend selection logic ────────────────────────────────────────────────

/// Helper: given a backend string, returns (use_toast, use_popup).
fn backend_flags(backend: &str) -> (bool, bool) {
    let use_toast = backend == "toast" || backend == "both";
    let use_popup = backend == "popup" || backend == "both";
    (use_toast, use_popup)
}

#[test]
fn backend_toast_only() {
    let (toast, popup) = backend_flags("toast");
    assert!(toast);
    assert!(!popup);
}

#[test]
fn backend_popup_only() {
    let (toast, popup) = backend_flags("popup");
    assert!(!toast);
    assert!(popup);
}

#[test]
fn backend_both() {
    let (toast, popup) = backend_flags("both");
    assert!(toast);
    assert!(popup);
}

#[test]
fn backend_invalid_falls_through() {
    let (toast, popup) = backend_flags("invalid");
    assert!(!toast);
    assert!(!popup);
}

#[test]
fn backend_empty_string_falls_through() {
    let (toast, popup) = backend_flags("");
    assert!(!toast);
    assert!(!popup);
}

// ─── Intent struct fields ───────────────────────────────────────────────────

#[test]
fn intent_event_field_matches_source() {
    let events = vec![NotificationEvent::Inactivity];
    let intents = NotificationService::build_intents(&events, 0, 0);
    assert!(matches!(intents[0].event, NotificationEvent::Inactivity));
}

#[test]
fn sit_limit_intent_event_type() {
    let intent = NotificationService::sit_limit_intent();
    assert!(matches!(intent.event, NotificationEvent::Inactivity));
}

#[test]
fn stand_limit_intent_event_type() {
    let intent = NotificationService::stand_limit_intent();
    assert!(matches!(intent.event, NotificationEvent::StandLimitReached));
}

#[test]
fn praise_halfway_intent_event_type() {
    let intent = NotificationService::praise_halfway_intent();
    assert!(matches!(intent.event, NotificationEvent::Praise));
}

// ─── Edge cases ─────────────────────────────────────────────────────────────

#[test]
fn build_intents_large_values_no_panic() {
    let intents = NotificationService::build_intents(
        &[NotificationEvent::PostureBalance],
        i64::MAX,
        1,
    );
    assert_eq!(intents.len(), 1);
}

#[test]
fn build_intents_negative_sitting_secs() {
    let intents = NotificationService::build_intents(
        &[NotificationEvent::PostureBalance],
        -100,
        50,
    );
    assert_eq!(intents.len(), 1);
}

#[test]
fn notification_intent_debug_format() {
    let intent = NotificationService::sit_limit_intent();
    let debug_str = format!("{:?}", intent);
    assert!(!debug_str.is_empty());
}

#[test]
fn notification_intent_clone() {
    let intent = NotificationService::sit_limit_intent();
    let cloned = intent.clone();
    assert_eq!(cloned.title, intent.title);
    assert_eq!(cloned.body, intent.body);
}
