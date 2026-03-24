//! notification_service_tests.rs — Tests for NotificationService intent building.

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
