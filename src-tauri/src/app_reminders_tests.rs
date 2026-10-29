//! Disposable state, policy and production dispatch/acknowledgement regressions.
use crate::app_reminders::*;
use crate::communication_types::*;
use crate::communication_policy::{CommunicationPolicy, SessionNotificationFacts};
use crate::communication_profile::CommunicationProfile;
use crate::ergonomic_profile::ErgonomicProfile;
use crate::session_types::{DeskState};

fn draft(reason: ReminderReason) -> ReminderDraft {
    ReminderDraft { occurrence: reason as u64 + 1, reason, category: ReminderCategory::Warning, title: "Policy title".into(),
        body: "Policy body".into(), elapsed_secs: Some(2500), limit_secs: Some(2400), position_anchor: None, observation_day: None }
}
fn day() -> chrono::NaiveDate { chrono::Local::now().date_naive() }
fn state_with_message() -> AppReminderState {
    let state = AppReminderState::default();
    { let mut guard = state.0.lock().unwrap();
      guard.claim(None, "first").unwrap();
      guard.publish(draft(ReminderReason::SittingLimit), ReminderSource::Policy, day()); }
    state
}

fn acknowledged_close_guard(state: &AppReminderState, client: &str) -> crate::commands::ReminderCloseGuard {
    let active = state.0.lock().unwrap().snapshot();
    let id = active.active.as_ref().unwrap().id.clone();
    crate::commands_reminders::acknowledge_with(state, client, &id, active.revision, ReminderAction::Dismiss,
        |_| Ok(()), || Ok(())).unwrap();
    let acknowledged = state.0.lock().unwrap().snapshot();
    crate::commands::ReminderCloseGuard {
        session_id: acknowledged.session_id,
        client_id: acknowledged.client_id.unwrap(),
        reminder_id: id,
        revision: acknowledged.revision,
    }
}

#[test]
fn reminder_x_hide_rechecks_delayed_state_without_holding_business_lock_during_effect() {
    use std::cell::Cell;

    let state = state_with_message();
    let expected = acknowledged_close_guard(&state, "first");
    let invoked = Cell::new(false);
    crate::commands::hide_main_window_if_current_reminder(&state, &expected, || {
        assert!(state.0.try_lock().is_ok(), "native hide runs after the reminder-state lock is released");
        invoked.set(true);
        Ok(())
    }).unwrap();
    assert!(invoked.get());

    let newer_state = state_with_message();
    let stale = acknowledged_close_guard(&newer_state, "first");
    let newer_effect = Cell::new(false);
    // Model a hide callback delayed in the existing main-thread queue until after
    // reminder publication. It must read current state only when the callback runs.
    let delayed_hide = || crate::commands::hide_main_window_if_current_reminder(&newer_state, &stale, || {
        newer_effect.set(true); Ok(())
    });
    newer_state.0.lock().unwrap().publish(draft(ReminderReason::ScreenBreak), ReminderSource::Policy, day());
    assert_eq!(delayed_hide().unwrap_err(), "reminder_changed_before_hide");
    assert!(!newer_effect.get(), "a newer reminder cannot be hidden by the earlier X");

    let protected_state = state_with_message();
    let stale = acknowledged_close_guard(&protected_state, "first");
    protected_state.0.lock().unwrap().set_flow("first", 1, Some(ProtectedFlow::Settings)).unwrap();
    let protected_effect = Cell::new(false);
    assert_eq!(crate::commands::hide_main_window_if_current_reminder(&protected_state, &stale, || {
        protected_effect.set(true); Ok(())
    }).unwrap_err(), "reminder_changed_before_hide");
    assert!(!protected_effect.get(), "a protected view cannot be hidden by the earlier X");
}

#[test]
fn focus_loss_only_retains_a_presented_active_reminder_outside_protected_flows() {
    let state = AppReminderState::default();
    let mut guard = state.0.lock().unwrap();
    guard.claim(None, "main").unwrap();
    guard.set_flow("main", 1, Some(ProtectedFlow::Settings)).unwrap();
    assert!(!guard.retain_on_focus_loss(), "Settings without a message keeps ordinary blur behavior");
    guard.publish(draft(ReminderReason::Preview), ReminderSource::Test, day());
    assert!(!guard.retain_on_focus_loss(), "pending preview cannot make Settings sticky");
    guard.set_flow("main", 2, None).unwrap();
    assert!(!guard.retain_on_focus_loss(), "active state alone does not prove native presentation");
    let revision = guard.snapshot().revision;
    guard.record_presentation(revision, Ok(()));
    assert!(guard.retain_on_focus_loss());
    guard.set_flow("main", 3, Some(ProtectedFlow::Share)).unwrap();
    assert!(!guard.retain_on_focus_loss());
}

#[test]
fn entering_and_leaving_protected_flow_preserves_unread_a_with_pending_b() {
    let state = state_with_message();
    let mut guard = state.0.lock().unwrap();
    let first_id = guard.snapshot().active.unwrap().id;
    guard.publish(draft(ReminderReason::ScreenBreak), ReminderSource::Policy, day());
    let second_id = guard.snapshot().pending.unwrap().id;
    guard.set_flow("first", 1, Some(ProtectedFlow::Settings)).unwrap();
    guard.set_flow("first", 2, None).unwrap();
    assert_eq!(guard.snapshot().active.unwrap().id, first_id);
    assert_eq!(guard.snapshot().pending.unwrap().id, second_id);
}

#[test]
fn unread_current_without_ack_survives_settings_before_b_arrives() {
    let state = state_with_message();
    let a = state.0.lock().unwrap().snapshot().active.unwrap().id;
    let b = { let mut guard = state.0.lock().unwrap();
        guard.set_flow("first", 1, Some(ProtectedFlow::Settings)).unwrap();
        guard.publish(draft(ReminderReason::StandingTarget), ReminderSource::Session, day());
        assert!(!guard.retain_on_focus_loss());
        guard.set_flow("first", 2, None).unwrap();
        let current = guard.snapshot();
        assert_eq!(current.active.as_ref().map(|entry| &entry.id), Some(&a), "unread A cannot be replaced by later B during Settings");
        current.pending.unwrap().id };
    let revision = state.0.lock().unwrap().snapshot().revision;
    crate::commands_reminders::acknowledge_with(&state, "first", &a, revision, ReminderAction::Dismiss,
        |_| Ok(()), || Ok(())).unwrap();
    let next = state.0.lock().unwrap().snapshot();
    assert_eq!(next.active.as_ref().map(|entry| &entry.id), Some(&b));
    crate::commands_reminders::acknowledge_with(&state, "first", &b, next.revision, ReminderAction::Dismiss,
        |_| Ok(()), || Ok(())).unwrap();
    assert!(state.0.lock().unwrap().snapshot().active.is_none());
}
fn facts() -> SessionNotificationFacts {
    SessionNotificationFacts { state: DeskState::Standing, now: chrono::Utc::now(), observation_fresh: true,
        last_position_change_at: Some(chrono::Utc::now()), sitting_seconds_total: 7200, standing_seconds: 1800,
        standing_target_secs: 1800, inactivity_fired: false, posture_balance_fired: false,
        standing_target_fired: false, praise_fired: false }
}

#[test]
fn protected_flow_remount_rejects_old_client_and_preserves_pending_preview() {
    let state = AppReminderState::default();
    let mut guard = state.0.lock().unwrap();
    guard.claim(None, "old").unwrap();
    guard.set_flow("old", 1, Some(ProtectedFlow::Settings)).unwrap();
    guard.publish(draft(ReminderReason::Preview), ReminderSource::Test, day());
    let preview_id = guard.snapshot().pending.unwrap().id;
    guard.claim(Some("old"), "replacement").unwrap();
    assert!(guard.set_flow("old", 2, None).is_err());
    assert!(guard.claim(Some("old"), "late-mount").is_err());
    assert_eq!(guard.snapshot().protected_flow, Some(ProtectedFlow::Settings));
    guard.set_flow("replacement", 1, Some(ProtectedFlow::Share)).unwrap();
    assert!(guard.set_flow("replacement", 1, None).is_err());
    assert!(guard.snapshot().active.is_none());
    guard.set_flow("replacement", 2, None).unwrap();
    let current = guard.snapshot().active.unwrap();
    assert_eq!(current.id, preview_id); assert!(current.is_test);
}

#[test]
fn stale_id_revision_duplicate_delivery_and_native_result_cannot_reopen_or_cancel_newer() {
    let state = state_with_message();
    let mut guard = state.0.lock().unwrap();
    let first = guard.snapshot();
    let id = first.active.unwrap().id;
    guard.publish(draft(ReminderReason::SittingLimit), ReminderSource::Policy, day());
    assert_eq!(guard.snapshot().revision, first.revision);
    guard.begin_ack("first", &id, first.revision, ReminderAction::Dismiss).unwrap();
    guard.finish_ack(&id, Ok(())).unwrap();
    guard.publish(draft(ReminderReason::SittingLimit), ReminderSource::Policy, day());
    assert!(guard.snapshot().active.is_none());
    guard.publish(draft(ReminderReason::ScreenBreak), ReminderSource::Policy, day());
    let newer = guard.snapshot();
    assert!(guard.begin_ack("first", &id, newer.revision, ReminderAction::Dismiss).is_err());
    guard.record_presentation(first.revision, Err("late-native-error".into()));
    assert_eq!(guard.snapshot(), newer);
}

#[test]
fn actual_ack_orchestrator_retries_store_without_reapplying_snooze() {
    use std::cell::{Cell, RefCell};
    let state = state_with_message();
    let original = state.0.lock().unwrap().snapshot();
    let id = original.active.unwrap().id;
    let policy = RefCell::new(CommunicationPolicy::new(CommunicationProfile::default(), ErgonomicProfile::default()));
    let applications = Cell::new(0);
    let now = std::time::Instant::now();
    let invoke = |revision, fail| crate::commands_reminders::acknowledge_with(&state, "first", &id, revision,
        ReminderAction::Snooze, |_| {
            applications.set(applications.get() + 1);
            policy.borrow_mut().snooze_for_at(5, now);
            Ok(())
        }, || if fail { Err("store_failed".into()) } else { Ok(()) });
    assert!(invoke(original.revision, true).is_err());
    let retry = state.0.lock().unwrap().snapshot();
    assert_eq!(retry.active.unwrap().id, id);
    assert_eq!(retry.presentation_error.as_deref(), Some("store_failed"));
    invoke(retry.revision, false).unwrap();
    assert_eq!(applications.get(), 1);
    assert!(policy.borrow().is_snoozed_at(now));
    assert!(state.0.lock().unwrap().snapshot().active.is_none());
}

#[test]
fn preview_ack_uses_production_orchestrator_without_policy_or_store_side_effects() {
    let state = AppReminderState::default();
    let current = { let mut guard = state.0.lock().unwrap();
        guard.claim(None, "preview").unwrap();
        guard.publish(draft(ReminderReason::Preview), ReminderSource::Test, day()); guard.snapshot() };
    crate::commands_reminders::acknowledge_with(&state, "preview", &current.active.unwrap().id, current.revision,
        ReminderAction::Snooze, |_| panic!("preview must not touch policy"), || panic!("preview must not persist ergonomic state")).unwrap();
    let limiter = std::sync::atomic::AtomicI64::new(0);
    assert!(crate::commands::reserve_test_notification(&limiter, 100).is_ok());
    assert!(crate::commands::reserve_test_notification(&limiter, 159).is_err());
    assert!(crate::commands::reserve_test_notification(&limiter, 160).is_ok());
}

#[test]
fn two_rate_limited_product_previews_get_new_ids_without_real_policy_effects() {
    let state = AppReminderState::default();
    state.0.lock().unwrap().claim(None, "main").unwrap();
    let limiter = std::sync::atomic::AtomicI64::new(0);
    let mut ids = Vec::new();
    for now in [100, 160] {
        crate::commands::reserve_test_notification(&limiter, now).unwrap();
        let preview = crate::commands_reminders::preview_draft(ReminderCategory::Info);
        let current = { let mut guard = state.0.lock().unwrap();
            guard.publish(preview.clone(), ReminderSource::Test, day()); guard.snapshot() };
        let id = current.active.expect("each accepted invocation creates a preview").id;
        ids.push(id.clone());
        crate::commands_reminders::acknowledge_with(&state, "main", &id, current.revision,
            ReminderAction::Dismiss, |_| panic!("test has no policy effects"), || panic!("test has no ergonomic persistence")).unwrap();
        state.0.lock().unwrap().publish(preview, ReminderSource::Test, day());
        assert!(state.0.lock().unwrap().snapshot().active.is_none(), "late delivery of this occurrence cannot reopen it");
    }
    assert_ne!(ids[0], ids[1]);
}

#[test]
fn category_previews_validate_all_four_categories_share_one_rate_limit_and_have_no_policy_effects() {
    let state = AppReminderState::default(); state.0.lock().unwrap().claim(None, "main").unwrap();
    let limiter = std::sync::atomic::AtomicI64::new(0);
    for (index, name) in ["info", "warning", "urgent", "success"].into_iter().enumerate() {
        let category: ReminderCategory = serde_json::from_value(serde_json::json!(name)).unwrap();
        let now = 100 + index as i64 * 60;
        crate::commands::reserve_test_notification(&limiter, now).unwrap();
        let preview = crate::commands_reminders::preview_draft(category);
        let current = { let mut guard = state.0.lock().unwrap();
            guard.publish(preview, ReminderSource::Test, day()); guard.snapshot() };
        let message = current.active.unwrap();
        assert_eq!(message.category, category); assert!(message.is_test);
        assert_eq!(message.reason, ReminderReason::Preview);
        assert_eq!(message.elapsed_secs, None); assert_eq!(message.limit_secs, None);
        assert!(crate::commands::reserve_test_notification(&limiter, now + 1).is_err(), "another category cannot bypass shared limiter");
        crate::commands_reminders::acknowledge_with(&state, "main", &message.id, current.revision,
            ReminderAction::Snooze, |_| panic!("preview must not apply real policy"), || panic!("preview must not persist session")).unwrap();
    }
    assert!(serde_json::from_value::<ReminderCategory>(serde_json::json!("custom")).is_err());
}

#[test]
fn policy_session_production_dispatch_provides_typed_metadata_for_every_enabled_backend() {
    for backend in ["toast", "popup", "both", "none"] {
        let mut profile = CommunicationProfile::default(); profile.notification_backend = backend.into();
        let policy = CommunicationPolicy::new(profile, ErgonomicProfile::default());
        let mut current_facts = facts(); current_facts.praise_fired = true;
        let intents = policy.resolve_session_notifications(&current_facts, true);
        let state = AppReminderState::default();
        crate::notification_service::NotificationService::dispatch_reminders_with(&intents, |_| {}, |channels, _, _, metadata| {
            if !channels.is_empty() {
                let metadata = metadata.expect("real policy dispatch supplies metadata");
                assert_eq!(metadata.reason, ReminderReason::StandingTarget);
                state.0.lock().unwrap().publish(metadata.clone(), ReminderSource::Session, day());
            }
        });
        assert_eq!(state.0.lock().unwrap().snapshot().active.is_some(), backend != "none");
    }
}

#[test]
fn policy_retirement_preserves_goal_on_posture_change_but_retires_sitting_limit_and_stale() {
    let policy = CommunicationPolicy::new(CommunicationProfile::default(), ErgonomicProfile::default());
    let current_facts = facts();
    let mut input = crate::communication_policy::PolicyInput { state: DeskState::Standing, elapsed_secs: 3000,
        sensor_connected: true, observation_fresh: true, standing_lap: 0, standing_lap_progress: 0.2,
        standing_lap_flash: false, continuous_computer_secs: 0 };
    assert_eq!(policy.reminder_obsolete_reason(&draft(ReminderReason::StandingTarget), day(), &input, &current_facts, 2400, 3000, std::time::Instant::now()), None);
    assert_eq!(policy.reminder_obsolete_reason(&draft(ReminderReason::SittingLimit), day(), &input, &current_facts, 2400, 3000, std::time::Instant::now()), Some(RetirementReason::ContextChanged));
    input.observation_fresh = false;
    assert_eq!(policy.reminder_obsolete_reason(&draft(ReminderReason::StandingTarget), day(), &input, &current_facts, 2400, 3000, std::time::Instant::now()), Some(RetirementReason::ObservationUnavailable));
}

#[test]
fn bounded_queue_focus_guard_and_reconciliation_are_the_actual_adapter_state() {
    let state = state_with_message();
    let mut guard = state.0.lock().unwrap();
    let revision = guard.snapshot().revision;
    guard.record_presentation(revision, Ok(()));
    assert!(guard.retain_on_focus_loss());
    guard.publish(draft(ReminderReason::Praise), ReminderSource::Session, day());
    guard.publish(draft(ReminderReason::StandingTarget), ReminderSource::Session, day());
    assert_eq!(guard.snapshot().pending.unwrap().reason, ReminderReason::StandingTarget);
    guard.reconcile(|entry| (entry.draft.reason == ReminderReason::SittingLimit).then_some(RetirementReason::ContextChanged));
    assert_eq!(guard.snapshot().active.unwrap().reason, ReminderReason::StandingTarget);
    guard.reconcile(|_| Some(RetirementReason::ProfileDisabled));
    assert!(!guard.retain_on_focus_loss());
}

#[test]
fn snapshot_wire_has_explicit_nulls_and_inline_types() {
    use ts_rs::TS;
    let snapshot = AppReminderState::default().0.lock().unwrap().snapshot();
    let json = serde_json::to_value(snapshot).unwrap();
    for field in ["active", "pending", "protected_flow", "last_retirement", "presentation_error", "client_id"] {
        assert!(json.get(field).unwrap().is_null());
    }
    let ts = AppReminderSnapshot::export_to_string(&ts_rs::Config::new().with_large_int("number")).unwrap();
    assert!(ts.contains("active:")); assert!(ts.contains("null"));
    assert!(!ts.contains("bigint")); assert!(!ts.contains("import type"));
}

#[test]
fn ack_during_flow_change_removes_only_its_id_and_never_reopens_it_on_return() {
    let state = state_with_message();
    let mut guard = state.0.lock().unwrap();
    let current = guard.snapshot(); let id = current.active.unwrap().id;
    guard.begin_ack("first", &id, current.revision, ReminderAction::Dismiss).unwrap();
    guard.set_flow("first", 1, Some(ProtectedFlow::Settings)).unwrap();
    guard.finish_ack(&id, Ok(())).unwrap();
    guard.set_flow("first", 2, None).unwrap();
    assert!(guard.snapshot().active.is_none()); assert!(guard.snapshot().pending.is_none());
}

#[test]
fn round2_ack_reserved_a_survives_settings_publish_b_and_failed_store() {
    use std::cell::{Cell, RefCell};
    let state = state_with_message();
    let first = state.0.lock().unwrap().snapshot(); let a = first.active.unwrap().id;
    let applications = Cell::new(0);
    let policy = RefCell::new(CommunicationPolicy::new(CommunicationProfile::default(), ErgonomicProfile::default()));
    let now = std::time::Instant::now();
    let failed = crate::commands_reminders::acknowledge_with(&state, "first", &a, first.revision, ReminderAction::Snooze,
        |_| { applications.set(applications.get() + 1); policy.borrow_mut().snooze_for_at(5, now); Ok(()) },
        || {
            let mut guard = state.0.lock().unwrap();
            guard.set_flow("first", 1, Some(ProtectedFlow::Settings)).unwrap();
            guard.publish(draft(ReminderReason::StandingTarget), ReminderSource::Session, day());
            assert!(!guard.retain_on_focus_loss(), "protected Settings keeps ordinary blur behavior");
            Err("store_failed".into())
        });
    assert!(failed.is_err());
    let retry = state.0.lock().unwrap().snapshot();
    assert_eq!(retry.active.as_ref().map(|entry| &entry.id), Some(&a), "reserved A stays retryable, pending B cannot replace it");
    assert_eq!(retry.pending.as_ref().unwrap().reason, ReminderReason::StandingTarget);
    crate::commands_reminders::acknowledge_with(&state, "first", &a, retry.revision, ReminderAction::Snooze,
        |_| { applications.set(applications.get() + 1); Ok(()) }, || Ok(())).unwrap();
    assert_eq!(applications.get(), 1);
    assert!(policy.borrow().is_snoozed_at(now));
    let next = { let mut guard = state.0.lock().unwrap(); guard.set_flow("first", 2, None).unwrap(); guard.snapshot() };
    let b = next.active.unwrap().id;
    assert_ne!(a, b);
    crate::commands_reminders::acknowledge_with(&state, "first", &b, next.revision, ReminderAction::Dismiss,
        |_| { applications.set(applications.get() + 1); Ok(()) }, || Ok(())).unwrap();
    assert_eq!(applications.get(), 2);
    assert!(state.0.lock().unwrap().snapshot().active.is_none());
}

fn remount_retry_without_local_action(original: ReminderAction) {
    use std::cell::{Cell, RefCell};
    let state = state_with_message();
    let current = state.0.lock().unwrap().snapshot(); let id = current.active.unwrap().id;
    let applications = Cell::new(0);
    let policy = RefCell::new(CommunicationPolicy::new(CommunicationProfile::default(), ErgonomicProfile::default()));
    let now = std::time::Instant::now();
    crate::commands_reminders::acknowledge_with(&state, "first", &id, current.revision, original,
        |action| {
            applications.set(applications.get() + 1);
            match action {
                ReminderAction::Snooze => policy.borrow_mut().snooze_for_at(5, now),
                ReminderAction::Dismiss => policy.borrow_mut().dismiss_at(now),
                ReminderAction::Retry => panic!("initial action is not retry"),
            }
            Ok(())
        }, || Err("reminder_persistence_failed".into())).unwrap_err();
    let recovered = { let mut guard = state.0.lock().unwrap();
        guard.publish(draft(ReminderReason::StandingTarget), ReminderSource::Session, day());
        let observed = guard.snapshot();
        guard.claim(observed.client_id.as_deref(), "remounted").unwrap(); guard.snapshot() };
    // The remounted consumer uses only recovered wire ID/revision and the generic
    // server-owned Retry action. It retains neither the old local action nor policy.
    let recovered_id = recovered.active.as_ref().unwrap().id.clone();
    let retry: ReminderAction = serde_json::from_str("\"retry\"").unwrap();
    crate::commands_reminders::acknowledge_with(&state, "remounted", &recovered_id, recovered.revision, retry,
        |_| panic!("retry cannot apply policy twice"), || Ok(())).unwrap();
    assert_eq!(applications.get(), 1);
    assert!(policy.borrow().is_snoozed_at(now));
    let next = state.0.lock().unwrap().snapshot(); let b = next.active.unwrap().id;
    assert_ne!(b, recovered_id);
    crate::commands_reminders::acknowledge_with(&state, "remounted", &b, next.revision, ReminderAction::Dismiss,
        |_| Ok(()), || Ok(())).unwrap();
    assert!(state.0.lock().unwrap().snapshot().active.is_none());
}

#[test]
fn round3_remount_retry_after_snooze_persistence_failure() {
    remount_retry_without_local_action(ReminderAction::Snooze);
}

#[test]
fn round3_remount_retry_after_dismiss_persistence_failure() {
    remount_retry_without_local_action(ReminderAction::Dismiss);
}

#[test]
fn round3_generic_retry_keeps_no_reservation_busy_client_revision_and_id_guards() {
    let state = state_with_message();
    let initial = state.0.lock().unwrap().snapshot(); let id = initial.active.unwrap().id;
    let attempt = |client: &str, id: &str, revision| crate::commands_reminders::acknowledge_with(
        &state, client, id, revision, ReminderAction::Retry,
        |_| panic!("generic Retry cannot apply policy"), || Ok(()));
    assert_eq!(attempt("first", &id, initial.revision).unwrap_err(), "no_pending_persistence");
    crate::commands_reminders::acknowledge_with(&state, "first", &id, initial.revision, ReminderAction::Dismiss,
        |_| Ok(()), || {
            assert_eq!(attempt("first", &id, initial.revision).unwrap_err(), "acknowledgement_in_progress");
            Err("reminder_persistence_failed".into())
        }).unwrap_err();
    let recovered = { let mut guard = state.0.lock().unwrap();
        guard.claim(Some("first"), "replacement").unwrap(); guard.snapshot() };
    assert_eq!(attempt("first", &id, recovered.revision).unwrap_err(), "stale_reminder");
    assert_eq!(attempt("replacement", &id, initial.revision).unwrap_err(), "stale_reminder");
    assert_eq!(attempt("replacement", "old-message", recovered.revision).unwrap_err(), "stale_reminder");
    attempt("replacement", &id, recovered.revision).unwrap();
}

#[cfg(windows)]
#[test]
fn native_adapter_rehides_same_owned_hwnd_after_repeated_no_activate_reminder_shows() { unsafe {
    use windows::core::PCSTR;
    use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::*};
    struct Owned(HWND);
    impl Drop for Owned { fn drop(&mut self) { unsafe { let _ = DestroyWindow(self.0); } } }
    assert!(crate::commands_reminders::show_owned_main_hwnd(HWND::default()).is_err());
    let before = GetForegroundWindow();
    let owned = Owned(CreateWindowExA(WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
        PCSTR(c"STATIC".as_ptr() as _), PCSTR::null(), WS_POPUP,
        -30000, -30000, 100, 20, None, None, None, None).unwrap());
    assert!(!IsWindowVisible(owned.0).as_bool());
    crate::commands_reminders::show_owned_main_hwnd(owned.0).unwrap();
    assert!(IsWindowVisible(owned.0).as_bool());
    crate::commands::hide_owned_main_hwnd_handle(owned.0).unwrap();
    assert!(!IsWindowVisible(owned.0).as_bool());
    crate::commands_reminders::show_owned_main_hwnd(owned.0).unwrap();
    assert!(IsWindowVisible(owned.0).as_bool());
    crate::commands::hide_owned_main_hwnd_handle(owned.0).unwrap();
    assert!(!IsWindowVisible(owned.0).as_bool());
    assert_eq!(GetForegroundWindow(), before);
} }
