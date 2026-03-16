//! commands.rs — Tauri IPC commands exposed to the frontend.

use std::sync::{Arc, Mutex};

use tauri::State;

use crate::{
    db::{SessionRow, TodaySummary},
    serial::{available_port_infos, scan_and_connect, ConnectionState, PortInfo},
    session::{SessionManager, SessionStateDto},
};

// ─── Shared state ─────────────────────────────────────────────────────────────

/// Application-level state managed by Tauri.
pub struct AppState {
    pub conn: Arc<ConnectionState>,
    pub session: Arc<Mutex<SessionManager>>,
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
    scan_and_connect(app, state.conn.clone(), state.session.clone());
}

/// Signals the background reader thread to stop.
#[tauri::command]
pub fn stop_reading(state: State<'_, AppState>) {
    crate::serial::stop_reading(&state.conn);
}

/// Returns a snapshot of the current session state.
#[tauri::command]
pub fn get_session_state(state: State<'_, AppState>) -> SessionStateDto {
    state.session.lock().unwrap().snapshot()
}

/// Updates the sitting session limit.
///
/// `minutes` — new limit in minutes (e.g. 40).
#[tauri::command]
pub fn set_session_limit(minutes: u32, state: State<'_, AppState>) {
    state.session.lock().unwrap().set_limit_minutes(minutes);
}

/// Returns the SQLite schema DDL for the frontend to execute via tauri-plugin-sql.
///
/// The frontend calls this on startup, splits by `;`, and executes each statement
/// so the schema is always up to date before any reads or writes.
#[tauri::command]
pub fn get_schema_sql() -> &'static str {
    crate::db::SCHEMA_SQL
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
    state: State<'_, AppState>,
) {
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
}
