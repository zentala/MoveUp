//! setup_helpers.rs — App setup, window positioning, device notifications,
//! remote display wiring, and graceful shutdown persistence.

use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use log::{error, info, warn};
use tauri::{AppHandle, Listener, Manager};
use window_vibrancy::apply_acrylic;

#[cfg(not(debug_assertions))]
use tauri_plugin_autostart::ManagerExt;

use crate::commands::AppState;
use crate::ws_broadcaster::{self, DisplayEvent};

/// Main app setup — called from the Tauri `.setup()` closure in `lib.rs`.
///
/// Initialises logging, loads config, sets up tray, controllers, and serial scan.
pub fn perform_app_setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_data_dir)?;
    info!("App data dir: {:?}", app_data_dir);

    load_profiles(app.handle());
    crate::db_backup::backup_database(&app_data_dir);
    info!("DB path: {}", app_data_dir.join("desk.db").display());

    let logs_dir = app_data_dir.join("logs");
    let snapshot_logger = Arc::new(crate::snapshot_logger::SnapshotLogger::new(logs_dir.clone()));
    snapshot_logger.cleanup_old_logs(7);
    let event_logger = Arc::new(crate::event_logger::EventLogger::new(logs_dir));
    event_logger.log(&format!("START v{}", crate::APP_VERSION));
    app.manage(crate::Loggers {
        snapshot: snapshot_logger.clone(),
        event: event_logger.clone(),
    });

    ensure_autostart(app.handle(), &event_logger);
    crate::tray::setup_tray(app.handle())?;
    crate::tray_controller::setup(app.handle());
    crate::tray_signal_exec::start_blink_thread(app.handle().clone());
    position_main_window(app.handle());

    {
        let state: tauri::State<'_, AppState> = app.state();
        if let Some(store) = app.try_state::<tauri_plugin_store::Store<tauri::Wry>>() {
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

    setup_device_notifications(app.handle());
    Ok(())
}

/// Minimum interval between device-missing/lost notifications (5 minutes).
const DEVICE_NOTIFICATION_COOLDOWN_SECS: u64 = 300;

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
    let autostart = app.autolaunch();
    let current_exe = std::env::current_exe().unwrap_or_default();
    let current_path = current_exe.display().to_string();

    let is_enabled = autostart.is_enabled().unwrap_or(false);
    let registered_path = read_autostart_registry_path();

    // [B] Self-heal: if enabled but registered path differs, re-register
    let needs_reregister = is_enabled
        && registered_path
            .as_ref()
            .map_or(true, |p| !paths_equal(p, &current_path));

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

/// Throttled notifications for missing/lost sensor events.
pub fn setup_device_notifications(app: &AppHandle) {
    let last_notif = Arc::new(Mutex::new(
        Instant::now() - std::time::Duration::from_secs(DEVICE_NOTIFICATION_COOLDOWN_SECS),
    ));

    {
        let last = Arc::clone(&last_notif);
        app.listen("desk:device-missing", move |_| {
            let mut guard = last.lock().unwrap_or_else(|e| {
                log::warn!("Recovered from poisoned mutex");
                e.into_inner()
            });
            if guard.elapsed().as_secs() >= DEVICE_NOTIFICATION_COOLDOWN_SECS {
                crate::notify::show("Sensor not connected", "Plug in desk sensor.");
                *guard = Instant::now();
            }
        });
    }

    {
        let last = Arc::clone(&last_notif);
        app.listen("desk:device-lost", move |_| {
            let mut guard = last.lock().unwrap_or_else(|e| {
                log::warn!("Recovered from poisoned mutex");
                e.into_inner()
            });
            if guard.elapsed().as_secs() >= DEVICE_NOTIFICATION_COOLDOWN_SECS {
                crate::notify::show("Sensor disconnected", "Check USB cable.");
                *guard = Instant::now();
            }
        });
    }
}

/// Spawns the remote display HTTP+WS server and wires broadcast listeners.
pub fn setup_remote_display(app: &AppHandle) {
    let state: tauri::State<'_, AppState> = app.state();
    let remote_state = crate::remote_server::RemoteState {
        ws_tx: state.ws_tx.clone(),
        session: state.session.clone(),
        comm_policy: state.comm_policy.clone(),
        today_cache: state.today_cache.clone(),
        active_clients: Arc::new(AtomicUsize::new(0)),
    };
    let port = std::env::var("DESK_REMOTE_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(crate::remote_server::DEFAULT_PORT);
    tauri::async_runtime::spawn(async move {
        crate::remote_server::start(remote_state, port).await;
    });
    setup_broadcast_listeners(app);
}

/// Forwards device/daily-reset events to WebSocket for remote clients.
pub fn setup_broadcast_listeners(app: &AppHandle) {
    // desk:device-connected → broadcast to remote clients
    let handle1 = app.clone();
    app.listen("desk:device-connected", move |event| {
        let ws_tx = handle1.state::<AppState>().ws_tx.clone();
        let port = serde_json::from_str::<serde_json::Value>(event.payload())
            .ok()
            .and_then(|v| v.get("port").and_then(|p| p.as_str()).map(String::from))
            .unwrap_or_default();
        ws_broadcaster::broadcast_event(&ws_tx, &DisplayEvent::DeviceConnected { port });
    });

    // desk:device-lost → broadcast to remote clients
    let handle2 = app.clone();
    app.listen("desk:device-lost", move |_event| {
        let ws_tx = handle2.state::<AppState>().ws_tx.clone();
        ws_broadcaster::broadcast_event(&ws_tx, &DisplayEvent::DeviceLost);
    });

    // desk:daily-reset → broadcast + clear today_cache
    let handle3 = app.clone();
    app.listen("desk:daily-reset", move |_event| {
        let state = handle3.state::<AppState>();
        ws_broadcaster::broadcast_event(&state.ws_tx, &DisplayEvent::DailyReset);
        // Clear cached today summary
        let mut cache = state.today_cache.lock().unwrap_or_else(|e| e.into_inner());
        *cache = crate::db::TodaySummary::default();
    });
}

/// Loads communication + ergonomic profiles into CommunicationPolicy.
pub fn load_profiles(app: &AppHandle) {
    let app_data_dir = match app.path().app_data_dir() {
        Ok(d) => d,
        Err(e) => {
            warn!("Cannot get app_data_dir for profiles: {}", e);
            return;
        }
    };
    crate::profile_loader::ensure_profiles_dir(&app_data_dir);
    let comm_path = app_data_dir.join("profiles/communication/default.json");
    let ergo_path = app_data_dir.join("profiles/ergonomic/default.json");
    let comm = crate::profile_loader::load_profile::<crate::communication_profile::CommunicationProfile>(&comm_path);
    let ergo = crate::profile_loader::load_profile::<crate::ergonomic_profile::ErgonomicProfile>(&ergo_path);
    let state: tauri::State<'_, AppState> = app.state();
    let mut policy = state.comm_policy.lock().unwrap();
    policy.set_comm_profile(comm);
    policy.set_ergo_profile(ergo);
    info!("Loaded communication + ergonomic profiles from {:?}", app_data_dir);
}

/// Persists session flags + credit and flushes in-progress session to DB on exit.
pub fn flush_session_on_shutdown(app: &AppHandle) {
    let state: tauri::State<'_, AppState> = app.state();
    let (completed, state_label) = {
        let Ok(sess) = state.session.try_lock() else {
            warn!("Session lock held during shutdown — skipping flush");
            return;
        };
        crate::session_persistence::save_via_app(app, &sess);
        (sess.flush_current_session(), format!("{:?}", sess.current_state()))
    };
    let Some(ref completed) = completed else { return };
    let Ok(db_lock) = state.db.try_lock() else {
        warn!("DB lock held during shutdown — skipping flush");
        return;
    };
    let Some(ref conn) = *db_lock else {
        warn!("DB not initialized — cannot save session on shutdown");
        return;
    };
    match crate::db_sessions::insert_session(
        conn, &completed.started_at, &completed.ended_at,
        &state_label, completed.duration_secs,
    ) {
        Ok(()) => info!(
            "Graceful shutdown: saved {} session ({}s) to DB",
            state_label, completed.duration_secs
        ),
        Err(e) => error!("Graceful shutdown: failed to save session: {}", e),
    }
}

// ── Autostart helpers (release-only, placed after all pub fns) ───────────────

/// Reads the registered autostart exe path from the Windows Run registry key.
///
/// Returns `None` when the value is absent or on read error.
#[cfg(all(not(debug_assertions), target_os = "windows"))]
fn read_autostart_registry_path() -> Option<String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run")
        .ok()?
        .get_value("SmartDesk")
        .ok()
}

/// Non-Windows stub — always returns `None`.
#[cfg(all(not(debug_assertions), not(target_os = "windows")))]
fn read_autostart_registry_path() -> Option<String> {
    None
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
