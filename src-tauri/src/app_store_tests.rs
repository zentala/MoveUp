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
fn shared_store_closes_the_plugins_exit_writer_but_keeps_rust_readers() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("settings.json");
    let owner = app();
    let store = crate::app_store::initialize_at(owner.handle(), &file).unwrap();
    store.set("preserved_unknown_key", serde_json::json!({"keep": true}));
    store.save().unwrap();

    assert_eq!(store.get("preserved_unknown_key").unwrap()["keep"], true);
    let plugin_reader = owner.store_builder(&file).build().unwrap();
    assert!(
        !std::ptr::eq(plugin_reader.as_ref(), &*store),
        "a fresh plugin Store proves close_resource removed the shared Store from the plugin's exit-save registry"
    );
    assert_eq!(plugin_reader.get("preserved_unknown_key").unwrap()["keep"], true);
    plugin_reader.close_resource();
}

#[test]
fn checkpoint_recovers_known_sitting_prefix_without_crediting_the_restart_gap() {
    use chrono::{Duration as ChronoDuration, Utc};

    let temp = tempfile::tempdir().unwrap();
    let settings_file = temp.path().join("settings.json");
    let db_file = temp.path().join("desk.db");
    let owner = app();
    let store = crate::app_store::initialize_at(owner.handle(), &settings_file).unwrap();
    let mut live = crate::session::SessionManager::new();
    let t0 = Utc::now() - ChronoDuration::seconds(600);

    // Establish a real Sitting span and keep every sensor observation within
    // the existing five-minute coverage rule through t0 + 600 seconds.
    // Five elapsed seconds of debounce establish Sitting at t0.
    for offset in (-5..=0).chain((60..=600).step_by(60)) {
        live.on_reading_at(720, true, t0 + ChronoDuration::seconds(offset));
    }
    assert_eq!(live.state.state, crate::session_types::DeskState::Sitting);
    assert!(live.state.sitting_seconds == 0 && live.state.sitting_seconds_total == 0,
        "the producer leaves the current interval uncommitted until a real boundary");
    assert!(live.pending_history.is_empty());

    let policy = crate::communication_policy::CommunicationPolicy::new(
        crate::communication_profile::CommunicationProfile::default(),
        crate::ergonomic_profile::ErgonomicProfile::default(),
    );
    let checkpoint = crate::session_persistence::PersistedEngineState::from_session_with_policy(
        &live,
        None,
        &policy,
        t0 + ChronoDuration::seconds(600),
        std::time::Instant::now(),
    );
    checkpoint.save(&store).unwrap();

    assert!(live.pending_history.is_empty(), "checkpointing must not close or mutate the live interval");
    assert_eq!(live.state.sitting_seconds, 0);
    assert_eq!(live.state.sitting_seconds_total, 0);
    let checkpoint_owner = app();
    let checkpoint_store = crate::app_store::initialize_at(checkpoint_owner.handle(), &settings_file).unwrap();
    let saved = crate::session_persistence::PersistedEngineState::load(&checkpoint_store).unwrap();
    let row = saved.pending_history.as_slice();
    assert_eq!(row.len(), 1, "the checkpoint carries one recoverable known prefix");
    assert_eq!(row[0].state, crate::session_types::DeskState::Sitting);
    assert_eq!(row[0].span.duration_secs, 600);
    let saved_engine = saved.engine.as_ref().unwrap();
    assert_eq!(saved_engine.sitting_seconds, 600);
    assert_eq!(saved_engine.sitting_seconds_total, 600);

    let db = rusqlite::Connection::open(&db_file).unwrap();
    crate::db::init_schema(&db).unwrap();
    let empty_totals = crate::db_sessions::load_today_totals(&db, None).unwrap();
    let mut recovered = crate::session::SessionManager::new();
    recovered.load_today_totals(&empty_totals);
    recovered.load_persisted_state(&saved);
    assert_eq!(recovered.pending_history.len(), 1);
    assert_eq!(recovered.pending_history[0].span.duration_secs, 600);
    assert!(recovered.state.last_tick_ts.is_none(), "restart must clear the live observation anchor");

    // The stopped interval is unknown. A new coherent sample run starts a new
    // observed prefix instead of extending Sitting through the outage.
    for offset in 1200..=1205 {
        recovered.on_reading_at(720, true, t0 + ChronoDuration::seconds(offset));
    }
    recovered.on_reading_at(720, true, t0 + ChronoDuration::seconds(1265));
    assert_eq!(recovered.state.state, crate::session_types::DeskState::Sitting);
    assert_eq!(recovered.history_span_started, Some(t0 + ChronoDuration::seconds(1205)));
    assert_eq!(recovered.get_live_sitting_seconds(t0 + ChronoDuration::seconds(1265)), 660,
        "the six hundred seconds of stopped time are not credited");
    let after_resume = crate::session_persistence::PersistedEngineState::from_session(&recovered, None);
    assert_eq!(after_resume.sitting_seconds, 660);
    assert_eq!(after_resume.pending_history.len(), 2);
    assert_eq!(after_resume.pending_history.iter().filter(|pending|
        pending.span.duration_secs == 600).count(), 1,
        "repeated checkpoints retain the original history row exactly once");

    let mut inserted = Vec::new();
    crate::serial_periodic_reset::persist_pending_history(&mut recovered, &db, |pending, id| {
        inserted.push((pending.span.duration_secs, id));
    }).unwrap();
    assert_eq!(inserted.len(), 1, "the live manager retries only its restored pending row");
    assert_eq!(inserted[0].0, 600);
    let rows: i64 = db.query_row("SELECT COUNT(*) FROM sessions", [], |record| record.get(0)).unwrap();
    assert_eq!(rows, 1, "the known prefix is stored once; the resumed live span stays open");

    // A crash after the INSERT but before the next settings save replays the
    // same durable envelope. The adapter recognizes the existing row.
    let replay_owner = app();
    let replay_store = crate::app_store::initialize_at(replay_owner.handle(), &settings_file).unwrap();
    let replay_snapshot = crate::session_persistence::PersistedEngineState::load(&replay_store).unwrap();
    let replay_totals = crate::db_sessions::load_today_totals(&db, None).unwrap();
    let mut replayed = crate::session::SessionManager::new();
    replayed.load_today_totals(&replay_totals);
    replayed.load_persisted_state(&replay_snapshot);
    assert_eq!(replayed.pending_history.len(), 1);
    let mut replay_insertions = Vec::new();
    crate::serial_periodic_reset::persist_pending_history(&mut replayed, &db, |pending, id| {
        replay_insertions.push((pending.span.duration_secs, id));
    }).unwrap();
    assert!(replay_insertions.is_empty(), "replay acknowledges the already inserted observation without a duplicate");
    let rows_after_replay: i64 = db.query_row("SELECT COUNT(*) FROM sessions", [], |record| record.get(0)).unwrap();
    assert_eq!(rows_after_replay, 1, "first persistence and crash replay leave exactly one DB row");
}

#[test]
fn checkpointed_standing_and_away_break_keeps_known_credit_across_restart() {
    use chrono::{Duration as ChronoDuration, Utc};

    let temp = tempfile::tempdir().unwrap();
    let settings_file = temp.path().join("settings.json");
    let db_file = temp.path().join("desk.db");
    let owner = app();
    let store = crate::app_store::initialize_at(owner.handle(), &settings_file).unwrap();
    let mut live = crate::session::SessionManager::new();
    let t0 = Utc::now() - ChronoDuration::seconds(1812);

    for offset in -5..=0 {
        live.on_reading_at(720, true, t0 + ChronoDuration::seconds(offset));
    }
    for offset in (60..=600).step_by(60) {
        live.on_reading_at(720, true, t0 + ChronoDuration::seconds(offset));
    }
    for offset in 601..=606 {
        live.on_reading_at(1200, true, t0 + ChronoDuration::seconds(offset));
    }
    assert_eq!(live.state.state, crate::session_types::DeskState::Standing);
    for offset in (666..=1206).step_by(60) {
        live.on_reading_at(1200, true, t0 + ChronoDuration::seconds(offset));
    }
    for offset in 1207..=1212 {
        live.on_reading_at(720, false, t0 + ChronoDuration::seconds(offset));
    }
    assert_eq!(live.state.state, crate::session_types::DeskState::Away);
    for offset in (1272..=1812).step_by(60) {
        live.on_reading_at(720, false, t0 + ChronoDuration::seconds(offset));
    }

    let policy = crate::communication_policy::CommunicationPolicy::new(
        crate::communication_profile::CommunicationProfile::default(),
        crate::ergonomic_profile::ErgonomicProfile::default(),
    );
    let checkpoint = crate::session_persistence::PersistedEngineState::from_session_with_policy(
        &live,
        None,
        &policy,
        t0 + ChronoDuration::seconds(1812),
        std::time::Instant::now(),
    );
    let expected_break = checkpoint.engine.as_ref().unwrap().break_seconds;
    assert!(expected_break >= 1200, "fixture has an observed standing-plus-away break prefix");
    assert_eq!(checkpoint.engine.as_ref().unwrap().sitting_seconds_total, 606);
    assert_eq!(checkpoint.engine.as_ref().unwrap().standing_seconds, 606);
    checkpoint.save(&store).unwrap();

    let reopened_owner = app();
    let reopened_store = crate::app_store::initialize_at(reopened_owner.handle(), &settings_file).unwrap();
    let saved = crate::session_persistence::PersistedEngineState::load(&reopened_store).unwrap();
    let db = rusqlite::Connection::open(&db_file).unwrap();
    crate::db::init_schema(&db).unwrap();
    let mut restored = crate::session::SessionManager::new();
    restored.load_today_totals(&crate::db_sessions::load_today_totals(&db, None).unwrap());
    restored.load_persisted_state(&saved);
    assert_eq!(restored.state.sitting_seconds_total, 606, "raw Sitting prefix survives");
    assert_eq!(restored.state.standing_seconds, 606, "raw Standing prefix survives");
    assert_eq!(restored.state.break_seconds, expected_break);

    // The outage is unknown. Only new readings establish a return to Sitting;
    // the pre-crash observed break earns credit at that real transition.
    for offset in 2400..=2405 {
        restored.on_reading_at(720, true, t0 + ChronoDuration::seconds(offset));
    }
    assert_eq!(restored.state.state, crate::session_types::DeskState::Sitting);
    assert_eq!(restored.state.last_break_secs, expected_break);
    assert_eq!(restored.state.last_break_credit, crate::session_types::BreakCredit::Full);
    assert_eq!(restored.state.sitting_seconds, 0);
    assert_eq!(restored.state.sitting_seconds_total, 606, "break credit never changes raw history");
    restored.on_reading_at(720, true, t0 + ChronoDuration::seconds(2465));
    assert_eq!(restored.state.last_break_secs, expected_break, "a later reading does not apply break credit twice");

    let pending = restored.pending_history.len();
    assert_eq!(pending, 3, "the Sitting, Standing, and Away history prefixes are each retained once");
    let mut inserted = Vec::new();
    crate::serial_periodic_reset::persist_pending_history(&mut restored, &db, |row, id| {
        inserted.push((row.state.clone(), row.span.duration_secs, id));
    }).unwrap();
    assert_eq!(inserted.len(), 3);
    let rows: i64 = db.query_row("SELECT COUNT(*) FROM sessions", [], |record| record.get(0)).unwrap();
    assert_eq!(rows, 3, "all known individual intervals persist once across the outage");
}

#[test]
fn concurrent_settings_updates_are_saved_as_one_serialized_complete_store() {
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("settings.json");
    let owner = app();
    let store = Arc::new(crate::app_store::initialize_at(owner.handle(), &file).unwrap());
    let (first_entered_tx, first_entered_rx) = mpsc::channel();
    let (release_first_tx, release_first_rx) = mpsc::channel();
    let (second_done_tx, second_done_rx) = mpsc::channel();

    let first_store = store.clone();
    let first = thread::spawn(move || {
        let writer = first_store.write();
        writer.set("first_writer", serde_json::json!(true));
        first_entered_tx.send(()).unwrap();
        release_first_rx.recv().unwrap();
        writer.save().unwrap();
    });
    first_entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();

    let second_store = store.clone();
    let second = thread::spawn(move || {
        let writer = second_store.write();
        writer.set("second_writer", serde_json::json!(true));
        writer.save().unwrap();
        second_done_tx.send(()).unwrap();
    });
    assert!(second_done_rx.recv_timeout(Duration::from_millis(30)).is_err(),
        "the second complete update waits for the first writer to release the shared Store lock");
    release_first_tx.send(()).unwrap();
    first.join().unwrap();
    second.join().unwrap();

    let reader = app();
    let reloaded = crate::app_store::initialize_at(reader.handle(), &file).unwrap();
    assert_eq!(reloaded.get("first_writer"), Some(serde_json::json!(true)));
    assert_eq!(reloaded.get("second_writer"), Some(serde_json::json!(true)));
}

#[test]
fn startup_restores_both_selected_profiles_from_fresh_shared_store() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("settings.json");
    let profiles_dir = temp.path().join("app-data");
    crate::profile_loader::ensure_profiles_dir(&profiles_dir);
    let event_logger = crate::event_logger::EventLogger::new(temp.path().join("startup-logs"));

    let first_owner = app();
    let first = crate::app_store::initialize_at(first_owner.handle(), &file).unwrap();
    first.set("active_communication_profile", "silent");
    first.set("active_ergonomic_profile", "strict");
    first.set("unrelated_startup_key", serde_json::json!({"keep": true}));
    first.save().unwrap();

    let second_owner = app();
    let reopened = crate::app_store::initialize_at(second_owner.handle(), &file).unwrap();
    let policy = Mutex::new(crate::communication_policy::CommunicationPolicy::new(
        crate::communication_profile::CommunicationProfile::default(),
        crate::ergonomic_profile::ErgonomicProfile::default(),
    ));
    let session = Mutex::new(crate::session::SessionManager::new());

    crate::setup_helpers::load_profiles_for_startup(
        &profiles_dir, &reopened, &policy, &session, &event_logger,
    );

    {
        let policy = policy.lock().unwrap();
        assert_eq!(policy.comm_profile().id, "silent");
        assert!(!policy.comm_profile().notifications_enabled);
        assert_eq!(policy.ergo_profile().id, "strict");
    }
    {
        let policy = policy.lock().unwrap();
        let config = crate::config::AppConfig::default();
        let initialized_later = crate::session::SessionManager::new_from_config(
            &config, policy.ergo_profile(),
        );
        assert_eq!(initialized_later.state.session_limit_secs, 1500);
        assert_eq!(initialized_later.state.stand_limit_secs, 600);
    }
    assert_eq!(session.lock().unwrap().state.session_limit_secs, 1500);
    assert_eq!(session.lock().unwrap().state.stand_limit_secs, 600);
    assert_eq!(reopened.get("active_communication_profile").unwrap(), "silent");
    assert_eq!(reopened.get("active_ergonomic_profile").unwrap(), "strict");
    assert_eq!(reopened.get("unrelated_startup_key").unwrap()["keep"], true);

    let third_owner = app();
    let disk_reload = crate::app_store::initialize_at(third_owner.handle(), &file).unwrap();
    assert_eq!(disk_reload.get("active_communication_profile").unwrap(), "silent");
    assert_eq!(disk_reload.get("active_ergonomic_profile").unwrap(), "strict");
}

#[test]
fn startup_defaults_only_when_selection_is_absent_and_reports_invalid_selection() {
    let temp = tempfile::tempdir().unwrap();
    let app_data = temp.path().join("app-data");
    crate::profile_loader::ensure_profiles_dir(&app_data);
    let event_logger = crate::event_logger::EventLogger::new(temp.path().join("startup-logs"));

    let absent_file = temp.path().join("absent-settings.json");
    let absent_owner = app();
    let absent_store = crate::app_store::initialize_at(absent_owner.handle(), &absent_file).unwrap();
    let absent_policy = Mutex::new(crate::communication_policy::CommunicationPolicy::new(
        crate::communication_profile::CommunicationProfile::default(),
        crate::ergonomic_profile::ErgonomicProfile::default(),
    ));
    let absent_session = Mutex::new(crate::session::SessionManager::new());
    crate::setup_helpers::load_profiles_for_startup(
        &app_data, &absent_store, &absent_policy, &absent_session, &event_logger,
    );
    assert_eq!(absent_policy.lock().unwrap().comm_profile().id, "default");
    assert_eq!(absent_policy.lock().unwrap().ergo_profile().id, "default");
    assert_eq!(absent_store.get("active_communication_profile"), None,
        "default startup must not materialize an absent selection key");

    let invalid_file = temp.path().join("invalid-settings.json");
    let invalid_owner = app();
    let invalid_store = crate::app_store::initialize_at(invalid_owner.handle(), &invalid_file).unwrap();
    invalid_store.set("active_communication_profile", "missing-profile");
    invalid_store.save().unwrap();
    let invalid_policy = Mutex::new(crate::communication_policy::CommunicationPolicy::new(
        crate::communication_profile::CommunicationProfile::default(),
        crate::ergonomic_profile::ErgonomicProfile::default(),
    ));
    let invalid_session = Mutex::new(crate::session::SessionManager::new());
    crate::setup_helpers::load_profiles_for_startup(
        &app_data, &invalid_store, &invalid_policy, &invalid_session, &event_logger,
    );
    assert_eq!(invalid_policy.lock().unwrap().comm_profile().id, "default");
    assert_eq!(invalid_store.get("active_communication_profile").unwrap(), "missing-profile");
    let invalid_reload_owner = app();
    let invalid_disk = crate::app_store::initialize_at(invalid_reload_owner.handle(), &invalid_file).unwrap();
    assert_eq!(invalid_disk.get("active_communication_profile").unwrap(), "missing-profile");

    let malformed_path = app_data.join("profiles/ergonomic/malformed.json");
    fs::write(&malformed_path, "{ malformed json").unwrap();
    let malformed_file = temp.path().join("malformed-settings.json");
    let malformed_owner = app();
    let malformed_store = crate::app_store::initialize_at(malformed_owner.handle(), &malformed_file).unwrap();
    malformed_store.set("active_ergonomic_profile", "malformed");
    malformed_store.save().unwrap();
    let malformed_policy = Mutex::new(crate::communication_policy::CommunicationPolicy::new(
        crate::communication_profile::CommunicationProfile::default(),
        crate::ergonomic_profile::ErgonomicProfile::default(),
    ));
    let malformed_session = Mutex::new(crate::session::SessionManager::new());
    crate::setup_helpers::load_profiles_for_startup(
        &app_data, &malformed_store, &malformed_policy, &malformed_session, &event_logger,
    );
    assert_eq!(malformed_policy.lock().unwrap().ergo_profile().id, "default");
    assert_eq!(malformed_store.get("active_ergonomic_profile").unwrap(), "malformed");
    let malformed_disk = app();
    let malformed_reload = crate::app_store::initialize_at(malformed_disk.handle(), &malformed_file).unwrap();
    assert_eq!(malformed_reload.get("active_ergonomic_profile").unwrap(), "malformed");

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let events = fs::read_to_string(temp.path().join("startup-logs").join(today).join("events.log")).unwrap();
    assert!(events.contains("PROFILE_LOAD_FAILED"));
    assert!(events.contains("missing-profile"));
    assert!(events.contains("Cannot parse selected ergonomic profile 'malformed'"));
}

#[cfg(windows)]
#[test]
fn failed_persistent_profile_switches_restore_runtime_memory_and_disk_then_retry() {
    fn make_read_only(path: &std::path::Path, read_only: bool) {
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_readonly(read_only);
        fs::set_permissions(path, permissions).unwrap();
    }

    let temp = tempfile::tempdir().unwrap();
    let app_data = temp.path().join("app-data");
    crate::profile_loader::ensure_profiles_dir(&app_data);

    // Communication: absence of a saved key remains absent after a failed switch.
    let comm_file = temp.path().join("communication-settings.json");
    let comm_owner = app();
    let comm_store = crate::app_store::initialize_at(comm_owner.handle(), &comm_file).unwrap();
    comm_store.set("unrelated", serde_json::json!({"keep": 11}));
    comm_store.save().unwrap();
    let comm_before = fs::read(&comm_file).unwrap();
    let comm_policy = Mutex::new(crate::communication_policy::CommunicationPolicy::new(
        crate::communication_profile::CommunicationProfile::default(),
        crate::ergonomic_profile::ErgonomicProfile::default(),
    ));
    make_read_only(&comm_file, true);
    let comm_error = crate::commands_profiles::switch_communication_profile_persisted(
        &app_data, &comm_store, &comm_policy, "silent",
    ).expect_err("read-only Store save must fail");
    assert_eq!(comm_policy.lock().unwrap().comm_profile().id, "default");
    assert_eq!(comm_store.get("active_communication_profile"), None,
        "failed switch must restore the originally absent in-memory key");
    assert!(comm_error.contains("Failed to save store"));
    assert!(comm_error.contains("rollback save also failed"));
    assert!(comm_error.contains("disk selection is uncertain"));
    assert_eq!(fs::read(&comm_file).unwrap(), comm_before,
        "this injected failure occurs before bytes change");
    let comm_reload_owner = app();
    let comm_disk = crate::app_store::initialize_at(comm_reload_owner.handle(), &comm_file).unwrap();
    assert_eq!(comm_disk.get("active_communication_profile"), None);
    assert_eq!(comm_disk.get("unrelated").unwrap()["keep"], 11);

    make_read_only(&comm_file, false);
    crate::commands_profiles::switch_communication_profile_persisted(
        &app_data, &comm_store, &comm_policy, "silent",
    ).expect("retry after clearing the injected Store fault must persist");
    assert_eq!(comm_policy.lock().unwrap().comm_profile().id, "silent");
    assert_eq!(comm_store.get("active_communication_profile").unwrap(), "silent");
    let comm_retry_owner = app();
    let comm_retry_disk = crate::app_store::initialize_at(comm_retry_owner.handle(), &comm_file).unwrap();
    assert_eq!(comm_retry_disk.get("active_communication_profile").unwrap(), "silent");
    assert_eq!(comm_retry_disk.get("unrelated").unwrap()["keep"], 11);

    // Ergonomics: an explicit prior key and the live session limits are preserved.
    let ergo_file = temp.path().join("ergonomic-settings.json");
    let ergo_owner = app();
    let ergo_store = crate::app_store::initialize_at(ergo_owner.handle(), &ergo_file).unwrap();
    ergo_store.set("active_ergonomic_profile", "default");
    ergo_store.set("unrelated", serde_json::json!({"keep": 22}));
    ergo_store.save().unwrap();
    let ergo_before = fs::read(&ergo_file).unwrap();
    let ergo_policy = Mutex::new(crate::communication_policy::CommunicationPolicy::new(
        crate::communication_profile::CommunicationProfile::default(),
        crate::ergonomic_profile::ErgonomicProfile::default(),
    ));
    let ergo_session = Mutex::new(crate::session::SessionManager::new_from_config(
        &crate::config::AppConfig::default(), &crate::ergonomic_profile::ErgonomicProfile::default(),
    ));
    make_read_only(&ergo_file, true);
    let ergo_error = crate::commands_profiles::switch_ergonomic_profile_persisted(
        &app_data, &ergo_store, &ergo_policy, &ergo_session, "strict",
    ).expect_err("read-only Store save must fail");
    assert_eq!(ergo_policy.lock().unwrap().ergo_profile().id, "default");
    assert_eq!(ergo_session.lock().unwrap().state.session_limit_secs, 2400);
    assert_eq!(ergo_session.lock().unwrap().state.stand_limit_secs, 900);
    assert_eq!(ergo_store.get("active_ergonomic_profile").unwrap(), "default");
    assert!(ergo_error.contains("Failed to save store"));
    assert!(ergo_error.contains("rollback save also failed"));
    assert!(ergo_error.contains("disk selection is uncertain"));
    assert_eq!(fs::read(&ergo_file).unwrap(), ergo_before,
        "this injected failure occurs before bytes change");
    let ergo_reload_owner = app();
    let ergo_disk = crate::app_store::initialize_at(ergo_reload_owner.handle(), &ergo_file).unwrap();
    assert_eq!(ergo_disk.get("active_ergonomic_profile").unwrap(), "default");
    assert_eq!(ergo_disk.get("unrelated").unwrap()["keep"], 22);

    make_read_only(&ergo_file, false);
    crate::commands_profiles::switch_ergonomic_profile_persisted(
        &app_data, &ergo_store, &ergo_policy, &ergo_session, "strict",
    ).expect("retry after clearing the injected Store fault must persist");
    assert_eq!(ergo_policy.lock().unwrap().ergo_profile().id, "strict");
    assert_eq!(ergo_session.lock().unwrap().state.session_limit_secs, 1500);
    assert_eq!(ergo_session.lock().unwrap().state.stand_limit_secs, 600);
    let ergo_retry_owner = app();
    let ergo_retry_disk = crate::app_store::initialize_at(ergo_retry_owner.handle(), &ergo_file).unwrap();
    assert_eq!(ergo_retry_disk.get("active_ergonomic_profile").unwrap(), "strict");
    assert_eq!(ergo_retry_disk.get("unrelated").unwrap()["keep"], 22);
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
    assert!(shared.shares_store_with(&again));
    assert!(shared.shares_store_with(app1.state::<crate::app_store::AppStore<MockRuntime>>().inner()));
    assert!(app1.get_store(&file).is_none(), "the exit-writing plugin must not retain a second registered Store");
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
    assert!(!shared.shares_store_with(&reopened));
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
    let value = serde_json::to_value(&restored).unwrap();
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
    for offset in 0..=crate::session_types::DEBOUNCE_COUNT {
        manager.on_reading_at(800, true,
            previous_start + chrono::Duration::seconds(offset as i64 - 5));
    }
    for offset in 0..=crate::session_types::DEBOUNCE_COUNT {
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

#[cfg(windows)]
#[test]
fn failed_app_config_save_restores_store_memory_and_preserves_disk_then_retry() {
    fn make_read_only(path: &std::path::Path, read_only: bool) {
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_readonly(read_only);
        fs::set_permissions(path, permissions).unwrap();
    }

    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("settings.json");
    let owner = app();
    let store = crate::app_store::initialize_at(owner.handle(), &file).unwrap();
    let original = serde_json::json!({
        "sitting_mm": 700,
        "future_config_key": {"keep": true}
    });
    store.set("app_config", original.clone());
    store.set("unrelated", serde_json::json!({"keep": 9}));
    store.save().unwrap();
    let disk_before = fs::read(&file).unwrap();

    make_read_only(&file, true);
    let attempted = crate::config::AppConfig {
        sitting_mm: 765,
        ..Default::default()
    };
    let error = attempted.save(&store).expect_err("read-only Store save must fail");

    assert!(error.contains("Failed to save store"));
    assert!(error.contains("rollback save also failed"));
    assert!(error.contains("disk config is uncertain"));
    assert_eq!(store.get("app_config"), Some(original.clone()),
        "failed save must restore the previous in-memory config value");
    assert_eq!(store.get("unrelated").unwrap()["keep"], 9,
        "rollback must preserve unrelated Store keys");
    assert_eq!(fs::read(&file).unwrap(), disk_before,
        "the injected failure occurs before disk bytes change");

    let reopened_owner = app();
    let reopened = crate::app_store::initialize_at(reopened_owner.handle(), &file).unwrap();
    assert_eq!(reopened.get("app_config"), Some(original.clone()));
    assert_eq!(reopened.get("unrelated").unwrap()["keep"], 9);

    make_read_only(&file, false);
    attempted.save(&store).expect("retry after clearing the Store fault must persist");
    let retry_owner = app();
    let retry = crate::app_store::initialize_at(retry_owner.handle(), &file).unwrap();
    assert_eq!(retry.get("app_config").unwrap()["sitting_mm"], 765);
    assert_eq!(retry.get("unrelated").unwrap()["keep"], 9);

    let absent_file = temp.path().join("absent-config.json");
    let absent_owner = app();
    let absent_store = crate::app_store::initialize_at(absent_owner.handle(), &absent_file).unwrap();
    absent_store.set("unrelated", serde_json::json!({"keep": 12}));
    absent_store.save().unwrap();
    let absent_disk_before = fs::read(&absent_file).unwrap();
    make_read_only(&absent_file, true);
    let absent_error = attempted.save(&absent_store)
        .expect_err("read-only Store save with no prior config must fail");
    assert!(absent_error.contains("Failed to save store"));
    assert!(absent_store.get("app_config").is_none(),
        "rollback must preserve an originally absent config key");
    assert_eq!(absent_store.get("unrelated").unwrap()["keep"], 12);
    assert_eq!(fs::read(&absent_file).unwrap(), absent_disk_before);
    let absent_reopened_owner = app();
    let absent_reopened = crate::app_store::initialize_at(absent_reopened_owner.handle(), &absent_file).unwrap();
    assert!(absent_reopened.get("app_config").is_none());
    assert_eq!(absent_reopened.get("unrelated").unwrap()["keep"], 12);
    make_read_only(&absent_file, false);
    attempted.save(&absent_store).expect("absent-key retry must persist");
    let absent_retry_owner = app();
    let absent_retry = crate::app_store::initialize_at(absent_retry_owner.handle(), &absent_file).unwrap();
    assert_eq!(absent_retry.get("app_config").unwrap()["sitting_mm"], 765);
    assert_eq!(absent_retry.get("unrelated").unwrap()["keep"], 12);
}

#[test]
fn actual_dismiss_state_survives_fresh_store_reload_without_a_sensor_reading() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("settings.json");
    let mut policy = crate::communication_policy::CommunicationPolicy::new(
        crate::communication_profile::CommunicationProfile::default(),
        crate::ergonomic_profile::ErgonomicProfile::default(),
    );
    let session = crate::session::SessionManager::new();
    let owner = app();
    let store = crate::app_store::initialize_at(owner.handle(), &file).unwrap();
    let monotonic_now = std::time::Instant::now();
    let dismissed = crate::tray_controller::apply_user_dismissal_at(
        &mut policy, true, monotonic_now,
    );
    assert!(dismissed);
    assert!(crate::tray_controller::notification_state_needs_save(
        dismissed, false, false,
    ), "the actual tray adapter must save a dismissal without another notification or session event");
    assert!(!crate::tray_controller::notification_state_needs_save(false, false, false));
    assert!(policy.is_snoozed_at(monotonic_now));
    assert!(crate::session_persistence::save_with_policy(&store, &session, &policy, None));
    let fresh_owner = app();
    let fresh_store = crate::app_store::initialize_at(fresh_owner.handle(), &file).unwrap();
    let restored = crate::session_persistence::PersistedEngineState::load(&fresh_store).unwrap();
    let value = serde_json::to_value(&restored).unwrap();
    assert!(value.get("notification_policy").is_some_and(serde_json::Value::is_object),
        "a real dismiss must be represented in the fresh Store snapshot even without another reading");
    let app_data = temp.path().join("app-data");
    crate::profile_loader::ensure_profiles_dir(&app_data);
    let fresh_policy = Mutex::new(crate::communication_policy::CommunicationPolicy::new(
        crate::communication_profile::CommunicationProfile::default(),
        crate::ergonomic_profile::ErgonomicProfile::default(),
    ));
    let fresh_session = Mutex::new(crate::session::SessionManager::new());
    crate::setup_helpers::load_profiles_from_store(
        &app_data, &fresh_store, &fresh_policy, &fresh_session,
    ).unwrap();
    assert!(fresh_policy.lock().unwrap().is_snoozed_at(
        monotonic_now + std::time::Duration::from_secs(1),
    ));

    let voice_snooze_at = monotonic_now + std::time::Duration::from_secs(2);
    fresh_policy.lock().unwrap().snooze_for_at(17, voice_snooze_at);
    assert!(crate::session_persistence::save_with_policy(
        &fresh_store, &fresh_session.lock().unwrap(), &fresh_policy.lock().unwrap(), None,
    ));
    let third_owner = app();
    let third_store = crate::app_store::initialize_at(third_owner.handle(), &file).unwrap();
    let third_policy = Mutex::new(crate::communication_policy::CommunicationPolicy::new(
        crate::communication_profile::CommunicationProfile::default(),
        crate::ergonomic_profile::ErgonomicProfile::default(),
    ));
    let third_session = Mutex::new(crate::session::SessionManager::new());
    crate::setup_helpers::load_profiles_from_store(
        &app_data, &third_store, &third_policy, &third_session,
    ).unwrap();
    assert!(third_policy.lock().unwrap().is_snoozed_at(
        voice_snooze_at + std::time::Duration::from_secs(1),
    ));
}

#[test]
fn checkpoint_refuses_to_overwrite_unsupported_future_snapshot() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("settings.json");
    let owner = app();
    let store = crate::app_store::initialize_at(owner.handle(), &file).unwrap();
    let future = serde_json::json!({
        "schema_version": crate::session_persistence::ENGINE_SCHEMA_VERSION + 1,
        "date_local": chrono::Local::now().format("%Y-%m-%d").to_string(),
        "future_payload": {"preserve": true}
    });
    store.set("persisted_session_state", future.clone());
    store.set("unknown_key", serde_json::json!({"keep": 3}));
    store.save().unwrap();
    let before = fs::read(&file).unwrap();

    let error = crate::session_persistence::PersistedEngineState::from_session(
        &crate::session::SessionManager::new(), None,
    ).save(&store).expect_err("older code must refuse to overwrite a future schema");
    assert!(error.contains("unsupported persisted session state"));
    assert_eq!(store.get("persisted_session_state"), Some(future));
    assert_eq!(store.get("unknown_key").unwrap()["keep"], 3);
    assert_eq!(fs::read(&file).unwrap(), before);

    let fresh_owner = app();
    let fresh = crate::app_store::initialize_at(fresh_owner.handle(), &file).unwrap();
    assert_eq!(fresh.get("persisted_session_state").unwrap()["future_payload"]["preserve"], true);
    assert_eq!(fresh.get("unknown_key").unwrap()["keep"], 3);
}
