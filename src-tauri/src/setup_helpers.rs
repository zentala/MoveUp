//! setup_helpers.rs — App setup, window positioning, device notifications,
//! remote display wiring, and graceful shutdown persistence.

use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::path::Path;
use std::sync::Mutex;

use log::{info, warn};
use tauri::{AppHandle, Listener, Manager};
use window_vibrancy::apply_acrylic;

#[cfg(not(debug_assertions))]
use tauri_plugin_autostart::ManagerExt;

use crate::commands::AppState;
use crate::desk_events::{
    DESK_DAILY_RESET, DESK_DEVICE_CONNECTED, DESK_DEVICE_LOST,
};
use crate::ws_broadcaster::{self, DisplayEvent};

/// Main app setup — called from the Tauri `.setup()` closure in `lib.rs`.
///
/// Initialises logging, loads config, sets up tray, controllers, and serial scan.
pub fn perform_app_setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_data_dir)?;
    let lifecycle = app.state::<Arc<crate::app_lifecycle::AppLifecycle>>().inner().clone();
    let previous = match lifecycle.claim_primary(&app_data_dir) {
        Ok(previous) => previous,
        // A second activation must never replace the owner's run record.
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return Err(e.into()),
        Err(_) => {
            lifecycle.diagnostics.record(&lifecycle.identity.run_id,
                crate::startup_diagnostics::Diagnostic::StateLockFailed);
            crate::app_lifecycle_store::PreviousRun::InterruptedUnknown
        }
    };
    info!("App data dir: {:?}", app_data_dir);

    crate::app_store::initialize(app.handle()).map_err(std::io::Error::other)?;
    crate::db_backup::backup_database(&app_data_dir);
    info!("DB path: {}", app_data_dir.join("desk.db").display());

    let logs_dir = app_data_dir.join("logs");
    let snapshot_logger = Arc::new(crate::snapshot_logger::SnapshotLogger::new(logs_dir.clone()));
    snapshot_logger.cleanup_old_logs(7);
    let event_logger = Arc::new(crate::event_logger::EventLogger::new(logs_dir));
    event_logger.log(&format!("START v{}", crate::APP_VERSION));
    lifecycle.attach_logger(event_logger.clone(), previous);
    app.manage(crate::Loggers {
        snapshot: snapshot_logger.clone(),
        event: event_logger.clone(),
    });
    load_profiles(app.handle(), &event_logger);

    ensure_autostart(app.handle(), &event_logger);
    crate::tray::setup_tray(app.handle())?;
    lifecycle.tray_initialized.store(true, std::sync::atomic::Ordering::Release);
    lifecycle.event("tray_ready");
    crate::tray_controller::setup(app.handle());
    crate::tray_signal_exec::start_blink_thread(app.handle().clone());
    position_main_window(app.handle());

    {
        let state: tauri::State<'_, AppState> = app.state();
        if let Some(store) = app.try_state::<crate::app_store::AppStore<tauri::Wry>>() {
            let config = crate::config::AppConfig::load(store.inner());
            let ergo = state.comm_policy.lock().unwrap().ergo_profile().clone();
            let mut session = state.session.lock().unwrap_or_else(|e| e.into_inner());
            *session = crate::session::SessionManager::new_from_config(&config, &ergo);
            // Restore persisted flags and credit-reduced sitting_seconds.
            if let Some(persisted) = crate::session_persistence::PersistedSessionState::load(store.inner()) {
                session.load_persisted_state(&persisted);
            }
            *state.config.lock().unwrap_or_else(|e| e.into_inner()) = Some(config.clone());
            info!("startup: config loaded from store into SessionManager");
            if config.show_welcome_on_startup {
                if let Err(e) = crate::commands_welcome::show_welcome_window(app.handle()) {
                    log::warn!("Failed to show welcome popup: {}", e);
                }
            }
        }
    }

    // Initialize essential persistence during setup, not via a later frontend poll.
    // This reuses the existing schema/load/restore path without a second engine.
    let state = app.state::<AppState>();
    match crate::commands::ensure_initialized(app.handle(), &state) {
        Ok(()) => {
            lifecycle.persistence_initialized.store(true, std::sync::atomic::Ordering::Release);
            lifecycle.event("persistence_ready");
        }
        Err(_) => lifecycle.fail(crate::startup_diagnostics::Diagnostic::SetupFailed),
    }
    crate::readiness_server::setup(app.handle());
    setup_remote_display(app.handle());

    let state: tauri::State<'_, AppState> = app.state();
    crate::serial::scan_and_connect(
        app.handle().clone(),
        state.conn.clone(),
        state.session.clone(),
        state.db.clone(),
        state.config.clone(),
        snapshot_logger,
        event_logger,
    );

    lifecycle.setup_finished();
    Ok(())
}

/// Enables autostart for release builds only, self-heals stale registry paths,
/// and always emits an AUTOSTART line to `events.log` for visibility.
///
/// In debug builds the function is a no-op — dev executables must never
/// be registered as a system startup entry.
#[cfg(debug_assertions)]
pub fn ensure_autostart(
    _app: &AppHandle,
    event_logger: &Arc<crate::event_logger::EventLogger>,
) {
    info!("autostart: skipped (debug build)");
    event_logger.log("AUTOSTART skipped reason=debug_build");
}

/// Enables autostart for release builds only, self-heals stale registry paths,
/// and always emits an AUTOSTART line to `events.log` for visibility.
#[cfg(not(debug_assertions))]
pub fn ensure_autostart(
    app: &AppHandle,
    event_logger: &Arc<crate::event_logger::EventLogger>,
) {
    if crate::supervision::pm3_owns_autostart(|key| std::env::var(key).ok()) {
        event_logger.log("AUTOSTART owner=pm3 direct_registration=skipped");
        return;
    }
    let autostart = app.autolaunch();
    let current_exe = std::env::current_exe().unwrap_or_default();
    let current_path = current_exe.display().to_string();

    let is_enabled = autostart.is_enabled().unwrap_or(false);
    // The autostart plugin registers under the app name from tauri.conf.json
    // (`productName`), so the registry value must be read under that same name.
    let registered_path = read_autostart_registry_path(&app.package_info().name);
    if let (Some(command), Some(home)) = (&registered_path, std::env::var_os("USERPROFILE")) {
        let launcher = std::path::PathBuf::from(home).join(".pm3/releases/launch.mjs");
        if crate::supervision::is_pm3_login_activation(command, &launcher) {
            event_logger.log("AUTOSTART owner=pm3_activation direct_registration=skipped");
            return;
        }
    }

    // [B] Self-heal: if enabled but registered path differs, re-register
    let needs_reregister = is_enabled
        && registered_path
            .as_ref()
            .map_or(true, |p| !paths_equal(exe_from_command_line(p), &current_path));

    if needs_reregister {
        warn!(
            "autostart: stale path detected, re-registering. old={:?} new={}",
            registered_path, current_path
        );
        event_logger.log(&format!(
            "AUTOSTART stale_path old={:?} new={}",
            registered_path, current_path
        ));
        let _ = autostart.disable();
    }

    if !is_enabled || needs_reregister {
        match autostart.enable() {
            Ok(()) => {
                info!("autostart: enabled for {}", current_path);
                event_logger.log(&format!("AUTOSTART enabled path={}", current_path));
            }
            Err(e) => {
                warn!("autostart: failed to enable: {}", e);
                event_logger.log(&format!("AUTOSTART error msg={}", e));
            }
        }
    } else {
        // [C] Always log current status for visibility
        event_logger.log(&format!("AUTOSTART verified path={}", current_path));
    }
}

/// Applies Acrylic blur and positions the main window in the bottom-right corner.
/// Hides the window when launched with `--minimized` (autostart scenario).
pub fn position_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = apply_acrylic(&window, Some((18, 18, 18, 200)));

        if let Ok(Some(monitor)) = window.current_monitor() {
            let mon = monitor.size();
            let win = window.outer_size().unwrap_or_default();
            let x = mon.width as i32 - win.width as i32 - 16;
            let y = mon.height as i32 - win.height as i32 - 56;
            let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
        }

        // Hide window if started with --minimized (autostart scenario)
        if std::env::args().any(|a| a == "--minimized") {
            let _ = window.hide();
            log::info!("autostart: started with --minimized, popup hidden");
        }
    }
}

/// Spawns the remote display HTTP+WS server and wires broadcast listeners.
///
/// The server itself is gated on `remote_lan_enabled` (default `true`). The
/// broadcast listeners are not: they feed `ws_tx`, which the relay client also
/// subscribes to, so turning the LAN off must not silence the relay.
pub fn setup_remote_display(app: &AppHandle) {
    let state: tauri::State<'_, AppState> = app.state();
    // Before the LAN gate on purpose: the two transports are independent, and
    // a desk with the LAN switched off must still reach the relay (E022-T06).
    crate::commands_relay::sync_client(app);
    if !lan_enabled(&state) {
        info!("remote display: LAN server disabled ({}=false)", crate::remote_server::LAN_ENABLED_KEY);
        setup_broadcast_listeners(app);
        return;
    }
    let health: crate::health_source::HealthState = app
        .state::<crate::health_source::HealthState>()
        .inner()
        .clone();
    let ws_tx = state.ws_tx.clone();
    let session = state.session.clone();
    let comm_policy = state.comm_policy.clone();
    let today_cache = state.today_cache.clone();
    let voice = build_voice_state(app, &state);
    let remote_token = crate::remote_auth::token_from_env();
    if remote_token.is_none() {
        log::info!(
            "{} not set — /display/health is closed (503). Set it to enable the phone inlet.",
            crate::remote_auth::TOKEN_ENV
        );
    }
    let port = std::env::var("DESK_REMOTE_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(crate::remote_server::DEFAULT_PORT);
    let lifecycle = app.state::<Arc<crate::app_lifecycle::AppLifecycle>>().inner().clone();
    tauri::async_runtime::spawn(async move {
        // Registration is async, so it happens here rather than at manage()
        // time; the inlet exists before the first request can arrive.
        let health_push = crate::remote_routes_health::register(&health).await;
        let remote_state = crate::remote_server::RemoteState {
            ws_tx,
            session,
            comm_policy,
            today_cache,
            active_clients: Arc::new(AtomicUsize::new(0)),
            health,
            health_push,
            remote_token,
            voice,
        };
        crate::remote_server::start(remote_state, port, lifecycle).await;
    });
    setup_broadcast_listeners(app);
}

/// Whether the user left the LAN display switched on.
///
/// Goes through the serialized config so an older stored config — one written
/// before the toggle existed — reads as enabled rather than as off.
fn lan_enabled(state: &tauri::State<'_, AppState>) -> bool {
    let config = state.config.lock().unwrap_or_else(|e| e.into_inner());
    let value = config.as_ref().and_then(|c| serde_json::to_value(c).ok());
    crate::remote_server::lan_enabled_in(value.as_ref())
}

/// Assembles the voice inlet's dependencies (E021-T06).
///
/// Each leg is resolved independently and may end up `None`: the event logger
/// is absent until `perform_app_setup` has managed it, the database until
/// `ensure_initialized` has opened it, the AI client until the user brings an
/// OpenRouter key. A missing leg degrades that step only — the route still
/// parses intents and acknowledges them.
fn build_voice_state(
    app: &AppHandle,
    state: &tauri::State<'_, AppState>,
) -> crate::remote_server::VoiceState {
    let config = state
        .config
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let ai = crate::voice_ai::VoiceAi::from_env(
        config.as_ref().and_then(|c| c.voice_ai_model.as_deref()),
    );
    if ai.is_none() {
        info!(
            "{} not set — dictated notes are stored and acknowledged, without an AI reply.",
            crate::voice_ai::API_KEY_ENV
        );
    }
    crate::remote_server::VoiceState {
        event_logger: app.try_state::<crate::Loggers>().map(|l| l.event.clone()),
        db: Some(state.db.clone()),
        ai: ai.map(Arc::new),
        webhook_enabled: config.as_ref().is_some_and(|c| c.notify_webhook_enabled),
        webhook_url: config.and_then(|c| c.notify_webhook_url),
    }
}

/// Forwards device/daily-reset events to WebSocket for remote clients.
pub fn setup_broadcast_listeners(app: &AppHandle) {
    // desk:device-connected → broadcast to remote clients
    let handle1 = app.clone();
    app.listen(DESK_DEVICE_CONNECTED, move |event| {
        let ws_tx = handle1.state::<AppState>().ws_tx.clone();
        let port = serde_json::from_str::<serde_json::Value>(event.payload())
            .ok()
            .and_then(|v| v.get("port").and_then(|p| p.as_str()).map(String::from))
            .unwrap_or_default();
        ws_broadcaster::broadcast_event(&ws_tx, &DisplayEvent::DeviceConnected { port });
    });

    // desk:device-lost → broadcast to remote clients
    let handle2 = app.clone();
    app.listen(DESK_DEVICE_LOST, move |_event| {
        let ws_tx = handle2.state::<AppState>().ws_tx.clone();
        ws_broadcaster::broadcast_event(&ws_tx, &DisplayEvent::DeviceLost);
    });

    // desk:daily-reset → broadcast + clear today_cache
    let handle3 = app.clone();
    app.listen(DESK_DAILY_RESET, move |_event| {
        let state = handle3.state::<AppState>();
        ws_broadcaster::broadcast_event(&state.ws_tx, &DisplayEvent::DailyReset);
        // Clear cached today summary
        let mut cache = state.today_cache.lock().unwrap_or_else(|e| e.into_inner());
        *cache = crate::db::TodaySummary::default();
    });
}

/// Loads selected profiles after durable event logging exists; selection errors keep safe prior values and do not abort startup.
pub fn load_profiles(app: &AppHandle, event_logger: &crate::event_logger::EventLogger) {
    let app_data_dir = match app.path().app_data_dir() {
        Ok(dir) => dir,
        Err(error) => {
            record_profile_load_error(event_logger, &format!("Cannot get app_data_dir for profiles: {error}"));
            return;
        }
    };
    crate::profile_loader::ensure_profiles_dir(&app_data_dir);
    let Some(store) = app.try_state::<crate::app_store::AppStore<tauri::Wry>>() else {
        record_profile_load_error(event_logger, "Shared settings Store is unavailable while loading profiles");
        return;
    };
    let state: tauri::State<'_, AppState> = app.state();
    load_profiles_for_startup(
        &app_data_dir, store.inner(), &state.comm_policy, &state.session, event_logger,
    );
}

/// Applies both profiles or records a truthful durable error while preserving current policy/session values.
pub(crate) fn load_profiles_for_startup<R: tauri::Runtime>(
    app_data_dir: &Path,
    store: &tauri_plugin_store::Store<R>,
    policy: &Mutex<crate::communication_policy::CommunicationPolicy>,
    session: &Mutex<crate::session::SessionManager>,
    event_logger: &crate::event_logger::EventLogger,
) {
    match load_profiles_from_store(app_data_dir, store, policy, session) {
        Ok(()) => info!("Loaded selected communication + ergonomic profiles from {:?}", app_data_dir),
        Err(error) => record_profile_load_error(event_logger, &error),
    }
}

fn record_profile_load_error(event_logger: &crate::event_logger::EventLogger, error: &str) {
    warn!("Keeping prior/default communication and ergonomic profiles after startup load failure: {error}");
    event_logger.log(&format!("PROFILE_LOAD_FAILED message={error}"));
}

/// Testable startup seam over the actual shared Store used by app setup.
pub(crate) fn load_profiles_from_store<R: tauri::Runtime>(
    app_data_dir: &Path,
    store: &tauri_plugin_store::Store<R>,
    policy: &Mutex<crate::communication_policy::CommunicationPolicy>,
    session: &Mutex<crate::session::SessionManager>,
) -> Result<(), String> {
    fn selected_id(store: &tauri_plugin_store::Store<impl tauri::Runtime>, key: &str) -> Result<String, String> {
        match store.get(key) {
            None => Ok("default".to_string()),
            Some(value) => value.as_str().map(str::to_string)
                .ok_or_else(|| format!("Stored profile selection '{key}' must be a string")),
        }
    }

    let comm_id = selected_id(store, crate::commands_profiles::KEY_ACTIVE_COMM)?;
    let ergo_id = selected_id(store, crate::commands_profiles::KEY_ACTIVE_ERGO)?;
    let comm_path = crate::commands_profiles::profile_path(app_data_dir, "communication", &comm_id)?;
    let ergo_path = crate::commands_profiles::profile_path(app_data_dir, "ergonomic", &ergo_id)?;
    let comm = crate::profile_loader::load_communication_profile(&comm_path)
        .map_err(|error| format!("Cannot load selected communication profile '{comm_id}': {error}"))?;
    let ergo_text = std::fs::read_to_string(&ergo_path)
        .map_err(|error| format!("Cannot read selected ergonomic profile '{ergo_id}' at {:?}: {error}", ergo_path))?;
    let ergo = serde_json::from_str::<crate::ergonomic_profile::ErgonomicProfile>(&ergo_text)
        .map_err(|error| format!("Cannot parse selected ergonomic profile '{ergo_id}' at {:?}: {error}", ergo_path))?;

    // Keep the lock order used by profile switching: session, then policy.
    let mut session = session.lock().unwrap_or_else(|e| e.into_inner());
    let mut policy = policy.lock().unwrap_or_else(|e| e.into_inner());
    session.state.session_limit_secs = ergo.limits.sitting_secs as i64;
    session.state.stand_limit_secs = ergo.limits.standing_target_secs as i64;
    policy.set_comm_profile(comm);
    policy.set_ergo_profile(ergo);
    Ok(())
}


// ── Autostart helpers (release-only, placed after all pub fns) ───────────────

/// Reads the registered autostart exe path from the Windows Run registry key.
///
/// Returns `None` when the value is absent or on read error.
#[cfg(all(not(debug_assertions), target_os = "windows"))]
fn read_autostart_registry_path(app_name: &str) -> Option<String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run")
        .ok()?
        .get_value(app_name)
        .ok()
}

/// Non-Windows stub — always returns `None`.
#[cfg(all(not(debug_assertions), not(target_os = "windows")))]
fn read_autostart_registry_path(_app_name: &str) -> Option<String> {
    None
}

/// Extracts the executable path from a Windows `Run` command line.
///
/// The registry value carries launch arguments (`<exe> --minimized`), so the raw
/// value can never equal a bare executable path. Handles both the quoted form
/// (`"C:\dir with space\app.exe" --arg`) and the unquoted form
/// (`C:\dir\app.exe --arg`).
fn exe_from_command_line(value: &str) -> &str {
    let trimmed = value.trim();
    if let Some(rest) = trimmed.strip_prefix('"') {
        return match rest.find('"') {
            Some(end) => &rest[..end],
            None => rest,
        };
    }
    match trimmed.to_lowercase().find(".exe") {
        Some(idx) => trimmed[..idx + 4].trim(),
        None => trimmed,
    }
}

/// Compares two exe paths case-insensitively, normalising slashes and stripping quotes.
#[cfg(not(debug_assertions))]
fn paths_equal(a: &str, b: &str) -> bool {
    let normalize = |s: &str| s.trim_matches('"').replace('/', "\\").to_lowercase();
    normalize(a) == normalize(b)
}

#[cfg(test)]
mod tests {
    /// `paths_equal` is release-only; import it conditionally so tests still compile in debug.
    #[cfg(not(debug_assertions))]
    use super::paths_equal;

    /// Fallback for debug builds: define a local copy so tests can run.
    #[cfg(debug_assertions)]
    fn paths_equal(a: &str, b: &str) -> bool {
        let normalize = |s: &str| s.trim_matches('"').replace('/', "\\").to_lowercase();
        normalize(a) == normalize(b)
    }

    #[test]
    fn paths_equal_case_insensitive() {
        assert!(paths_equal(r"c:\Foo\bar.exe", r"C:\foo\bar.exe"));
    }

    #[test]
    fn paths_equal_trims_quotes() {
        assert!(paths_equal(r#""C:\bar.exe""#, r"C:\bar.exe"));
    }

    #[test]
    fn paths_equal_normalizes_slashes() {
        assert!(paths_equal(r"C:/foo/bar.exe", r"C:\foo\bar.exe"));
    }

    #[test]
    fn exe_from_command_line_strips_arguments() {
        assert_eq!(
            super::exe_from_command_line(r"C:\MoveUp\desk.exe --minimized"),
            r"C:\MoveUp\desk.exe"
        );
    }

    #[test]
    fn exe_from_command_line_handles_quoted_path_with_spaces() {
        assert_eq!(
            super::exe_from_command_line(r#""C:\Program Files\MoveUp\desk.exe" --minimized"#),
            r"C:\Program Files\MoveUp\desk.exe"
        );
    }

    #[test]
    fn exe_from_command_line_passes_through_bare_path() {
        assert_eq!(
            super::exe_from_command_line(r"C:\MoveUp\desk.exe"),
            r"C:\MoveUp\desk.exe"
        );
    }

    #[test]
    fn paths_equal_rejects_different_paths() {
        assert!(!paths_equal(r"C:\a.exe", r"C:\b.exe"));
    }

    #[test]
    fn is_minimized_arg_present() {
        // Can't mock std::env::args — test the detection logic inline
        let args: Vec<String> = vec!["exe".to_string(), "--minimized".to_string()];
        assert!(args.iter().any(|a| a == "--minimized"));
    }

    #[test]
    fn is_minimized_arg_absent() {
        let args: Vec<String> = vec!["exe".to_string()];
        assert!(!args.iter().any(|a| a == "--minimized"));
    }

    #[test]
    fn is_minimized_arg_in_middle() {
        let args: Vec<String> = vec![
            "exe".to_string(),
            "--foo".to_string(),
            "--minimized".to_string(),
            "--bar".to_string(),
        ];
        assert!(args.iter().any(|a| a == "--minimized"));
    }
}
