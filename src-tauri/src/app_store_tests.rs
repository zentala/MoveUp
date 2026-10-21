use std::{fs, sync::{Arc, Mutex}};
use tauri::{Manager, test::{mock_builder, mock_context, noop_assets, MockRuntime}};
use tauri_plugin_store::StoreExt;

#[cfg(all(target_os = "windows", target_env = "msvc"))]
#[link(name = "resource", kind = "static", modifiers = "-bundle")]
unsafe extern "C" {}

fn app() -> tauri::App<MockRuntime> {
    mock_builder().plugin(tauri_plugin_store::Builder::default().build())
        .build(mock_context(noop_assets())).unwrap()
}

#[test]
fn shared_real_store_load_preserves_bytes_and_fresh_app_reloads_config_and_engine() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("settings.json");
    let bytes = b"{ \"unknown_key\" : { \"keep\" : 7 } }\n";
    fs::write(&file, bytes).unwrap();
    let app1 = app();
    let shared = crate::app_store::initialize_at(app1.handle(), &file).unwrap();
    let again = crate::app_store::initialize_at(app1.handle(), &file).unwrap();
    assert!(Arc::ptr_eq(&shared, &again));
    assert!(Arc::ptr_eq(&shared, &app1.state::<crate::app_store::AppStore<MockRuntime>>()));
    assert!(Arc::ptr_eq(&shared, &app1.get_store(&file).unwrap()));
    assert_eq!(fs::read(&file).unwrap(), bytes);

    let config = crate::config::AppConfig {
        sitting_mm: 765,
        standing_mm: 1120,
        show_welcome_on_startup: false,
        telemetry_device_id: "synthetic-device".into(),
        ..Default::default()
    };
    config.save(&shared).unwrap();
    let mut manager = crate::session::SessionManager::new();
    manager.state.sitting_seconds = 123;
    manager.notify_inactivity_fired = true;
    let snapshot = crate::session_persistence::PersistedEngineState::from_session(&manager, Some("reset-marker".into()));
    snapshot.save(&shared).unwrap();

    let app2 = app();
    let reopened = crate::app_store::initialize_at(app2.handle(), &file).unwrap();
    assert!(!Arc::ptr_eq(&shared, &reopened));
    assert_eq!(reopened.get("unknown_key").unwrap()["keep"], 7);
    let loaded_config = crate::config::AppConfig::load(&reopened);
    assert_eq!(serde_json::to_value(loaded_config).unwrap(), serde_json::to_value(config).unwrap());
    let restored = crate::session_persistence::PersistedEngineState::load(&reopened).unwrap();
    assert_eq!(restored.sitting_seconds, 123);
    assert!(restored.notify_inactivity_fired);
    assert_eq!(restored.daily_reset_after.as_deref(), Some("reset-marker"));
}

#[test]
fn missing_file_initializes_shared_store_without_writing_defaults() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("settings.json");
    let owner = app();
    let shared = crate::app_store::initialize_at(owner.handle(), &file).unwrap();
    assert!(shared.is_empty());
    assert!(!file.exists());
    assert!(owner.try_state::<crate::app_store::AppStore<MockRuntime>>().is_some());
}

#[test]
fn malformed_load_preserves_file_and_never_exposes_managed_store() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("settings.json");
    for bytes in ["{", "[]"] {
        fs::write(&file, bytes).unwrap();
        let owner = app();
        assert!(crate::app_store::initialize_at(owner.handle(), &file).is_err());
        assert!(owner.try_state::<crate::app_store::AppStore<MockRuntime>>().is_none());
        assert!(owner.get_store(&file).is_none());
        assert_eq!(fs::read_to_string(&file).unwrap(), bytes);
    }
}

#[test]
fn initialization_io_error_leaves_no_managed_store_and_can_retry() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("settings.json");
    fs::create_dir(&file).unwrap();
    let owner = app();
    let error = crate::app_store::initialize_at(owner.handle(), &file).err().expect("initialization must fail");
    assert!(!error.is_empty());
    assert!(owner.try_state::<crate::app_store::AppStore<MockRuntime>>().is_none());
    assert!(owner.get_store(&file).is_none());
    assert!(file.is_dir());
    fs::remove_dir(&file).unwrap();
    crate::app_store::initialize_at(owner.handle(), &file).unwrap();
    assert!(owner.try_state::<crate::app_store::AppStore<MockRuntime>>().is_some());
    assert!(!file.exists());
}

#[test]
fn real_flush_save_failure_and_retry_persist_current_state() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("settings.json");
    let owner = app();
    let shared = crate::app_store::initialize_at(owner.handle(), &file).unwrap();
    fs::create_dir(&file).unwrap(); // A directory at the selected file makes real save fail.
    let session = Mutex::new(crate::session::SessionManager::new());
    let db = Mutex::new(None);
    let mut progress = crate::app_lifecycle_runtime::FlushProgress::new();
    let save = |manager: &crate::session::SessionManager| {
        let snapshot = crate::session_persistence::PersistedEngineState::from_session(manager, None);
        if snapshot.save(&shared).is_ok() { "saved" } else { "failed" }
    };
    assert_eq!(crate::app_lifecycle_runtime::flush_state_progress(
        &session, &db, &mut progress, save), "store=failed database=not_attempted");
    session.lock().unwrap().notify_inactivity_fired = true;
    fs::remove_dir(&file).unwrap();
    assert_eq!(crate::app_lifecycle_runtime::flush_state_progress(
        &session, &db, &mut progress, save), "store=saved database=not_needed");
    let reopened_owner = app();
    let reopened = crate::app_store::initialize_at(reopened_owner.handle(), &file).unwrap();
    let restored = crate::session_persistence::PersistedEngineState::load(&reopened).unwrap();
    assert!(restored.notify_inactivity_fired);
}
