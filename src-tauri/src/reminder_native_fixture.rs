//! Opt-in native scenario only. No product setup, lifecycle, workers or OS channels.
use std::{fs, io::Write, path::{Path, PathBuf}, sync::{Arc, Mutex, atomic::{AtomicBool, AtomicI64, AtomicU32, AtomicU8, Ordering}}, time::Duration};
use serde_json::{json, Value};
use tauri::{Manager, Runtime};
use crate::{app_reminders::{AppReminderSnapshot, AppReminderState, ReminderAction}, commands::AppState};

pub const APP_ID: &str = "io.zntl.moveup.reminder-native-test";
pub type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Role { Input, Reminder }
impl Role {
    pub fn parse(value: &str) -> Result<Self> { match value { "input" => Ok(Self::Input), "reminder" => Ok(Self::Reminder), _ => Err("invalid_role".into()) } }
    pub fn name(self) -> &'static str { if self == Self::Input { "input" } else { "reminder" } }
    pub fn label(self) -> &'static str { if self == Self::Input { "input-target" } else { "main" } }
}
struct Fixture {
    root: PathBuf, role: Role, ready: AtomicBool, round: AtomicU8,
    focus_losses: AtomicU32, first_engagement: AtomicI64,
    expected: Mutex<Option<AppReminderSnapshot>>, closed: tokio::sync::Notify,
}
pub fn no_lifecycle<R: Runtime>(app: &tauri::AppHandle<R>) -> Result<()> {
    if app.try_state::<Arc<crate::app_lifecycle::AppLifecycle>>().is_some() { return Err("lifecycle_forbidden".into()); }
    Ok(()) // record_checkpoint_result returns before notify::show when this state is absent.
}
fn guard(app: &tauri::AppHandle, role: Role) -> Result<()> {
    no_lifecycle(app)?;
    if app.config().identifier != APP_ID || app.state::<Fixture>().role != role { return Err("fixture_identity_mismatch".into()); }
    Ok(())
}
fn write_new(root: &Path, name: &str, value: &Value) -> Result<()> {
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(root.join(name)).map_err(|_| "evidence_path_exists_or_unwritable")?;
    file.write_all(&serde_json::to_vec_pretty(value).map_err(|_| "evidence_json")?).map_err(|_| "evidence_write")?;
    file.sync_all().map_err(|_| "evidence_sync".into())
}
fn read_json(root: &Path, name: &str) -> Result<Value> {
    let path = root.join(name);
    let metadata = fs::symlink_metadata(&path).map_err(|_| "evidence_missing")?;
    use std::os::windows::fs::MetadataExt;
    if !metadata.is_file() || metadata.file_attributes() & 0x400 != 0 || metadata.len() > 65536 { return Err("evidence_path_refused".into()); }
    serde_json::from_slice(&fs::read(path).map_err(|_| "evidence_read")?).map_err(|_| "evidence_json".into())
}
fn hwnd_value(window: &tauri::WebviewWindow) -> Result<u64> { Ok(window.hwnd().map_err(|_| "window_handle")?.0 as usize as u64) }
fn window_pid(hwnd: u64) -> Result<u32> {
    use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::{IsWindow, GetWindowThreadProcessId}};
    let hwnd = HWND(hwnd as usize as *mut _);
    if !unsafe { IsWindow(Some(hwnd)) }.as_bool() { return Err("invalid_owned_window".into()); }
    let mut pid = 0; unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)); } Ok(pid)
}
fn foreground() -> Value {
    let hwnd = unsafe { windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow() }.0 as usize as u64;
    json!({"hwnd": hwnd, "pid": window_pid(hwnd).ok(), "at_ms": chrono::Utc::now().timestamp_millis()})
}
fn visible(window: &tauri::WebviewWindow) -> Result<bool> {
    Ok(unsafe { windows::Win32::UI::WindowsAndMessaging::IsWindowVisible(window.hwnd().map_err(|_| "window_handle")?) }.as_bool())
}
async fn ui_sample(app: &tauri::AppHandle) -> Result<Value> {
    let (tx, rx) = tokio::sync::oneshot::channel(); let handle = app.clone();
    app.run_on_main_thread(move || {
        let result: Result<Value> = (|| { let main = handle.get_webview_window("main").ok_or("main_missing")?;
            Ok(json!({"foreground": foreground(), "main_visible": visible(&main)?, "main_hwnd": hwnd_value(&main)?})) })();
        let _ = tx.send(result);
    }).map_err(|_| "sample_dispatch")?;
    tokio::time::timeout(Duration::from_secs(8), rx).await.map_err(|_| "sample_timeout")?.map_err(|_| "sample_reply")?
}
// Native callbacks only update bounded counters. No disk, scripts, locks or focus restoration.
pub fn focus_event(app: &tauri::AppHandle, focused: bool) {
    let fixture = app.state::<Fixture>();
    if fixture.role == Role::Input && !focused { fixture.focus_losses.fetch_add(1, Ordering::Relaxed); }
    if fixture.role == Role::Reminder && focused && fixture.round.load(Ordering::Acquire) > 0 {
        let _ = fixture.first_engagement.compare_exchange(0, chrono::Utc::now().timestamp_millis(), Ordering::AcqRel, Ordering::Acquire);
    }
}
pub fn builder(root: PathBuf, role: Role) -> tauri::Builder<tauri::Wry> {
    let builder = tauri::Builder::default().manage(Fixture { root, role, ready: AtomicBool::new(false),
        round: AtomicU8::new(0), focus_losses: AtomicU32::new(0), first_engagement: AtomicI64::new(0),
        expected: Mutex::new(None), closed: tokio::sync::Notify::new() });
    if role == Role::Input {
        builder.invoke_handler(tauri::generate_handler![fixture_input_receipt, fixture_quit])
    } else {
        builder.plugin(tauri_plugin_store::Builder::default().build()).invoke_handler(tauri::generate_handler![
            crate::commands_reminders::get_app_reminder_snapshot, crate::commands_reminders::claim_app_reminder_client,
            crate::commands_reminders::set_app_reminder_flow, crate::commands_reminders::acknowledge_app_reminder,
            crate::commands::hide_main_window, crate::commands_profiles::get_active_profiles,
            fixture_frontend_ready, fixture_verify_close, fixture_quit])
    }
}
pub fn prepare(app: &tauri::AppHandle) -> Result<()> {
    no_lifecycle(app)?;
    let fixture = app.state::<Fixture>();
    if fixture.role == Role::Input { return Ok(()); }
    let root = &fixture.root;
    write_new(root, "settings.json", &json!({}))?;
    crate::app_store::initialize_at(app, &root.join("settings.json"))?;
    let conn = Arc::new(crate::serial::ConnectionState::default());
    *conn.connected_port.lock().unwrap() = Some("isolated-synthetic-observation".into()); // No serial API is called.
    let logs = root.join("logs");
    app.manage(crate::Loggers { snapshot: Arc::new(crate::snapshot_logger::SnapshotLogger::new(logs.clone())),
        event: Arc::new(crate::event_logger::EventLogger::new(logs)) });
    app.manage(AppReminderState::default());
    app.manage(AppState { conn, session: Arc::new(Mutex::new(crate::session::SessionManager::new())),
        db: Arc::new(Mutex::new(Some(rusqlite::Connection::open_in_memory().map_err(|_| "private_db")?))),
        config: Arc::new(Mutex::new(Some(crate::config::AppConfig::default()))),
        overlay: Arc::new(crate::overlay_renderer::OverlayRenderer { state: Arc::new(Mutex::new(Default::default())) }),
        comm_policy: Arc::new(Mutex::new(crate::communication_policy::CommunicationPolicy::new(Default::default(), Default::default()))),
        alert_popup: Arc::new(Mutex::new(crate::alert_popup::AlertPopup::new())), ws_tx: tokio::sync::broadcast::channel(1).0,
        today_cache: Arc::new(Mutex::new(Default::default())), relay: Arc::new(Mutex::new(Default::default())),
        relay_handle: Arc::new(Mutex::new(None)) });
    Ok(())
}
pub async fn native_ready(app: &tauri::AppHandle, ownership: Value) -> Result<()> {
    let role = app.state::<Fixture>().role; guard(app, role)?;
    write_new(&app.state::<Fixture>().root, &format!("identity-{}.json", role.name()), &ownership)?;
    if role == Role::Reminder { scenario(app).await?; }
    Ok(())
}
#[tauri::command]
fn fixture_frontend_ready(window: tauri::WebviewWindow) -> Result<()> {
    guard(window.app_handle(), Role::Reminder)?;
    if window.label() != "main" { return Err("main_only".into()); }
    window.state::<Fixture>().ready.store(true, Ordering::Release); Ok(())
}
#[tauri::command]
async fn fixture_input_receipt(window: tauri::WebviewWindow, round: u8, phase: String, keys: u32, pointers: u32, input_events: u32, length: u32) -> Result<()> {
    let app = window.app_handle(); guard(app, Role::Input)?;
    if window.label() != "input-target" || !(1..=2).contains(&round) || !["baseline", "during", "after"].contains(&phase.as_str())
        || keys == 0 || keys > 512 || pointers == 0 || pointers > 512 || input_events == 0 || input_events > 512 || length == 0 || length > 128 { return Err("actual_input_required".into()); }
    let fixture = app.state::<Fixture>(); let current = foreground(); let hwnd = hwnd_value(&window)?;
    if current["hwnd"] != hwnd || window_pid(hwnd)? != std::process::id() { return Err("input_target_not_foreground".into()); }
    read_json(&fixture.root, "identity-input.json")?; // Exists only after actual private profile/ownership verification.
    if round == 2 && phase == "baseline" { read_json(&fixture.root, "closed-1.json")?; }
    if phase != "baseline" { read_json(&fixture.root, &format!("arrival-{round}.json"))?; }
    write_new(&fixture.root, &format!("input-{round}-{phase}.json"), &json!({"round":round,"phase":phase,
        "foreground":current,"keys":keys,"pointers":pointers,"input_events":input_events,"length":length,
        "focus_losses":fixture.focus_losses.load(Ordering::Acquire)}))
}
fn input_identity(root: &Path) -> Result<Value> {
    let identity = read_json(root, "identity-input.json")?;
    let pid = identity["pid"].as_u64().ok_or("input_pid")? as u32;
    let hwnd = identity["hwnd"].as_u64().ok_or("input_hwnd")?;
    if pid == std::process::id() || window_pid(hwnd)? != pid || identity["app_id"] != APP_ID
        || PathBuf::from(identity["executable"].as_str().ok_or("input_executable")?).canonicalize().map_err(|_| "input_executable")?
            != std::env::current_exe().map_err(|_| "own_executable")?.canonicalize().map_err(|_| "own_executable")?
        || PathBuf::from(identity["profile"].as_str().ok_or("input_profile")?).canonicalize().map_err(|_| "input_profile")?
            != root.join("profiles/input").canonicalize().map_err(|_| "expected_input_profile")? { return Err("input_process_identity_refused".into()); }
    Ok(identity)
}
async fn baseline(app: &tauri::AppHandle, round: u8) -> Result<Value> {
    let fixture = app.state::<Fixture>();
    for _ in 0..600 { // Finite scenario wait, not a product poller; 120 seconds per operator phase.
        if fixture.ready.load(Ordering::Acquire) && fixture.root.join(format!("input-{round}-baseline.json")).exists() {
            return read_json(&fixture.root, &format!("input-{round}-baseline.json"));
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    Err("baseline_or_frontend_timeout".into())
}
fn dispatch(app: &tauri::AppHandle) -> Result<()> {
    guard(app, Role::Reminder)?;
    let state = app.state::<AppState>(); let now = chrono::Utc::now();
    let (signals, draft) = {
        let mut session = state.session.lock().unwrap();
        // Explicitly synthetic PRIVATE posture transition/new sitting occurrence, not live history.
        session.state.state = crate::session_types::DeskState::Sitting;
        session.state.sitting_started = Some(now); session.state.last_tick_ts = Some(now);
        session.state.sitting_seconds = session.state.session_limit_secs + 60;
        session.history_span_started = Some(now); session.history_last_reading = Some(now);
        let input = session.policy_input(now, true);
        if !input.observation_fresh { return Err("synthetic_observation_not_fresh".into()); }
        let mut policy = state.comm_policy.lock().unwrap(); policy.on_position_changed();
        let signals = policy.evaluate_with_stand_limit_fired_at(&input, session.state.session_limit_secs, 0, false, std::time::Instant::now());
        let draft = policy.take_reminder().ok_or("policy_produced_no_reminder")?;
        (signals, draft)
    };
    let signal = signals.notify.as_ref().ok_or("policy_produced_no_signal")?;
    let logs = app.state::<crate::Loggers>();
    crate::notification_service::NotificationService::dispatch_policy_signal(app, signal, Some(&draft), Some(&logs.event), &state.alert_popup);
    Ok(()) // Actual publish/reconcile/announce queues SW_SHOWNOACTIVATE; no manual Show here.
}
async fn scenario(app: &tauri::AppHandle) -> Result<()> {
    let fixture = app.state::<Fixture>();
    for round in 1..=2 {
        let baseline = baseline(app, round).await?; let target = input_identity(&fixture.root)?;
        tokio::time::sleep(Duration::from_secs(3)).await; // Leave time to keep typing in the independent target.
        let before = ui_sample(app).await?;
        if before["foreground"]["hwnd"] != target["hwnd"] { return Err("foreground_baseline_changed".into()); }
        fixture.first_engagement.store(0, Ordering::Release); fixture.round.store(round, Ordering::Release);
        dispatch(app)?;
        // FIFO UI sample brackets the actual production announce callback, before any Record-phase click.
        let after = ui_sample(app).await?;
        let snapshot = crate::commands_reminders::snapshot(app);
        let message = snapshot.active.as_ref().ok_or("automatic_message_missing")?;
        if round == 2 && read_json(&fixture.root, "closed-1.json")?["message"] == message.id { return Err("second_round_reused_message".into()); }
        if message.is_test || !after["main_visible"].as_bool().unwrap_or(false)
            || after["foreground"]["hwnd"] != target["hwnd"] || fixture.first_engagement.load(Ordering::Acquire) != 0 { return Err("automatic_foreground_or_visibility_failed".into()); }
        *fixture.expected.lock().unwrap() = Some(snapshot.clone());
        write_new(&fixture.root, &format!("arrival-{round}.json"), &json!({"round":round,"message":message.id,
            "is_test":message.is_test,"baseline":baseline,"before_dispatch":before,"after_presentation_queue":after,"target":target}))?;
        tokio::time::timeout(Duration::from_secs(180), fixture.closed.notified()).await.map_err(|_| "real_x_timeout")?;
    }
    write_new(&fixture.root, "complete.json", &json!({"rounds":2,"result":"isolated_backend_and_input_evidence_received",
        "original_incident":"UNKNOWN","installed_production":"NOT_CHECKED","all_applications":"NOT_CHECKED"}))?;
    Ok(())
}
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct HookReceipt {
    session_id: String, client_id: Option<String>, revision: u32,
    ready: bool, busy: bool, close_error: bool, action_error: bool,
    active: bool, pending: bool, protected: bool, rendered_alerts: u32,
}
#[tauri::command]
async fn fixture_verify_close(window: tauri::WebviewWindow, trusted_pointer: bool, hook: HookReceipt) -> Result<()> {
    let app = window.app_handle(); guard(app, Role::Reminder)?;
    if window.label() != "main" || !trusted_pointer { return Err("real_pointer_x_required".into()); }
    let fixture = app.state::<Fixture>(); let round = fixture.round.load(Ordering::Acquire);
    let expected = fixture.expected.lock().unwrap().clone().ok_or("no_published_round")?;
    let message = expected.active.as_ref().ok_or("round_message")?;
    let arrival = read_json(&fixture.root, &format!("arrival-{round}.json"))?;
    let engagement = fixture.first_engagement.load(Ordering::Acquire);
    for phase in ["during", "after"] {
        let receipt = read_json(&fixture.root, &format!("input-{round}-{phase}.json"))?;
        if receipt["focus_losses"] != arrival["baseline"]["focus_losses"]
            || receipt["foreground"]["hwnd"] != arrival["target"]["hwnd"]
            || engagement == 0 || receipt["foreground"]["at_ms"].as_i64().ok_or("receipt_time")? >= engagement { return Err("intervening_focus_loss_or_late_receipt".into()); }
    }
    let current = crate::commands_reminders::snapshot(app);
    if !hook.ready || hook.busy || hook.close_error || hook.action_error || hook.active || hook.pending || hook.protected || hook.rendered_alerts != 0
        || hook.session_id != current.session_id || hook.client_id != current.client_id || hook.revision != current.revision {
        return Err("settled_react_close_state_not_clear".into());
    }
    if current.session_id != expected.session_id || current.client_id != expected.client_id
        || current.active.is_some() || current.pending.is_some() || current.protected_flow.is_some() || visible(&window)? { return Err("real_x_did_not_hide_same_context".into()); }
    let bytes = fs::read(fixture.root.join("settings.json")).map_err(|_| "store_disk_read")?;
    let disk: Value = serde_json::from_slice(&bytes).map_err(|_| "store_disk_json")?;
    let saved = crate::session_persistence::PersistedEngineState::from_stored_value(disk["persisted_session_state"].clone()).ok_or("store_snapshot_missing")?;
    let policy = saved.notification_policy.ok_or("store_policy_missing")?;
    if policy.snooze_index != 1 || policy.snoozed_until_utc.is_none() { return Err("dismiss_not_exactly_once_on_disk".into()); }
    // Deliberately replay the OLD ACK through the same actual production command; never the hook's hide Retry.
    let duplicate = crate::commands_reminders::acknowledge_app_reminder(window.clone(), expected.client_id.clone().ok_or("client_missing")?,
        message.id.clone(), expected.revision, ReminderAction::Dismiss);
    if duplicate.err().as_deref() != Some("stale_reminder") { return Err("duplicate_ack_not_rejected".into()); }
    tokio::time::sleep(Duration::from_millis(300)).await;
    if fs::read(fixture.root.join("settings.json")).map_err(|_| "store_reread")? != bytes || visible(&window)? { return Err("duplicate_changed_disk_or_reopened".into()); }
    let log = fs::read_to_string(fixture.root.join("logs").join(chrono::Local::now().format("%Y-%m-%d").to_string()).join("events.log")).map_err(|_| "ack_log_missing")?;
    if log.len() > 65536 { return Err("fixture_log_limit".into()); }
    let successes = log.lines().filter_map(|line| line.split_once("APP_REMINDER_ACK ")).filter_map(|(_, text)| serde_json::from_str::<Value>(text).ok())
        .filter(|event| event["reminder"] == message.id && event["is_test"] == false && event["policy_apply"] == "applied" && event["persistence"] == "succeeded" && event["succeeded"] == true).count();
    if successes != 1 { return Err("matching_real_ack_evidence_missing_or_duplicated".into()); }
    let hides = log.lines().filter_map(|line| line.split_once("MAIN_WINDOW_HIDE ")).filter_map(|(_, text)| serde_json::from_str::<Value>(text).ok())
        .filter(|event| event["context"]["reminder"] == message.id && event["context"]["session"] == current.session_id
            && event["context"]["client"].as_str() == current.client_id.as_deref() && event["context"]["revision"] == current.revision
            && event["succeeded"] == true && event["stage"] == "native" && event["reason"] == "hidden").count();
    if hides != 1 { return Err("matching_native_hide_missing_or_repeated".into()); }
    write_new(&fixture.root, &format!("closed-{round}.json"), &json!({"message":message.id,"session":current.session_id,
        "client":current.client_id,"trusted_pointer_x":true,"first_engagement_ms":engagement,"native_hidden":true,"settled_react":hook,
        "persisted_snooze_index":policy.snooze_index,"successful_real_ack_records":successes,"matching_native_hides":hides,"duplicate":"stale_reminder","duplicate_disk_unchanged":true}))?;
    fixture.closed.notify_one(); Ok(())
}
#[tauri::command]
fn fixture_quit(window: tauri::WebviewWindow) { window.app_handle().exit(0); }

#[cfg(test)]
mod tests {
    #[test]
    fn missing_lifecycle_keeps_real_checkpoint_error_out_of_notification_branch() {
        let app = tauri::test::mock_builder().build(tauri::test::mock_context(tauri::test::noop_assets())).unwrap();
        assert!(super::no_lifecycle(app.handle()).is_ok());
        let result = crate::session_persistence::run_checkpoint_attempt(app.handle(), chrono::Utc::now(),
            || Err("private_disposable_store_failed".into()));
        assert_eq!(result.unwrap_err(), "private_disposable_store_failed");
        assert!(super::no_lifecycle(app.handle()).is_ok());
    }
}
