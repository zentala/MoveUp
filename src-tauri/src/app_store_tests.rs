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

fn pending_history_snapshot(date_local: String) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "date_local": date_local,
        "daily_reset_after": null,
        "engine": {
            "sitting_seconds": 900,
            "sitting_seconds_total": 900,
            "standing_seconds": 600
        },
        "sitting_seconds": 900,
        "pending_history": [{
            "state": "Standing",
            "span": {
                "started_at": "2026-10-21T10:00:00+00:00",
                "ended_at": "2026-10-21T10:00:50+00:00",
                "duration_secs": 50
            },
            "credit": null
        }]
    })
}

#[test]
fn pending_history_survives_fresh_real_store_reload() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("settings.json");
    let first_owner = app();
    let first = crate::app_store::initialize_at(first_owner.handle(), &file).unwrap();
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    first.set("persisted_session_state", pending_history_snapshot(today));
    first.save().unwrap();

    let second_owner = app();
    let reopened = crate::app_store::initialize_at(second_owner.handle(), &file).unwrap();
    let restored = crate::session_persistence::PersistedEngineState::load(&reopened)
        .expect("same-day snapshot must load");
    let value = serde_json::to_value(restored).unwrap();
    let pending = value["pending_history"].as_array()
        .expect("unacknowledged history must survive a real Store reload");
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0]["state"], "Standing");
    assert_eq!(pending[0]["span"]["duration_secs"], 50);
}

#[test]
fn stale_snapshot_keeps_pending_history_without_overwriting_seeded_daily_totals() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("settings.json");
    let owner = app();
    let store = crate::app_store::initialize_at(owner.handle(), &file).unwrap();
    let yesterday = (chrono::Local::now().date_naive() - chrono::Duration::days(1))
        .format("%Y-%m-%d").to_string();
    store.set("persisted_session_state", pending_history_snapshot(yesterday));
    store.save().unwrap();

    let restored = crate::session_persistence::PersistedEngineState::load(&store)
        .expect("stale counters must not discard unacknowledged history");
    let value = serde_json::to_value(&restored).unwrap();
    assert_eq!(value["pending_history"].as_array().unwrap().len(), 1);

    let mut manager = crate::session::SessionManager::new();
    manager.state.sitting_seconds = 20;
    manager.state.sitting_seconds_total = 30;
    manager.state.standing_seconds = 40;
    manager.load_persisted_state(&restored);
    assert_eq!(manager.state.sitting_seconds, 20);
    assert_eq!(manager.state.sitting_seconds_total, 30);
    assert_eq!(manager.state.standing_seconds, 40);
}

#[test]
fn failed_cancellation_store_save_keeps_acknowledged_database_history_durable() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("settings.json");
    let owner = app();
    let store = crate::app_store::initialize_at(owner.handle(), &file).unwrap();
    let mut manager = crate::session::SessionManager::new();
    let end = chrono::DateTime::parse_from_rfc3339("2026-10-21T10:01:00Z")
        .unwrap().with_timezone(&chrono::Utc);
    let start = end - chrono::Duration::seconds(60);
    manager.state.state = crate::session_types::DeskState::Sitting;
    manager.state.sitting_started = Some(start);
    manager.history_span_started = Some(start);
    manager.history_last_reading = Some(end);
    manager.state.last_tick_ts = Some(end);
    let session = Mutex::new(manager);
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    crate::db::init_schema(&conn).unwrap();
    let db = Mutex::new(Some(conn));
    let mut progress = crate::app_lifecycle_runtime::FlushProgress::new();
    let save = |manager: &crate::session::SessionManager| {
        let snapshot = crate::session_persistence::PersistedEngineState::from_session(manager, None);
        if snapshot.save(&store).is_ok() { "saved" } else { "failed" }
    };
    assert_eq!(crate::app_lifecycle_runtime::flush_state_progress_at(
        &session, &db, &mut progress, end, save), "store=saved database=saved");
    let acknowledged_copy = temp.path().join("acknowledged.json");
    fs::copy(&file, &acknowledged_copy).unwrap();

    fs::remove_file(&file).unwrap();
    fs::create_dir(&file).unwrap();
    let failed = progress.cancel(&session, &db, |manager| {
        let snapshot = crate::session_persistence::PersistedEngineState::from_session(manager, None);
        if snapshot.save(&store).is_ok() { "saved" } else { "failed" }
    });
    assert!(failed.is_err(), "the synthetic Store save failure must be observed");
    assert!(session.lock().unwrap().pending_history.len() == 1,
        "the in-memory queue remains available for retry");
    let rows: i64 = db.lock().unwrap().as_ref().unwrap().query_row(
        "SELECT COUNT(*) FROM sessions", [], |row| row.get(0),
    ).unwrap();
    assert_eq!(rows, 1, "a failed Store save must not delete the durable acknowledged row");

    let reopened_owner = app();
    let reopened = crate::app_store::initialize_at(reopened_owner.handle(), &acknowledged_copy).unwrap();
    let restored = crate::session_persistence::PersistedEngineState::load(&reopened).unwrap();
    assert!(restored.pending_history.is_empty(),
        "the last durable Store snapshot is the acknowledged state; SQLite still owns the row");
}

#[test]
fn shutdown_snapshot_before_first_new_day_reading_keeps_pending_and_stale_counters_separate() {
    use chrono::TimeZone;

    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("settings.json");
    let owner = app();
    let store = crate::app_store::initialize_at(owner.handle(), &file).unwrap();
    let today = chrono::Local::now().date_naive();
    let previous_day = today.pred_opt().unwrap();
    let previous_start = chrono::Local.from_local_datetime(
        &previous_day.and_hms_opt(10, 0, 0).unwrap(),
    ).single().unwrap().with_timezone(&chrono::Utc);

    let mut manager = crate::session::SessionManager::new();
    for _ in 0..crate::session_types::DEBOUNCE_COUNT {
        manager.on_reading_at(800, true, previous_start);
    }
    for offset in 1..=crate::session_types::DEBOUNCE_COUNT {
        manager.on_reading_at(1200, true,
            previous_start + chrono::Duration::seconds(100 + offset as i64));
    }
    assert_eq!(manager.pending_history.len(), 1,
        "the test obtains pending history from a real confirmed posture transition");
    manager.last_reset_date = previous_day;
    manager.state.sitting_seconds = 900;
    manager.state.sitting_seconds_total = 900;
    manager.state.standing_seconds = 700;
    manager.state.daily_score = 17.0;
    manager.notify_inactivity_fired = true;
    let session = Mutex::new(manager);

    let conn = rusqlite::Connection::open_in_memory().unwrap();
    crate::db::init_schema(&conn).unwrap();
    let today_label = today.format("%Y-%m-%d").to_string();
    let today_start = chrono::Local.from_local_datetime(&today.and_hms_opt(10, 0, 0).unwrap())
        .single().unwrap().with_timezone(&chrono::Utc);
    crate::db_sessions::insert_session_with_date(
        &conn, &today_start.to_rfc3339(), &(today_start + chrono::Duration::seconds(300)).to_rfc3339(),
        "Sitting", 300, &today_label,
    ).unwrap();
    crate::db_sessions::insert_session_with_date(
        &conn, &(today_start + chrono::Duration::seconds(300)).to_rfc3339(),
        &(today_start + chrono::Duration::seconds(500)).to_rfc3339(), "Standing", 200, &today_label,
    ).unwrap();
    conn.execute_batch("CREATE TRIGGER reject_history BEFORE INSERT ON sessions
        BEGIN SELECT RAISE(FAIL, 'synthetic new-day history failure'); END;").unwrap();
    let db = Mutex::new(Some(conn));
    let now = chrono::Utc::now();
    let mut progress = crate::app_lifecycle_runtime::FlushProgress::new();
    let save = |engine: &crate::session::SessionManager| {
        let snapshot = crate::session_persistence::PersistedEngineState::from_session(engine, None);
        if snapshot.save(&store).is_ok() { "saved" } else { "failed" }
    };
    assert_eq!(crate::app_lifecycle_runtime::flush_state_progress_at(
        &session, &db, &mut progress, now, save,
    ), "store=saved database=failed");

    let fresh_owner = app();
    let fresh_store = crate::app_store::initialize_at(fresh_owner.handle(), &file).unwrap();
    let restored = crate::session_persistence::PersistedEngineState::load(&fresh_store)
        .expect("the pre-ack history snapshot remains readable after reopening the Store");
    assert_eq!(restored.date_local, previous_day.format("%Y-%m-%d").to_string(),
        "a shutdown before a new-day reading must not label old counters as today");
    assert_eq!(restored.pending_history.len(), 1,
        "the actual observed span remains pending when its database insert fails");

    let current_totals = {
        let guard = db.lock().unwrap();
        crate::db_sessions::load_today_totals(guard.as_ref().unwrap(), None).unwrap()
    };
    let mut recovered = crate::session::SessionManager::new();
    recovered.load_today_totals(&current_totals);
    recovered.load_persisted_state(&restored);
    assert_eq!(recovered.state.sitting_seconds, 300);
    assert_eq!(recovered.state.sitting_seconds_total, 300);
    assert_eq!(recovered.state.standing_seconds, 200);
    assert_eq!(recovered.state.daily_score, 0.0);
    assert!(!recovered.notify_inactivity_fired);
    assert_eq!(recovered.pending_history.len(), 1);
}
