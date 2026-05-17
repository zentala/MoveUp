//! commands.rs — Tauri IPC commands: session, connection, and re-exports.

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::broadcast;

use rusqlite::Connection;
use tauri::{Emitter, Manager, State};
use crate::{alert_popup::AlertPopup, communication_policy::CommunicationPolicy,
    config::AppConfig,
    db::TodaySummary, metrics::{DashboardState, MetricEngine},
    overlay_renderer::OverlayRenderer,
    serial::{available_port_infos, scan_and_connect, ConnectionState, PortInfo},
    session::{SessionManager, SessionStateDto}};

/// Rate limiter for test notification (epoch seconds of last send).
static LAST_TEST_NOTIFICATION: AtomicI64 = AtomicI64::new(0);

// ─── Shared state ────────────────────────────────────────────────────────────

/// Application-level state managed by Tauri.
pub struct AppState {
    pub conn: Arc<ConnectionState>,
    pub session: Arc<Mutex<SessionManager>>,
    pub db: Arc<Mutex<Option<Connection>>>,
    pub config: Arc<Mutex<Option<AppConfig>>>,
    pub overlay: Arc<OverlayRenderer>,
    /// Central communication policy engine (replaces AlertManager).
    pub comm_policy: Arc<Mutex<CommunicationPolicy>>,
    /// WinAPI popup window shown at Stage2.
    pub alert_popup: Arc<Mutex<AlertPopup>>,
    /// Broadcast sender for remote display WebSocket clients.
    pub ws_tx: broadcast::Sender<String>,
    /// Cached today summary — refreshed on state transitions, not per-tick.
    pub today_cache: Arc<Mutex<TodaySummary>>,
}

// ─── Initialization ──────────────────────────────────────────────────────────

/// Lazy-initializes AppState fields (db, config, session) on first call.
pub fn ensure_initialized(app: &tauri::AppHandle, state: &AppState) -> Result<(), String> {
    let mut db_guard = state.db.lock().unwrap();
    if db_guard.is_some() {
        return Ok(());
    }

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let db_path = app_data_dir.join("desk.db");

    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    crate::db::init_schema(&conn)
        .map_err(|e| format!("Failed to initialize database schema: {}", e))?;

    // Load reset_after timestamp to filter out pre-daily-reset sessions.
    // On first run (no reset_after in store), all today's sessions are loaded.
    // After the first daily reset with this code, the filter activates.
    let reset_after = app
        .try_state::<tauri_plugin_store::Store<tauri::Wry>>()
        .and_then(|store| crate::session_persistence::load_reset_after(store.inner()));

    if let Some(ref ts) = reset_after {
        log::info!("Seeding with daily_reset_after filter: {}", ts);
    } else {
        log::info!("Seeding without daily_reset_after filter (first run or no reset yet)");
    }
    if let Ok(totals) = crate::db::load_today_totals(&conn, reset_after.as_deref()) {
        state.session.lock().unwrap().load_today_totals(&totals);
    }

    // Restore persisted notification flags and credit-reduced sitting_seconds.
    if let Some(store) = app.try_state::<tauri_plugin_store::Store<tauri::Wry>>() {
        if let Some(persisted) = crate::session_persistence::PersistedSessionState::load(store.inner()) {
            state.session.lock().unwrap().load_persisted_state(&persisted);
        }
    }

    // Populate today_cache from DB on first init
    if let Ok(summary) = crate::db::get_today_summary(&conn) {
        *state.today_cache.lock().unwrap() = summary;
    }

    *db_guard = Some(conn);

    let cfg = app
        .try_state::<tauri_plugin_store::Store<tauri::Wry>>()
        .map(|store| AppConfig::load(store.inner()))
        .unwrap_or_default();
    *state.config.lock().unwrap() = Some(cfg.clone());

    let mut session = state.session.lock().unwrap();
    session.sitting_height_cm = cfg.sitting_mm as f32 / 10.0;
    session.standing_height_cm = cfg.standing_mm as f32 / 10.0;
    session.desk_thickness_cm = cfg.desk_thickness_mm as f32 / 10.0;
    // Limits come from the ergonomic profile (already in CommunicationPolicy)
    let ergo = state.comm_policy.lock().unwrap().ergo_profile().clone();
    session.state.session_limit_secs = ergo.limits.sitting_secs as i64;
    session.state.stand_limit_secs = ergo.limits.standing_target_secs as i64;

    Ok(())
}

// ─── Commands ────────────────────────────────────────────────────────────────

/// Lists all available serial ports on the system.
#[tauri::command]
pub fn list_ports() -> Vec<PortInfo> {
    available_port_infos()
}

/// Triggers the auto-detection scan.
///
/// Loggers may not yet be managed if the frontend races setup completion;
/// in that case we no-op and the user can retry once setup finishes (or
/// setup_helpers's own `scan_and_connect` call will have already started a
/// scan thread with the loggers it had in scope).
#[tauri::command]
pub fn start_auto_connect(app: tauri::AppHandle, state: State<'_, AppState>) {
    let Some(loggers) = app.try_state::<crate::Loggers>() else {
        log::warn!("start_auto_connect: Loggers state not yet managed; scan skipped");
        return;
    };
    let sl = loggers.snapshot.clone();
    let el = loggers.event.clone();
    scan_and_connect(
        app,
        state.conn.clone(),
        state.session.clone(),
        state.db.clone(),
        state.config.clone(),
        sl,
        el,
    );
}

/// Signals the background reader thread to stop.
#[tauri::command]
pub fn stop_reading(state: State<'_, AppState>) {
    crate::serial::stop_reading(&state.conn);
}

/// Returns a snapshot of the current session state.
#[tauri::command]
pub fn get_session_state(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<SessionStateDto, String> {
    ensure_initialized(&app, &state)?;
    Ok(state.session.lock().unwrap().snapshot())
}

/// Returns session snapshot + all KPI metrics in a single IPC call.
#[tauri::command]
pub fn get_dashboard_state(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<DashboardState, String> {
    ensure_initialized(&app, &state)?;
    let (snapshot, ss) = {
        let s = state.session.lock().unwrap();
        let now = chrono::Utc::now();
        let mut raw = s.state.clone();
        // Inject live values so metrics see current totals (not stale committed values).
        raw.sitting_seconds_total = s.get_live_sitting_seconds_total(now);
        raw.standing_seconds = s.get_live_standing_seconds(now);
        (s.snapshot(), raw)
    };
    let ergo = state.comm_policy.lock().unwrap().ergo_profile().clone();
    let metrics = MetricEngine::with_defaults().compute_all(&ss, &ergo);
    Ok(DashboardState { session: snapshot, metrics })
}

/// Returns the currently connected serial port name, or null.
#[tauri::command]
pub fn get_connected_port(state: State<'_, AppState>) -> Option<String> {
    state.conn.connected_port.lock().unwrap().clone()
}

/// Returns today's summary of sitting and standing time.
#[tauri::command]
pub fn get_today_summary(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<TodaySummary, String> {
    ensure_initialized(&app, &state)?;

    let db_lock = state.db.lock().unwrap();
    let conn = db_lock
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    let mut summary = crate::db::get_today_summary(conn)?;
    let session = state.session.lock().unwrap();
    summary.position_changes = session.snapshot().position_changes;

    Ok(summary)
}

/// Test/debug command: inject a sensor reading directly.
#[tauri::command]
#[cfg(any(test, debug_assertions))]
pub fn inject_reading(
    mm: i32,
    active: bool,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    ensure_initialized(&app, &state)?;

    let mut session = state.session.lock().unwrap();
    let result = session.on_reading(mm, active);

    if let Some(payload) = result.state_change {
        let _ = app.emit("desk:state-changed", &payload);
    }

    Ok(())
}

/// Sends a test notification to verify notifications are working.
/// Rate-limited: max once per 60 seconds to prevent spam.
/// Available in all builds (not debug-only) so users can test from settings.
#[tauri::command]
pub fn trigger_test_notification(_app: tauri::AppHandle) -> Result<(), String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let last = LAST_TEST_NOTIFICATION.load(Ordering::Relaxed);
    if now - last < 60 {
        return Err(format!("Rate limited: wait {}s", 60 - (now - last)));
    }
    LAST_TEST_NOTIFICATION.store(now, Ordering::Relaxed);
    crate::notify::show("Test notification", "Notifications are working!");
    Ok(())
}

// DB backup commands moved to commands_backup.rs
// Welcome popup commands moved to commands_welcome.rs
