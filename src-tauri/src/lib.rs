mod activity;
mod alert_actions;
mod alert_config;
mod alert_manager;
mod alert_popup;
mod alert_popup_window;
mod colors;
mod commands;
mod commands_backup;
mod commands_config;
mod commands_share;
mod commands_welcome;
mod config;
mod db;
mod db_backup;
mod db_queries;
mod db_sessions;
mod event_logger;
mod height_stabilizer;
mod hourly_break_tracker;
mod metrics;
mod notification_service;
#[cfg(test)] mod notification_service_tests;
#[cfg(test)] mod notification_service_tests_edge;
mod overlay_layered;
mod overlay_layered_wndproc;
mod overlay_opaque;
mod overlay_renderer;
mod overlay_standing;
#[cfg(test)] mod overlay_standing_tests;
mod overlay_variants;
mod remote_server;
#[cfg(test)] mod remote_server_tests;
mod ws_broadcaster;
#[cfg(test)] mod overlay_tests;
#[cfg(test)] mod overlay_variant_tests;
#[cfg(test)] mod alert_manager_tests;
#[cfg(test)] mod alert_snooze_tests;
#[cfg(test)] mod config_tests;
#[cfg(test)] mod db_tests;
#[cfg(test)] mod db_tests_roundtrip;
mod serial;
mod serial_parser;
mod serial_periodic;
mod setup_helpers;
mod snapshot_logger;
mod telemetry;
pub mod session;
pub mod session_manager;
mod session_breaks;
mod session_live;
mod session_daily;
mod session_reading;
pub mod session_types;
#[cfg(test)] mod session_tests;
#[cfg(test)] mod session_tests_alerts;
#[cfg(test)] mod session_tests_floating;
#[cfg(test)] mod session_tests_daily;
#[cfg(test)] mod session_tests_props;
#[cfg(test)] mod session_tests_score;
#[cfg(test)] mod session_tests_timers;
#[cfg(test)] mod session_tests_timers_live;
#[cfg(test)] mod session_tests_serde;
#[cfg(test)] mod session_tests_break_credit;
#[cfg(test)] mod session_tests_kpi;
#[cfg(test)] mod session_tests_away;
#[cfg(test)] mod session_tests_away_transitions;
#[cfg(test)] mod session_tests_scenarios;
#[cfg(test)] mod session_tests_scenarios_adv;
#[cfg(test)] mod session_tests_sleep;
#[cfg(test)] mod session_tests_flush;
mod tray;
mod tray_controller;
mod tray_helpers;
#[cfg(test)] mod tray_controller_tests;
mod tray_icon;
use std::sync::{Arc, Mutex};
use alert_manager::{AlertConfig, AlertManager};
use alert_popup::AlertPopup;
use commands::AppState;
use event_logger::EventLogger;
use log::info;
use overlay_renderer::OverlayRenderer;
use serial::ConnectionState;
use session::SessionManager;
use snapshot_logger::SnapshotLogger;
use tauri::Manager;
/// App version constant, used by loggers.
pub(crate) const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Holds file-based loggers, managed as separate Tauri state.
pub(crate) struct Loggers {
    pub snapshot: Arc<SnapshotLogger>,
    pub event: Arc<EventLogger>,
}

/// Application entry point called from main.rs.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec![])))
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") { let _ = w.show(); let _ = w.set_focus(); }
        }))
        .manage(AppState {
            conn: Arc::new(ConnectionState::default()),
            session: Arc::new(Mutex::new(SessionManager::new())),
            db: Arc::new(Mutex::new(None)),
            config: Arc::new(Mutex::new(None)),
            overlay: Arc::new(OverlayRenderer::new()),
            alert_manager: Arc::new(Mutex::new(AlertManager::new(AlertConfig::default()))),
            alert_popup: Arc::new(Mutex::new(AlertPopup::new())),
            ws_tx: ws_broadcaster::create_channel(),
            today_cache: Arc::new(Mutex::new(crate::db::TodaySummary::default())),
        })
        .invoke_handler({
            #[cfg(any(test, debug_assertions))]
            {
                tauri::generate_handler![
                    commands::list_ports,
                    commands::start_auto_connect,
                    commands::stop_reading,
                    commands::get_session_state,
                    commands::get_dashboard_state,
                    commands::get_connected_port,
                    commands::get_today_summary,
                    commands::inject_reading,
                    commands::trigger_test_notification,
                    commands_welcome::dismiss_welcome,
                    commands_welcome::show_welcome,
                    commands_config::set_session_limit,
                    commands_config::set_stand_limit,
                    commands_config::calibrate,
                    commands_config::get_settings,
                    commands_config::save_settings,
                    commands_config::get_overlay_state,
                    commands_share::get_share_text,
                    commands_backup::list_db_backups,
                    commands_backup::restore_db_backup,
                ]
            }
            #[cfg(not(any(test, debug_assertions)))]
            {
                tauri::generate_handler![
                    commands::list_ports,
                    commands::start_auto_connect,
                    commands::stop_reading,
                    commands::get_session_state,
                    commands::get_dashboard_state,
                    commands::get_connected_port,
                    commands::get_today_summary,
                    commands::trigger_test_notification,
                    commands_welcome::dismiss_welcome,
                    commands_welcome::show_welcome,
                    commands_config::set_session_limit,
                    commands_config::set_stand_limit,
                    commands_config::calibrate,
                    commands_config::get_settings,
                    commands_config::save_settings,
                    commands_share::get_share_text,
                    commands_backup::list_db_backups,
                    commands_backup::restore_db_backup,
                ]
            }
        })
        .setup(|app| {
            // Create app data directory
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            info!("App data dir: {:?}", app_data_dir);

            // Auto-backup DB before anything else (eager, not lazy)
            let db_path = app_data_dir.join("desk.db");
            info!("DB path: {}", db_path.display());
            db_backup::backup_database(&app_data_dir);

            // Initialize loggers now that app_data_dir is known.
            let logs_dir = app_data_dir.join("logs");
            let snapshot_logger = Arc::new(SnapshotLogger::new(logs_dir.clone()));
            snapshot_logger.cleanup_old_logs(7);
            let event_logger = Arc::new(EventLogger::new(logs_dir));
            event_logger.log(&format!("START v{}", APP_VERSION));
            app.manage(Loggers {
                snapshot: snapshot_logger.clone(),
                event: event_logger.clone(),
            });

            tray::setup_tray(app.handle())?;
            tray_controller::setup(app.handle());
            setup_helpers::position_main_window(app.handle());

            // Load config from store and apply to SessionManager before sensor scan starts.
            // This ensures calibration values are correct from the first sensor reading.
            {
                let state: tauri::State<'_, AppState> = app.state();
                if let Some(store) = app.try_state::<tauri_plugin_store::Store<tauri::Wry>>() {
                    let config = crate::config::AppConfig::load(store.inner());
                    let mut session = state.session.lock().unwrap_or_else(|e| e.into_inner());
                    *session = crate::session::SessionManager::new_from_config(&config);
                    *state.config.lock().unwrap_or_else(|e| e.into_inner()) = Some(config.clone());
                    info!("startup: config loaded from store into SessionManager");

                    // Show welcome popup on first launch (or if user hasn't dismissed it).
                    if config.show_welcome_on_startup {
                        if let Err(e) = commands_welcome::show_welcome_window(app.handle()) {
                            log::warn!("Failed to show welcome popup: {}", e);
                        }
                    }
                }
            }

            // Spawn remote display server + wire broadcast listeners.
            setup_helpers::setup_remote_display(app.handle());

            // Kick off auto-detection immediately on startup.
            let state: tauri::State<'_, AppState> = app.state();
            serial::scan_and_connect(
                app.handle().clone(),
                state.conn.clone(),
                state.session.clone(),
                state.db.clone(),
                state.config.clone(),
                snapshot_logger,
                event_logger,
            );

            // Throttled notifications for missing/lost sensor.
            setup_helpers::setup_device_notifications(app.handle());

            Ok(())
        })
        .on_window_event(|window, event| {
            // T038: Hide the popup window when it loses focus (click outside).
            if window.label() == "main" {
                if let tauri::WindowEvent::Focused(false) = event {
                    let _ = window.hide();
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("error building Desk")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                setup_helpers::flush_session_on_shutdown(app);
            }
        });
}
