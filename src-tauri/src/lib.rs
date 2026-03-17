//! lib.rs — zntl Desk Tauri backend entry point.
//!
//! Registers all plugins and commands, then starts the auto-connect scan
//! as soon as the app is set up.

mod activity;
mod colors;
mod commands;
mod config;
mod db;
mod overlay_renderer;
mod serial;
pub mod session;
mod tray;
mod tray_controller;
mod tray_icon;

use std::sync::{Arc, Mutex};

use commands::AppState;
use log::info;
use overlay_renderer::OverlayRenderer;
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
            overlay: Arc::new(OverlayRenderer::new()),
        })
        .invoke_handler({
            #[cfg(any(test, debug_assertions))]
            {
                tauri::generate_handler![
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
                    commands::inject_reading,
                ]
            }
            #[cfg(not(any(test, debug_assertions)))]
            {
                tauri::generate_handler![
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
                ]
            }
        })
        .setup(|app| {
            // Create app data directory
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            info!("App data dir: {:?}", app_data_dir);

            // System tray icon and context menu.
            tray::setup_tray(app.handle())?;

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
                state.db.clone(),
                state.config.clone(),
            );
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running Desk");
}
