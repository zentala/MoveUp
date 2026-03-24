//! notification_service_tests.rs — Comprehensive tests for NotificationService.
//!
//! Covers: build_intents routing, named intent constructors, log tag mapping,
//! backend selection logic, and edge cases.

use crate::notification_service::{notification_log_tag, NotificationService};
use crate::session_types::NotificationEvent;

// ─── build_intents: individual event types ──────────────────────────────────

#[test]
fn build_intents_inactivity_content() {
    let intents = NotificationService::build_intents(
        &[NotificationEvent::Inactivity],
        3600,
        0,
    );
    assert_eq!(intents.len(), 1);
    assert!(intents[0].title.contains("60 minutes"));
    assert_eq!(intents[0].body, "Time to move.");
}

#[test]
fn build_intents_posture_balance_content() {
    let intents = NotificationService::build_intents(
        &[NotificationEvent::PostureBalance],
        7200,
        600,
    );
    assert_eq!(intents.len(), 1);
    assert!(intents[0].title.contains("sitting most"));
    assert!(intents[0].body.contains("standing"));
}

#[test]
fn build_intents_posture_balance_zero_standing() {
    let intents = NotificationService::build_intents(
        &[NotificationEvent::PostureBalance],
        3600,
        0,
    );
    assert_eq!(intents.len(), 1);
    // Should not panic on division by zero — ratio becomes INFINITY
    assert!(intents[0].title.contains("sitting"));
}

#[test]
fn build_intents_praise_content() {
    let intents = NotificationService::build_intents(
        &[NotificationEvent::Praise],
        0,
        450,
    );
    assert_eq!(intents.len(), 1);
    assert!(intents[0].title.contains("Halfway"));
    assert_eq!(intents[0].body, "Keep it up.");
}

#[test]
fn build_intents_standing_target_reached_content() {
    let intents = NotificationService::build_intents(
        &[NotificationEvent::StandingTargetReached],
        0,
        900,
    );
    assert_eq!(intents.len(), 1);
    assert!(intents[0].title.contains("Standing target reached"));
    assert!(intents[0].body.contains("full target"));
}

#[test]
fn build_intents_stand_limit_reached_is_skipped() {
    let intents = NotificationService::build_intents(
        &[NotificationEvent::StandLimitReached],
        0,
        5400,
    );
    assert!(intents.is_empty(), "StandLimitReached should be a no-op in build_intents");
}

// ─── build_intents: empty and multiple events ───────────────────────────────

#[test]
fn build_intents_empty_input() {
    let intents = NotificationService::build_intents(&[], 0, 0);
    assert!(intents.is_empty());
}

#[test]
fn build_intents_multiple_events_preserves_order() {
    let events = vec![
        NotificationEvent::Inactivity,
        NotificationEvent::Praise,
        NotificationEvent::StandingTargetReached,
    ];
    let intents = NotificationService::build_intents(&events, 3600, 900);
    assert_eq!(intents.len(), 3);
    assert!(intents[0].title.contains("60 minutes"));
    assert!(intents[1].title.contains("Halfway"));
    assert!(intents[2].title.contains("Standing target"));
}

#[test]
fn build_intents_filters_noop_from_mixed() {
    let events = vec![
        NotificationEvent::Inactivity,
        NotificationEvent::StandLimitReached,
        NotificationEvent::Praise,
    ];
    let intents = NotificationService::build_intents(&events, 3600, 0);
    assert_eq!(intents.len(), 2, "StandLimitReached filtered, others kept");
    assert!(intents[0].title.contains("60 minutes"));
    assert!(intents[1].title.contains("Halfway"));
}

#[test]
fn build_intents_all_noop_events() {
    let events = vec![
        NotificationEvent::StandLimitReached,
        NotificationEvent::StandLimitReached,
    ];
    let intents = NotificationService::build_intents(&events, 0, 0);
    assert!(intents.is_empty());
}

// ─── Named intent constructors ──────────────────────────────────────────────

#[test]
fn sit_limit_intent_title_and_body() {
    let intent = NotificationService::sit_limit_intent();
    assert!(intent.title.contains("stand up"));
    assert!(intent.body.contains("40 minutes"));
}

#[test]
fn stand_limit_intent_title_and_body() {
    let intent = NotificationService::stand_limit_intent();
    assert!(intent.title.contains("standing"));
    assert!(intent.body.contains("sit down"));
}

#[test]
fn praise_halfway_intent_title_and_body() {
    let intent = NotificationService::praise_halfway_intent();
    assert!(intent.title.contains("Halfway"));
    assert_eq!(intent.body, "Keep it up.");
}

// ─── Log tag mapping ────────────────────────────────────────────────────────

#[test]
fn log_tag_all_variants() {
    assert_eq!(notification_log_tag(&NotificationEvent::Inactivity), "inactivity");
    assert_eq!(notification_log_tag(&NotificationEvent::PostureBalance), "posture_balance");
    assert_eq!(notification_log_tag(&NotificationEvent::Praise), "praise");
    assert_eq!(notification_log_tag(&NotificationEvent::StandLimitReached), "stand_limit");
    assert_eq!(
        notification_log_tag(&NotificationEvent::StandingTargetReached),
        "standing_target_reached"
    );
}

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
    // Invalid value: neither toast nor popup fires — matches dispatch() behavior.
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
    // Currently reuses Inactivity variant for sit-limit
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
    // Defensive: negative values should not panic
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
    // NotificationIntent derives Debug — verify it doesn't panic
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
