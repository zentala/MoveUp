//! lib.rs — zntl Desk Tauri backend entry point.
//!
//! Registers all plugins and commands, then starts the auto-connect scan
//! as soon as the app is set up.

mod activity;
mod commands;
mod db;
mod serial;
mod session;

use std::sync::{Arc, Mutex};

use commands::AppState;
use serial::ConnectionState;
use session::SessionManager;
use tauri::Manager;

/// Application entry point called from main.rs.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(tauri_plugin_sql::Builder::default().build())
        .manage(AppState {
            conn: Arc::new(ConnectionState::default()),
            session: Mutex::new(SessionManager::new()),
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_ports,
            commands::start_auto_connect,
            commands::stop_reading,
            commands::get_session_state,
            commands::get_today_summary,
            commands::set_session_limit,
        ])
        .setup(|app| {
            // Kick off auto-detection immediately on startup.
            let state: tauri::State<'_, AppState> = app.state();
            serial::scan_and_connect(app.handle().clone(), state.conn.clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running Desk");
}
