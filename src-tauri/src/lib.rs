//! lib.rs — zntl Desk Tauri backend entry point.
//!
//! Registers all plugins and commands, then starts the auto-connect scan
//! as soon as the app is set up.

mod activity;
mod commands;
mod config;
mod db;
mod overlay;
mod serial;
mod session;
mod tray;
mod tray_controller;
mod tray_icon;

use std::sync::{Arc, Mutex};

use commands::AppState;
use config::AppConfig;
use log::info;
use serial::ConnectionState;
use session::SessionManager;
use tauri::Manager;
use window_vibrancy::apply_acrylic;

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
        .manage(AppState {
            conn: Arc::new(ConnectionState::default()),
            session: Arc::new(Mutex::new(SessionManager::new())),
            db: Arc::new(Mutex::new(None)),
            config: Arc::new(Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_ports,
            commands::start_auto_connect,
            commands::stop_reading,
            commands::get_session_state,
            commands::set_session_limit,
            commands::set_stand_limit,
            commands::calibrate,
            commands::get_settings,
            commands::save_settings,
            commands::get_today_summary,
        ])
        .setup(|app| {
            // Load configuration from store (lazy-load since plugin may not be ready yet)
            let config = match app.try_state::<tauri_plugin_store::Store<tauri::Wry>>() {
                Some(store) => AppConfig::load(store.inner()),
                None => {
                    info!("Store not ready in setup, using defaults");
                    AppConfig::default()
                }
            };
            info!("Config: {:?}", config);

            // Initialize SQLite database
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let db_path = app_data_dir.join("desk.db");
            let db_conn = rusqlite::Connection::open(&db_path)?;
            db::init_schema(&db_conn)?;

            // Load today's totals from database
            let (sitting_secs, standing_secs) = match db::load_today_totals(&db_conn) {
                Ok((s, st)) => (s, st),
                Err(e) => {
                    log::error!("Failed to load today's totals: {}", e);
                    (0, 0)
                }
            };

            // Create session manager from config and seed with today's totals
            let mut session = SessionManager::new_from_config(&config);
            session.load_today_totals(sitting_secs, standing_secs);

            // Update AppState with db and config
            let app_state = app.state::<AppState>();
            *app_state.db.lock().unwrap() = Some(db_conn);
            *app_state.config.lock().unwrap() = Some(config);
            *app.state::<tauri::State<AppState>>()
                .session
                .lock()
                .unwrap() = session;

            // System tray icon and context menu.
            tray::setup_tray(app.handle())?;

            // Top-of-screen overlay progress bar window.
            overlay::setup_overlay(app.handle())?;

            // Wire state-changed events to tray + overlay updates.
            tray_controller::setup(app.handle());

            // Apply Acrylic blur on the main window (Windows 10/11).
            if let Some(window) = app.get_webview_window("main") {
                let _ = apply_acrylic(&window, Some((18, 18, 18, 200)));
            }

            // Kick off auto-detection immediately on startup.
            let state: tauri::State<'_, AppState> = app.state();
            serial::scan_and_connect(
                app.handle().clone(),
                state.conn.clone(),
                state.session.clone(),
                state.config.clone(),
            );
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running Desk");
}
