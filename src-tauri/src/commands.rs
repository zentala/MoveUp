//! commands.rs — Tauri IPC commands exposed to the frontend.

use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tauri::{Emitter, Manager, State};

use crate::{
    config::AppConfig,
    db::TodaySummary,
    overlay_renderer::OverlayRenderer,
    serial::{available_port_infos, scan_and_connect, ConnectionState, PortInfo},
    session::{SessionManager, SessionStateDto},
};

// ─── Shared state ─────────────────────────────────────────────────────────────

/// Application-level state managed by Tauri.
pub struct AppState {
    pub conn: Arc<ConnectionState>,
    pub session: Arc<Mutex<SessionManager>>,
    pub db: Arc<Mutex<Option<Connection>>>,
    pub config: Arc<Mutex<Option<AppConfig>>>,
    pub overlay: Arc<OverlayRenderer>,
}

// ─── Initialization ───────────────────────────────────────────────────────────

/// Lazy-initializes AppState fields (db, config, session) on first command call.
/// Safe to call multiple times — subsequent calls are no-ops.
fn ensure_initialized(app: &tauri::AppHandle, state: &AppState) -> Result<(), String> {
    let mut db_guard = state.db.lock().unwrap();
    if db_guard.is_some() {
        return Ok(()); // Already initialized
    }

    // Initialize database
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let db_path = app_data_dir.join("desk.db");

    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    crate::db::init_schema(&conn)
        .map_err(|e| format!("Failed to initialize database schema: {}", e))?;

    // Load today's totals from database into session manager
    if let Ok((sitting, standing)) = crate::db::load_today_totals(&conn) {
        let mut session = state.session.lock().unwrap();
        session.load_today_totals(sitting, standing);
    }

    *db_guard = Some(conn);

    // Initialize config from store (if available)
    if let Some(store) = app.try_state::<tauri_plugin_store::Store<tauri::Wry>>() {
        let loaded_config = AppConfig::load(store.inner());
        *state.config.lock().unwrap() = Some(loaded_config.clone());

        // Update session manager calibration from config
        let mut session = state.session.lock().unwrap();
        session.sitting_height_cm = loaded_config.sitting_mm as f32 / 10.0;
        session.standing_height_cm = loaded_config.standing_mm as f32 / 10.0;
        session.desk_thickness_cm = loaded_config.desk_thickness_mm as f32 / 10.0;
        session.set_limit_minutes(loaded_config.sit_limit_mins);
        session.set_stand_limit_minutes(loaded_config.stand_limit_mins);
    } else {
        // No store available — use AppConfig defaults
        let default_config = AppConfig::default();
        *state.config.lock().unwrap() = Some(default_config.clone());

        let mut session = state.session.lock().unwrap();
        session.sitting_height_cm = default_config.sitting_mm as f32 / 10.0;
        session.standing_height_cm = default_config.standing_mm as f32 / 10.0;
        session.desk_thickness_cm = default_config.desk_thickness_mm as f32 / 10.0;
        session.set_limit_minutes(default_config.sit_limit_mins);
        session.set_stand_limit_minutes(default_config.stand_limit_mins);
    }

    Ok(())
}

// ─── Commands ────────────────────────────────────────────────────────────────

/// Lists all available serial ports on the system.
#[tauri::command]
pub fn list_ports() -> Vec<PortInfo> {
    available_port_infos()
}

/// Triggers the auto-detection scan; starts a background reader if a desk
/// sensor is found. Safe to call multiple times — a running scan is a no-op.
#[tauri::command]
pub fn start_auto_connect(app: tauri::AppHandle, state: State<'_, AppState>) {
    scan_and_connect(app, state.conn.clone(), state.session.clone(), state.db.clone(), state.config.clone());
}

/// Signals the background reader thread to stop.
#[tauri::command]
pub fn stop_reading(state: State<'_, AppState>) {
    crate::serial::stop_reading(&state.conn);
}

/// Returns a snapshot of the current session state.
#[tauri::command]
pub fn get_session_state(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<SessionStateDto, String> {
    ensure_initialized(&app, &state)?;
    Ok(state.session.lock().unwrap().snapshot())
}

/// Updates the sitting session limit.
///
/// `minutes` — new limit in minutes (e.g. 40).
#[tauri::command]
pub fn set_session_limit(minutes: u32, app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    ensure_initialized(&app, &state)?;
    state.session.lock().unwrap().set_limit_minutes(minutes);
    Ok(())
}

/// Updates the standing session limit.
///
/// `minutes` — new limit in minutes (e.g. 15). Set to 0 to disable.
#[tauri::command]
pub fn set_stand_limit(minutes: u32, app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    ensure_initialized(&app, &state)?;
    state.session.lock().unwrap().set_stand_limit_minutes(minutes);
    Ok(())
}

/// Updates height calibration values used by the session state machine.
///
/// All parameters are optional; only provided values are updated.
///
/// - `sitting_mm`       — desk height when sitting (default 750 mm = 75 cm)
/// - `standing_mm`      — desk height when standing (default 1150 mm = 115 cm)
/// - `desk_thickness_mm`— desk surface thickness to subtract from sensor reading (default 30 mm = 3 cm)
#[tauri::command]
pub fn calibrate(
    sitting_mm: Option<i32>,
    standing_mm: Option<i32>,
    desk_thickness_mm: Option<i32>,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    ensure_initialized(&app, &state)?;
    let mut session = state.session.lock().unwrap();
    if let Some(mm) = sitting_mm {
        session.sitting_height_cm = mm as f32 / 10.0;
    }
    if let Some(mm) = standing_mm {
        session.standing_height_cm = mm as f32 / 10.0;
    }
    if let Some(mm) = desk_thickness_mm {
        session.desk_thickness_cm = mm as f32 / 10.0;
    }
    Ok(())
}

/// Returns the current application configuration.
#[tauri::command]
pub fn get_settings(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<AppConfig, String> {
    ensure_initialized(&app, &state)?;
    Ok(state.config.lock().unwrap().clone().unwrap_or_default())
}

/// Saves updated application configuration.
/// Validates via `config.clamped()`, writes to store, and updates SessionManager.
#[tauri::command]
pub fn save_settings(
    config: AppConfig,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    use tauri::Manager;

    ensure_initialized(&app, &state)?;

    let clamped = config.clamped();

    // Save to store
    let store_state = app
        .try_state::<tauri_plugin_store::Store<tauri::Wry>>()
        .ok_or_else(|| "Failed to access store".to_string())?;

    clamped.save(store_state.inner())?;

    // Update in-memory config
    *state.config.lock().unwrap() = Some(clamped.clone());

    // Update session manager with new calibration
    let mut session = state.session.lock().unwrap();
    session.sitting_height_cm = clamped.sitting_mm as f32 / 10.0;
    session.standing_height_cm = clamped.standing_mm as f32 / 10.0;
    session.desk_thickness_cm = clamped.desk_thickness_mm as f32 / 10.0;
    session.set_limit_minutes(clamped.sit_limit_mins);
    session.set_stand_limit_minutes(clamped.stand_limit_mins);

    Ok(())
}

/// Returns today's summary of sitting and standing time.
#[tauri::command]
pub fn get_today_summary(app: tauri::AppHandle, state: State<'_, AppState>) -> Result<TodaySummary, String> {
    ensure_initialized(&app, &state)?;

    let db_lock = state.db.lock().unwrap();
    let conn = db_lock
        .as_ref()
        .ok_or_else(|| "Database not initialized".to_string())?;

    let mut summary = crate::db::get_today_summary(conn)?;

    // Add position_changes from in-memory SessionManager
    let session = state.session.lock().unwrap();
    let dto = session.snapshot();
    summary.position_changes = dto.position_changes;

    Ok(summary)
}

/// Test/debug command: inject a sensor reading directly into the session manager.
/// Only available in debug builds or when cfg(test) is enabled.
///
/// `mm` — raw sensor reading in millimeters
/// `active` — whether user has been active recently (keyboard/mouse)
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

    // Emit state change event if there was a transition
    if let Some(payload) = result.state_change {
        let _ = app.emit("desk:state-changed", &payload);
    }

    Ok(())
}
