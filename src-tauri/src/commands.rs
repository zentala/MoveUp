//! commands.rs — Tauri IPC commands: session, connection, and re-exports.

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::sync::broadcast;

use rusqlite::Connection;
use tauri::{Emitter, Manager, State};
use tauri_plugin_notification::NotificationExt;

use crate::{alert_manager::AlertManager, alert_popup::AlertPopup, config::AppConfig,
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
    /// Alert escalation state machine (Idle -> Stage1 -> Stage2).
    pub alert_manager: Arc<Mutex<AlertManager>>,
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

    if let Ok(totals) = crate::db::load_today_totals(&conn) {
        state.session.lock().unwrap().load_today_totals(&totals);
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
    session.set_limit_minutes(cfg.sit_limit_mins);
    session.set_stand_limit_minutes(cfg.stand_limit_mins);

    Ok(())
}

// ─── Commands ────────────────────────────────────────────────────────────────

/// Lists all available serial ports on the system.
#[tauri::command]
pub fn list_ports() -> Vec<PortInfo> {
    available_port_infos()
}

/// Triggers the auto-detection scan.
#[tauri::command]
pub fn start_auto_connect(app: tauri::AppHandle, state: State<'_, AppState>) {
    let loggers: tauri::State<'_, crate::Loggers> = app.state();
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
    let cfg = state.config.lock().unwrap();
    let cfg = cfg.as_ref().ok_or("Config not loaded")?;
    let metrics = MetricEngine::with_defaults().compute_all(&ss, cfg);
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
pub fn trigger_test_notification(app: tauri::AppHandle) -> Result<(), String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let last = LAST_TEST_NOTIFICATION.load(Ordering::Relaxed);
    if now - last < 60 {
        return Err(format!("Rate limited: wait {}s", 60 - (now - last)));
    }
    LAST_TEST_NOTIFICATION.store(now, Ordering::Relaxed);
    app.notification()
        .builder()
        .title("zntlDesk — Test")
        .body("Notifications are working!")
        .show()
        .map_err(|e| format!("Notification error: {}", e))
}

// DB backup commands moved to commands_backup.rs
// Welcome popup commands moved to commands_welcome.rs
