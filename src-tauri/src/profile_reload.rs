//! profile_reload.rs — Hot-reload profiles from disk when files change.
//!
//! Reads the currently active profile IDs from the store each invocation so
//! that switching profiles is reflected immediately — the watcher follows the
//! active file instead of always watching `default.json`.

use std::cell::RefCell;
use std::path::PathBuf;

use log::info;
use tauri::{AppHandle, Manager};

use crate::commands::AppState;
use crate::communication_profile::CommunicationProfile;
use crate::ergonomic_profile::ErgonomicProfile;
use crate::profile_loader::{load_profile, ProfileWatcher};

/// Holds watchers together with the paths they are currently tracking.
struct Watchers {
    comm: ProfileWatcher,
    comm_path: PathBuf,
    ergo: ProfileWatcher,
    ergo_path: PathBuf,
}

thread_local! {
    static WATCHERS: RefCell<Option<Watchers>> = const { RefCell::new(None) };
}

const DEFAULT_ID: &str = "default";
const KEY_COMM: &str = "active_communication_profile";
const KEY_ERGO: &str = "active_ergonomic_profile";

/// Reloads communication and ergonomic profiles from disk if they changed.
/// Called every ~60s from `serial_periodic::check_periodic`.
pub fn reload_profiles_if_changed(app: &AppHandle) {
    let app_data_dir = match app.path().app_data_dir() {
        Ok(d) => d,
        Err(_) => return,
    };

    let (active_comm_id, active_ergo_id) =
        if let Some(store) = app.try_state::<tauri_plugin_store::Store<tauri::Wry>>() {
            let comm = store
                .get(KEY_COMM)
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_else(|| DEFAULT_ID.to_string());
            let ergo = store
                .get(KEY_ERGO)
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_else(|| DEFAULT_ID.to_string());
            (comm, ergo)
        } else {
            (DEFAULT_ID.to_string(), DEFAULT_ID.to_string())
        };

    let comm_path = app_data_dir
        .join("profiles/communication")
        .join(format!("{}.json", active_comm_id));
    let ergo_path = app_data_dir
        .join("profiles/ergonomic")
        .join(format!("{}.json", active_ergo_id));

    WATCHERS.with(|w| {
        let mut w = w.borrow_mut();

        if w.is_none() {
            *w = Some(Watchers {
                comm: ProfileWatcher::new(comm_path.clone()),
                comm_path: comm_path.clone(),
                ergo: ProfileWatcher::new(ergo_path.clone()),
                ergo_path: ergo_path.clone(),
            });
            return;
        }

        let watchers = w.as_mut().expect("watchers initialized");
        let state: tauri::State<'_, AppState> = app.state();

        if watchers.comm_path != comm_path {
            watchers.comm = ProfileWatcher::new(comm_path.clone());
            watchers.comm_path = comm_path.clone();
        }
        if watchers.ergo_path != ergo_path {
            watchers.ergo = ProfileWatcher::new(ergo_path.clone());
            watchers.ergo_path = ergo_path.clone();
        }

        if watchers.comm.has_changed() {
            let profile = load_profile::<CommunicationProfile>(&comm_path);
            info!("Hot-reloaded communication profile from {:?}", comm_path);
            state
                .comm_policy
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .set_comm_profile(profile);
        }
        if watchers.ergo.has_changed() {
            let profile = load_profile::<ErgonomicProfile>(&ergo_path);
            info!("Hot-reloaded ergonomic profile from {:?}", ergo_path);
            // The session engine holds its own copy of the limits; without this
            // it would keep the values captured at construction and the reload
            // would silently change nothing for break credit, PostureBalance or
            // the computer-time reset (E020-T02).
            state
                .session
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .set_ergo_profile(&profile);
            state
                .comm_policy
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .set_ergo_profile(profile);
        }
    });
}
