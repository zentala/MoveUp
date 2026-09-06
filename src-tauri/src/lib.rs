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
#[cfg(test)] mod commands_tests;
mod commands_analyst;
#[cfg(test)] mod commands_analyst_tests;
mod commands_backup;
mod commands_catalog;
mod commands_catalog_sources;
#[cfg(test)] mod commands_catalog_tests;
mod commands_config;
mod commands_profiles;
mod commands_google_fit;
mod commands_share;
mod commands_welcome;
mod google_fit;
mod google_fit_client;
#[cfg(test)] mod google_fit_http_tests;
mod google_fit_models;
mod google_fit_service;
#[cfg(test)] mod google_fit_service_tests;
#[cfg(test)] mod google_fit_tests;
mod config;
mod db;
mod db_backup;
mod db_queries;
mod db_sessions;
mod today_totals;
mod desk_events;
pub mod event_logger;
mod height_stabilizer;
pub mod candidate_list;
#[cfg(test)] mod candidate_list_tests;
pub mod health_probe;
#[cfg(test)] mod health_probe_tests;
pub mod last_known_good;
#[cfg(test)] mod last_known_good_tests;
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
pub mod release_store;
#[cfg(test)] mod release_store_tests;
mod remote_display_state;
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
mod serial_periodic_reset;
#[cfg(test)] mod serial_periodic_tests;
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
#[cfg(test)] mod session_tests_clock;
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
#[cfg(test)] mod session_tests_e015_credited;
#[cfg(test)] mod session_tests_limits;
#[cfg(test)] mod session_tests_scenario_table;
mod tray;
#[cfg(test)] mod tray_tests;
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
use google_fit_service::{GoogleFitService, GoogleFitState};
use overlay_renderer::OverlayRenderer;
use serial::ConnectionState;
use session::SessionManager;
use snapshot_logger::SnapshotLogger;
use tray_signal_exec::BlinkState;
use tauri::Manager;
pub(crate) const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Commands registered only in test/debug builds.
///
/// `tauri::generate_handler!` is a proc macro over a fixed list of paths and
/// does not expand `#[cfg(...)]` on its entries, so the debug and release
/// handler lists below must stay hand-written duplicates. This constant is the
/// declared difference between them, and `e019_t04_handler_parity` reads the
/// two lists back out of this file and fails if they drift from it.
pub(crate) const DEBUG_ONLY_COMMANDS: &[&str] = &[
    "commands::inject_reading",
    "commands_config::get_overlay_state",
];
pub(crate) struct Loggers {
    pub snapshot: Arc<SnapshotLogger>,
    pub event: Arc<EventLogger>,
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Best-effort load of Google Fit credentials; absence treated as "not configured".
    let _ = dotenv::from_filename(".env.local");
    let _ = dotenv::dotenv();
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--minimized"])))
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
        .manage::<GoogleFitState>(std::sync::Arc::new(GoogleFitService::from_env()))
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
                    commands_analyst::get_snapshots_range,
                    commands_analyst::get_events_range,
                    commands_analyst::get_sessions_range,
                    commands_catalog::get_data_catalog,
                    commands_google_fit::get_steps_today,
                    commands_google_fit::refresh_steps_now,
                    tray::open_analyst_window,
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
                    commands_analyst::get_snapshots_range,
                    commands_analyst::get_events_range,
                    commands_analyst::get_sessions_range,
                    commands_catalog::get_data_catalog,
                    commands_google_fit::get_steps_today,
                    commands_google_fit::refresh_steps_now,
                    tray::open_analyst_window,
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

/// Guards the two hand-written `invoke_handler` command lists against drift.
///
/// The debug and release arms of `invoke_handler` are duplicates that differ
/// only by [`DEBUG_ONLY_COMMANDS`]. Nothing in the type system enforces that,
/// so this test reads both lists back out of this file's own source and
/// compares them. Adding a command to one arm and forgetting the other — the
/// actual failure mode, which produces a release build missing a command the
/// frontend calls — fails here.
#[cfg(test)]
mod e019_t04_handler_parity {
    use super::DEBUG_ONLY_COMMANDS;

    const SOURCE: &str = include_str!("lib.rs");

    /// Built by `concat!` so this literal never appears verbatim in the source
    /// being scanned — otherwise the test's own text would match.
    const MARKER: &str = concat!("generate_handler", "![");

    /// Returns the command paths of the `n`-th `generate_handler!` list in
    /// declaration order (0 = debug arm, 1 = release arm).
    fn handler_list(n: usize) -> Vec<String> {
        let start = SOURCE
            .match_indices(MARKER)
            .nth(n)
            .unwrap_or_else(|| panic!("no generate_handler list #{n} in lib.rs"))
            .0
            + MARKER.len();
        let len = SOURCE[start..]
            .find(']')
            .expect("unterminated generate_handler list");
        SOURCE[start..start + len]
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn debug_and_release_lists_differ_only_by_debug_only_commands() {
        let debug = handler_list(0);
        let release = handler_list(1);

        // An empty parse must not read as a clean pass.
        assert!(debug.len() > 1, "debug handler list parsed as {debug:?}");
        assert!(release.len() > 1, "release handler list parsed as {release:?}");

        let missing: Vec<&String> = release.iter().filter(|c| !debug.contains(c)).collect();
        assert!(
            missing.is_empty(),
            "commands in the release handler but not the debug handler: {missing:?}"
        );

        let mut extra: Vec<&str> = debug
            .iter()
            .filter(|c| !release.contains(c))
            .map(String::as_str)
            .collect();
        extra.sort_unstable();
        let mut expected: Vec<&str> = DEBUG_ONLY_COMMANDS.to_vec();
        expected.sort_unstable();
        assert_eq!(
            extra, expected,
            "debug-only commands drifted from DEBUG_ONLY_COMMANDS; add the command \
             to both handler lists, or declare it in DEBUG_ONLY_COMMANDS"
        );
    }
}

/// Exports the Rust→TypeScript bindings under `src/generated/` (ADR 017).
///
/// ts-rs normally emits one `export_bindings_*` test per `#[ts(export)]` type,
/// but those run under `Config::from_env()`, which renders `i64` as `bigint`.
/// The DTOs cross the wire as JSON, where `JSON.parse` yields `number`, so the
/// export runs from an explicit config instead and the attribute is left off.
#[cfg(test)]
mod ts_export {
    use ts_rs::{Config, TS};

    /// `i64` reaches TypeScript through `JSON.parse`, i.e. as `number`.
    fn config() -> Config {
        Config::new().with_large_int("number")
    }

    /// Writes every DTO TypeScript consumes, plus everything they reference.
    #[test]
    fn export_bindings() {
        let cfg = config();
        crate::metrics::DashboardState::export_all(&cfg).expect("DashboardState exports");
        crate::session_types::StateChangedPayload::export_all(&cfg)
            .expect("StateChangedPayload exports");
        crate::db::TodaySummary::export_all(&cfg).expect("TodaySummary exports");
        crate::config::AppConfig::export_all(&cfg).expect("AppConfig exports");
        crate::serial_parser::PortInfo::export_all(&cfg).expect("PortInfo exports");
    }

    /// The wire counters must not be `bigint` — arithmetic on the TS side
    /// would throw at runtime, and no test would catch it.
    #[test]
    fn export_bindings_render_i64_as_number() {
        let dto = crate::session_types::SessionStateDto::export_to_string(&config())
            .expect("SessionStateDto renders");
        assert!(dto.contains("limit_used_secs: number"), "got: {dto}");
        assert!(!dto.contains("bigint"), "got: {dto}");
    }
}
