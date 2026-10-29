//! Main-only IPC and native presentation adapter. No lock is held across native calls.
use tauri::{Emitter, Manager};
use crate::app_reminders::{AppReminderSnapshot, AppReminderState, ProtectedFlow, ReminderAction, ReminderSource};
use crate::communication_types::{ReminderCategory, ReminderDraft, ReminderReason};

fn main_only(window: &tauri::WebviewWindow) -> Result<(), String> {
    if window.label() == "main" { Ok(()) } else { Err("main_window_only".into()) }
}

pub(crate) fn snapshot(app: &tauri::AppHandle) -> AppReminderSnapshot {
    app.state::<AppReminderState>().0.lock().unwrap_or_else(|error| error.into_inner()).snapshot()
}

/// Snapshot and events share a revision; consumers ignore older responses/events.
#[tauri::command]
pub fn get_app_reminder_snapshot(window: tauri::WebviewWindow) -> Result<AppReminderSnapshot, String> {
    main_only(&window)?;
    Ok(snapshot(window.app_handle()))
}

#[tauri::command]
pub fn claim_app_reminder_client(window: tauri::WebviewWindow, previous_client_id: Option<String>, client_id: String) -> Result<AppReminderSnapshot, String> {
    main_only(&window)?;
    window.state::<AppReminderState>().0.lock().unwrap_or_else(|error| error.into_inner())
        .claim(previous_client_id.as_deref(), &client_id)?;
    announce(window.app_handle());
    Ok(snapshot(window.app_handle()))
}

#[tauri::command]
pub fn set_app_reminder_flow(window: tauri::WebviewWindow, client_id: String, sequence: u32, flow: Option<ProtectedFlow>) -> Result<AppReminderSnapshot, String> {
    main_only(&window)?;
    window.state::<AppReminderState>().0.lock().unwrap_or_else(|error| error.into_inner())
        .set_flow(&client_id, sequence, flow)?;
    announce(window.app_handle());
    Ok(snapshot(window.app_handle()))
}

#[tauri::command]
pub fn acknowledge_app_reminder(window: tauri::WebviewWindow, client_id: String, id: String, revision: u32, action: ReminderAction) -> Result<AppReminderSnapshot, String> {
    main_only(&window)?;
    acknowledge(window.app_handle(), &client_id, &id, revision, action)?;
    Ok(snapshot(window.app_handle()))
}

pub(crate) fn acknowledge(app: &tauri::AppHandle, client: &str, id: &str, revision: u32, action: ReminderAction) -> Result<(), String> {
    let reminders = app.state::<AppReminderState>();
    let result = acknowledge_with(&reminders, client, id, revision, action, |action| {
        let state = app.state::<crate::commands::AppState>();
        let _session = state.session.lock().unwrap_or_else(|error| error.into_inner());
        let mut policy = state.comm_policy.lock().unwrap_or_else(|error| error.into_inner());
        match action {
            ReminderAction::Dismiss => policy.dismiss_at(std::time::Instant::now()),
            ReminderAction::Snooze => {
                let minutes = policy.reminder_snooze_minutes()?;
                policy.snooze_for_at(minutes, std::time::Instant::now());
            }
            ReminderAction::Retry => return Err("no_pending_persistence".into()),
        }
        Ok(())
    }, || crate::session_persistence::save_via_app_at(app, chrono::Utc::now(), std::time::Instant::now())
        .map_err(|_| "reminder_persistence_failed".into()));
    reconcile(app);
    announce(app);
    result
}

/// Actual command orchestration with disposable persistence/action seams for regression tests.
pub(crate) fn acknowledge_with(state: &AppReminderState, client: &str, id: &str, revision: u32,
    action: ReminderAction, apply: impl FnOnce(ReminderAction) -> Result<(), String>,
    persist: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    let (is_test, needs_apply) = state.0.lock().unwrap_or_else(|error| error.into_inner())
        .begin_ack(client, id, revision, action)?;
    let result = if is_test { Ok(()) } else {
        let applied = if needs_apply { apply(action) } else { Ok(()) };
        match applied {
            Err(error) => Err(error),
            Ok(()) => {
                state.0.lock().unwrap_or_else(|error| error.into_inner()).ack_applied(id);
                persist()
            }
        }
    };
    state.0.lock().unwrap_or_else(|error| error.into_inner()).finish_ack(id, result)
}

pub(crate) fn publish(app: &tauri::AppHandle, draft: ReminderDraft, source: ReminderSource) {
    let Some(reminders) = app.try_state::<AppReminderState>() else { return; };
    let changed = {
        let mut guard = reminders.0.lock().unwrap_or_else(|error| error.into_inner());
        let before = guard.snapshot().revision;
        let day = draft.observation_day.unwrap_or_else(|| chrono::Local::now().date_naive());
        guard.publish(draft, source, day);
        before != guard.snapshot().revision
    };
    if !changed { return; }
    reconcile(app);
    announce(app);
}

/// Retire late deliveries and queued reasons made obsolete by an acknowledgement.
fn reconcile(app: &tauri::AppHandle) {
    let state = app.state::<crate::commands::AppState>();
    let connected = state.conn.connected_port.lock().unwrap_or_else(|error| error.into_inner()).is_some();
    let now = chrono::Utc::now();
    let session = state.session.lock().unwrap_or_else(|error| error.into_inner());
    let input = session.policy_input(now, connected);
    let facts = session.notification_facts(now);
    let snapshot = session.snapshot();
    let policy = state.comm_policy.lock().unwrap_or_else(|error| error.into_inner());
    app.state::<AppReminderState>().0.lock().unwrap_or_else(|error| error.into_inner()).reconcile(|entry|
        policy.reminder_obsolete_reason(&entry.draft, entry.day, &input, &facts,
            snapshot.session_limit_secs, snapshot.standing_session_secs, std::time::Instant::now()));
}

pub(crate) fn preview(app: &tauri::AppHandle, category: ReminderCategory) {
    publish(app, preview_draft(category), ReminderSource::Test);
}

pub(crate) fn preview_draft(category: ReminderCategory) -> ReminderDraft {
    static NEXT_PREVIEW: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    ReminderDraft { occurrence: NEXT_PREVIEW.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        reason: ReminderReason::Preview, category,
        title: "Test reminder".into(), body: "This is a test reminder. Your ergonomic session is unchanged.".into(),
        elapsed_secs: None, limit_secs: None, position_anchor: None, observation_day: None,
    }
}

pub(crate) fn retain_on_focus_loss(app: &tauri::AppHandle) -> bool {
    app.try_state::<AppReminderState>().is_some_and(|state|
        state.0.lock().unwrap_or_else(|error| error.into_inner()).retain_on_focus_loss())
}

/// Called after policy reconciliation or mutation, with all business locks released.
pub(crate) fn announce(app: &tauri::AppHandle) {
    let current = snapshot(app);
    if let Some(window) = app.get_webview_window("main") {
        if window.emit(crate::desk_events::DESK_APP_REMINDER, &current).is_err() {
            record_presentation(app, current.revision, Err("reminder_event_failed".into()));
        }
    }
    if current.active.is_none() || current.protected_flow.is_some() { return; }
    let handle = app.clone();
    let revision = current.revision;
    if app.run_on_main_thread(move || {
        // A delayed show must never reopen an acknowledged/retired message or a protected flow.
        let latest = snapshot(&handle);
        if latest.revision != revision || latest.active.is_none() || latest.protected_flow.is_some() { return; }
        let result = handle.get_webview_window("main").ok_or_else(|| "main_window_missing".into())
            .and_then(|window| show_without_activation(&window));
        record_presentation(&handle, revision, result);
    }).is_err() {
        record_presentation(app, revision, Err("main_thread_unavailable".into()));
    }
}

fn record_presentation(app: &tauri::AppHandle, revision: u32, result: Result<(), String>) {
    let state = app.state::<AppReminderState>();
    let updated = {
        let mut guard = state.0.lock().unwrap_or_else(|error| error.into_inner());
        guard.record_presentation(revision, result);
        guard.snapshot()
    };
    if updated.revision != revision {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.emit(crate::desk_events::DESK_APP_REMINDER, updated);
        }
    }
}

fn show_without_activation(window: &tauri::WebviewWindow) -> Result<(), String> {
    #[cfg(windows)] { unsafe {
        let hwnd = window.hwnd().map_err(|_| "main_handle_unavailable")?;
        let hwnd = windows::Win32::Foundation::HWND(hwnd.0 as _);
        show_owned_main_hwnd(hwnd)
    } }
    #[cfg(not(windows))] { let _ = window; Err("no_focus_presentation_unsupported".into()) }
}

#[cfg(windows)]
pub(crate) unsafe fn show_owned_main_hwnd(hwnd: windows::Win32::Foundation::HWND) -> Result<(), String> {
    use windows::Win32::UI::WindowsAndMessaging::{GetWindowThreadProcessId, IsWindow, IsWindowVisible, ShowWindow, SW_SHOWNOACTIVATE};
    let mut owner = 0;
    if !IsWindow(Some(hwnd)).as_bool() { return Err("main_handle_invalid".into()); }
    GetWindowThreadProcessId(hwnd, Some(&mut owner));
    if owner != std::process::id() { return Err("main_handle_not_owned".into()); }
    // The return value is previous visibility, not success.
    let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    if !IsWindowVisible(hwnd).as_bool() { return Err("main_show_not_visible".into()); }
    Ok(()) // Visibility only; this does not certify painting or focus behavior.
}
