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
    pub session: Mutex<SessionManager>,
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
    scan_and_connect(app, state.conn.clone());
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

/// Returns aggregated totals and session rows for today.
///
/// The actual DB query must be issued from the frontend via the SQL plugin
/// (tauri-plugin-sql exposes its API to JS). This command returns a summary
/// computed from the in-memory session manager and a placeholder session list.
/// For a production implementation wire the DB read here via the plugin's
/// `Database::load` API once it exposes a Rust-side query interface.
#[tauri::command]
pub fn get_today_summary(state: State<'_, AppState>) -> TodaySummary {
    let session = state.session.lock().unwrap();
    let snap = session.snapshot();

    // In-memory totals only — persistent totals are accumulated in SQLite
    // and should be queried from the frontend via tauri-plugin-sql directly.
    TodaySummary {
        sitting_secs: snap.sitting_seconds,
        standing_secs: 0, // placeholder; real value comes from SQLite
        sessions: Vec::<SessionRow>::new(),
    }
}

/// Updates the sitting session limit.
///
/// `minutes` — new limit in minutes (e.g. 40).
#[tauri::command]
pub fn set_session_limit(minutes: u32, state: State<'_, AppState>) {
    state.session.lock().unwrap().set_limit_minutes(minutes);
}
