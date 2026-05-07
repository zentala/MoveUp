mod activity;
mod alert_popup;
mod alert_popup_window;
mod colors;
mod communication_policy;
mod communication_policy_helpers;
mod communication_profile;
mod communication_profile_defaults;
mod communication_profile_visuals;
mod communication_types;
mod ergonomic_profile;
mod profile_loader;
mod profile_reload;
mod commands;
mod commands_backup;
mod commands_config;
mod commands_profiles;
mod commands_share;
mod commands_welcome;
mod config;
mod db;
mod db_backup;
mod db_queries;
mod db_sessions;
pub mod event_logger;
mod height_stabilizer;
mod hourly_break_tracker;
mod metrics;
mod notification_service;
mod notify;
mod screen_break_nudge;
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
#[cfg(test)] mod communication_policy_tests;
#[cfg(test)] mod communication_policy_cooldown_tests;
#[cfg(test)] mod communication_policy_nudge_tests;
#[cfg(test)] mod commands_profiles_tests;
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
mod session_persistence;
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
#[cfg(test)] mod session_tests_day_break;
#[cfg(test)] mod session_tests_kpi;
#[cfg(test)] mod session_tests_away;
#[cfg(test)] mod session_tests_away_transitions;
#[cfg(test)] mod session_tests_scenarios;
#[cfg(test)] mod session_tests_scenarios_adv;
#[cfg(test)] mod session_tests_sleep;
#[cfg(test)] mod session_tests_flush;
#[cfg(test)] mod session_tests_persistence;
#[cfg(test)] mod session_tests_break_tracker;
mod tray;
mod tray_blink;
mod tray_icon;
mod tray_controller;
mod tray_helpers;
mod tray_signal_exec;
#[cfg(test)] mod tray_blink_tests;
#[cfg(test)] mod tray_controller_tests;
use std::sync::{Arc, Mutex};
use alert_popup::AlertPopup;
use communication_policy::CommunicationPolicy;
use communication_profile::CommunicationProfile;
use ergonomic_profile::ErgonomicProfile;
use commands::AppState;
use event_logger::EventLogger;
use overlay_renderer::OverlayRenderer;
use serial::ConnectionState;
use session::SessionManager;
use snapshot_logger::SnapshotLogger;
use tray_signal_exec::BlinkState;
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
            comm_policy: Arc::new(Mutex::new(CommunicationPolicy::new(
                CommunicationProfile::default(),
                ErgonomicProfile::default(),
            ))),
            alert_popup: Arc::new(Mutex::new(AlertPopup::new())),
            ws_tx: ws_broadcaster::create_channel(),
            today_cache: Arc::new(Mutex::new(crate::db::TodaySummary::default())),
        })
        .manage(BlinkState::new())
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
                    commands_profiles::list_communication_profiles,
                    commands_profiles::list_ergonomic_profiles,
                    commands_profiles::get_active_profiles,
                    commands_profiles::switch_communication_profile,
                    commands_profiles::switch_ergonomic_profile,
                    commands_profiles::open_profile_in_editor,
                    commands_profiles::duplicate_profile,
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
                    commands_profiles::list_communication_profiles,
                    commands_profiles::list_ergonomic_profiles,
                    commands_profiles::get_active_profiles,
                    commands_profiles::switch_communication_profile,
                    commands_profiles::switch_ergonomic_profile,
                    commands_profiles::open_profile_in_editor,
                    commands_profiles::duplicate_profile,
                ]
            }
        })
        .setup(|app| setup_helpers::perform_app_setup(app))
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
