//! Regression tests for `commands.rs` lock handling (E019-T01).
//!
//! A panic while any `AppState` mutex is held poisons it for the rest of the
//! process. With `.lock().unwrap()` every later IPC command panics too, so one
//! transient failure takes the whole backend down. `tray_controller.rs` and
//! `remote_server.rs` already recover with
//! `.lock().unwrap_or_else(|e| e.into_inner())`; these tests pin `commands.rs`
//! to the same behaviour.

#![cfg(test)]

use std::sync::{Arc, Mutex};

use crate::session::SessionManager;

/// Source of `commands.rs`, embedded at compile time.
const COMMANDS_SRC: &str = include_str!("commands.rs");

#[test]
fn e019_t01_commands_poison_source_has_no_lock_unwrap() {
    let offenders: Vec<usize> = COMMANDS_SRC
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains(".lock().unwrap()"))
        .map(|(i, _)| i + 1)
        .collect();

    assert!(
        offenders.is_empty(),
        "commands.rs still panics on a poisoned mutex at line(s) {:?}; \
         use .lock().unwrap_or_else(|e| e.into_inner()) instead",
        offenders
    );
}

#[test]
fn e019_t01_commands_poison_source_uses_recovery_pattern() {
    let recoveries = COMMANDS_SRC
        .matches(".lock().unwrap_or_else(|e| e.into_inner())")
        .count();

    assert_eq!(
        recoveries, 14,
        "expected 14 poison-safe lock sites in commands.rs, found {}",
        recoveries
    );
}

#[test]
fn e019_t01_commands_poison_recovery_preserves_session_state() {
    let session = Arc::new(Mutex::new(SessionManager::new()));

    {
        let guard = session.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(guard.state.position_changes, 0);
    }

    // Poison the mutex the way a panicking command would.
    let poisoner = Arc::clone(&session);
    let handle = std::thread::spawn(move || {
        let mut guard = poisoner.lock().unwrap_or_else(|e| e.into_inner());
        guard.state.position_changes = 7;
        panic!("simulated panic while holding the session lock");
    });
    assert!(handle.join().is_err(), "the poisoning thread must panic");
    assert!(session.is_poisoned(), "mutex must be poisoned by now");

    // The pattern used across commands.rs still returns the data.
    let guard = session.lock().unwrap_or_else(|e| e.into_inner());
    assert_eq!(
        guard.state.position_changes, 7,
        "recovered guard must expose the state written before the panic"
    );
}

#[test]
fn welcome_test_preview_uses_the_main_reminder_state_and_shared_limiter() {
    use crate::app_reminders::{AppReminderState, ReminderSource};
    use crate::communication_types::{ReminderCategory, ReminderReason};
    use std::sync::atomic::AtomicI64;

    let state = AppReminderState::default();
    let limiter = AtomicI64::new(0);
    let day = chrono::NaiveDate::from_ymd_opt(2026, 10, 28).unwrap();
    let mut publish = |category| {
        let draft = crate::commands_reminders::preview_draft(category);
        state.0.lock().unwrap().publish(draft, ReminderSource::Test, day);
    };

    crate::commands::dispatch_test_notification("welcome", None, &limiter, 100, &mut publish)
        .expect("the Welcome entry is allowed to request a main-window preview");
    let first = state.0.lock().unwrap().snapshot();
    let active = first.active.expect("the preview enters the shared reminder state");
    assert!(active.is_test);
    assert_eq!(active.source, ReminderSource::Test);
    assert_eq!(active.reason, ReminderReason::Preview);
    assert_eq!(active.category, ReminderCategory::Info, "Welcome uses the existing default category");
    assert_eq!(active.elapsed_secs, None);
    assert_eq!(active.limit_secs, None);
    assert!(first.pending.is_none());
    assert_eq!(first.client_id, None, "Welcome does not claim main reminder ownership");

    assert_eq!(
        crate::commands::dispatch_test_notification("main", Some(ReminderCategory::Warning), &limiter, 159, &mut publish),
        Err("Rate limited: wait before testing again".into()),
        "Welcome and Settings share the same 60-second limiter"
    );
    crate::commands::dispatch_test_notification("main", Some(ReminderCategory::Warning), &limiter, 160, &mut publish)
        .expect("the existing main Settings command can use the same seam");
    let current = state.0.lock().unwrap().snapshot();
    assert_eq!(current.active.unwrap().category, ReminderCategory::Info);
    let queued = current.pending.expect("the current main reminder is not replaced");
    assert!(queued.is_test);
    assert_eq!(queued.category, ReminderCategory::Warning);

    assert_eq!(
        crate::commands::dispatch_test_notification("analyst", None, &limiter, 220, &mut publish),
        Err("test_notification_window_not_allowed".into())
    );
}
