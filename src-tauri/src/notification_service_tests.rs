//! notification_service_tests.rs — Tests for NotificationService intent building.

use crate::notification_service::{notification_log_tag, NotificationService};
use crate::session_types::NotificationEvent;

#[test]
fn round2_disconnected_actual_final_effect_is_one_main_without_os_or_popup() {
    use crate::communication_policy::{CommunicationPolicy, PolicyInput};
    use crate::communication_profile::CommunicationProfile;
    use crate::ergonomic_profile::ErgonomicProfile;
    use crate::session_types::DeskState;
    for notify_once in ["toast", "popup"] {
        let mut profile = CommunicationProfile::default();
        profile.disconnected.notify_once = Some(notify_once.into());
        let mut policy = CommunicationPolicy::new(profile, ErgonomicProfile::default());
        let signal = policy.evaluate(&PolicyInput { state: DeskState::Sitting, elapsed_secs: 0,
            sensor_connected: false, observation_fresh: false, standing_lap: 0,
            standing_lap_progress: 0.0, standing_lap_flash: false, continuous_computer_secs: 0 }).notify.unwrap();
        let metadata = policy.take_reminder();
        let mut effects = Vec::new();
        NotificationService::dispatch_policy_signal_with(&signal, metadata.as_ref(), || {}, |channel, _, _, _|
            effects.push(format!("{channel:?}")));
        assert_eq!(effects, vec!["Main"], "actual final effect for disconnect={notify_once}");
        assert!(metadata.is_some(), "disconnected policy decision supplies typed metadata");
        let metadata = metadata.unwrap();
        assert_eq!(metadata.reason, crate::communication_types::ReminderReason::DeviceDisconnected);
        assert_eq!(metadata.category, crate::communication_types::ReminderCategory::Warning);
        assert_eq!(metadata.body, policy.comm_profile().messages.sensor_disconnected);
        assert_eq!(metadata.elapsed_secs, None); assert_eq!(metadata.limit_secs, None);
        let mut input = PolicyInput { state: DeskState::Sitting, elapsed_secs: 0,
            sensor_connected: false, observation_fresh: false, standing_lap: 0,
            standing_lap_progress: 0.0, standing_lap_flash: false, continuous_computer_secs: 0 };
        assert!(policy.evaluate(&input).notify.is_none(), "notify_once remains one shot");
        let facts = crate::session_manager::SessionManager::new().notification_facts(chrono::Utc::now());
        let day = facts.now.with_timezone(&chrono::Local).date_naive();
        let state = crate::app_reminders::AppReminderState::default();
        let first_id = { let mut guard = state.0.lock().unwrap();
            guard.publish(metadata.clone(), crate::app_reminders::ReminderSource::Policy, day);
            guard.reconcile(|entry| policy.reminder_obsolete_reason(&entry.draft, entry.day, &input, &facts, 2400, 0, std::time::Instant::now()));
            guard.snapshot().active.unwrap().id };
        input.sensor_connected = true;
        { let mut guard = state.0.lock().unwrap();
            guard.reconcile(|entry| policy.reminder_obsolete_reason(&entry.draft, entry.day, &input, &facts, 2400, 0, std::time::Instant::now()));
            assert!(guard.snapshot().active.is_none(), "reconnect retires notice even before fresh posture coverage"); }
        policy.on_sensor_connected(); input.sensor_connected = false;
        let next_signal = policy.evaluate(&input).notify.unwrap();
        let next_metadata = policy.take_reminder().unwrap();
        assert!(next_metadata.occurrence > metadata.occurrence);
        effects.clear();
        NotificationService::dispatch_policy_signal_with(&next_signal, Some(&next_metadata), || {}, |channel, _, _, draft| {
            effects.push(format!("{channel:?}"));
            state.0.lock().unwrap().publish(draft.unwrap().clone(), crate::app_reminders::ReminderSource::Policy, day);
        });
        assert_eq!(effects, vec!["Main"]);
        assert_ne!(state.0.lock().unwrap().snapshot().active.unwrap().id, first_id);
    }
}

#[test]
fn round2_disconnected_none_silent_and_missing_metadata_have_zero_final_effects() {
    use crate::{communication_policy::{CommunicationPolicy, PolicyInput}, communication_profile::CommunicationProfile,
        ergonomic_profile::ErgonomicProfile, session_types::DeskState, communication_types::NotifySignal};
    let mut none = CommunicationProfile::default(); none.disconnected.notify_once = None;
    let silent: CommunicationProfile = serde_json::from_str(include_str!("../profiles/communication/silent.json")).unwrap();
    let mut disabled_popup = CommunicationProfile::default(); disabled_popup.notifications_enabled = false;
    disabled_popup.disconnected.notify_once = Some("popup".into());
    for profile in [none, silent, disabled_popup] {
        let mut policy = CommunicationPolicy::new(profile, ErgonomicProfile::default());
        let signals = policy.evaluate(&PolicyInput { state: DeskState::Sitting, elapsed_secs: 0,
            sensor_connected: false, observation_fresh: false, standing_lap: 0,
            standing_lap_progress: 0.0, standing_lap_flash: false, continuous_computer_secs: 0 });
        assert!(signals.notify.is_none()); assert!(policy.take_reminder().is_none());
    }
    let mut effects = Vec::new();
    NotificationService::dispatch_policy_signal_with(&NotifySignal::Toast("missing metadata".into()), None, || {},
        |channel, _, _, _| effects.push(channel));
    assert!(effects.is_empty(), "missing metadata cannot silently select an OS/popup fallback");
}

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
    // Title now shows actual numbers: "Sitting 2h 0m vs standing 0h 10m"
    assert!(intents[0].title.contains("Sitting"), "title: {}", intents[0].title);
    assert!(intents[0].title.contains("standing"), "title: {}", intents[0].title);
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
    // "Sitting 1h 0m vs standing 0h 0m"
    assert!(intents[0].title.contains("Sitting"), "title: {}", intents[0].title);
    assert!(intents[0].title.contains("0h 0m"), "title: {}", intents[0].title);
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
