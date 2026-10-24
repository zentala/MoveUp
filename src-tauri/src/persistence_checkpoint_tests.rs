use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc,
    },
    thread,
    time::Duration,
};

use super::persistence_checkpoint::CheckpointWorker;

fn mock_app() -> tauri::App<tauri::test::MockRuntime> {
    tauri::test::mock_builder()
        .plugin(tauri_plugin_store::Builder::default().build())
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .unwrap()
}

#[test]
fn independent_worker_checkpoints_without_sensor_or_serial_events() {
    let worker = CheckpointWorker::default();
    let checkpoints = Arc::new(AtomicUsize::new(0));
    let count = checkpoints.clone();
    assert!(worker.start(Duration::from_millis(10), move || {
        count.fetch_add(1, Ordering::SeqCst);
    }));

    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while checkpoints.load(Ordering::SeqCst) == 0 && std::time::Instant::now() < deadline {
        thread::sleep(Duration::from_millis(5));
    }
    assert!(checkpoints.load(Ordering::SeqCst) > 0, "timer runs without any serial input callback");
    worker.stop().unwrap();
    let stopped_at = checkpoints.load(Ordering::SeqCst);
    thread::sleep(Duration::from_millis(30));
    assert_eq!(checkpoints.load(Ordering::SeqCst), stopped_at, "stop joins the sole periodic worker");
}

#[test]
fn shutdown_waits_for_an_in_flight_checkpoint_before_flushing() {
    let worker = Arc::new(CheckpointWorker::default());
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (stopped_tx, stopped_rx) = mpsc::channel();
    let worker_task = worker.clone();
    worker.start(Duration::from_millis(5), move || {
        entered_tx.send(()).unwrap();
        release_rx.recv().unwrap();
    });
    entered_rx.recv_timeout(Duration::from_secs(2)).expect("checkpoint began");

    thread::spawn(move || {
        let result = worker_task.stop();
        stopped_tx.send(result).unwrap();
    });
    assert!(stopped_rx.recv_timeout(Duration::from_millis(30)).is_err(),
        "shutdown must wait until the active save leaves its critical section");
    release_tx.send(()).unwrap();
    stopped_rx.recv_timeout(Duration::from_secs(2)).unwrap().unwrap();
}

#[test]
fn production_checkpoint_attempt_reports_failure_then_retry_recovery_once() {
    use std::{fs, sync::Arc};
    use tauri::Manager;

    let temp = tempfile::tempdir().unwrap();
    let settings = temp.path().join("settings.json");
    let app = mock_app();
    let lifecycle = Arc::new(crate::app_lifecycle::AppLifecycle::new(
        "test", temp.path().join("early.log"),
    ));
    lifecycle.attach_logger(
        Arc::new(crate::event_logger::EventLogger::new(temp.path().join("logs"))),
        crate::app_lifecycle_store::PreviousRun::None,
    );
    app.manage(lifecycle.clone());
    let store = crate::app_store::initialize_at(app.handle(), &settings).unwrap();
    store.set("unknown_key", serde_json::json!({"keep": true}));
    store.save().unwrap();

    // A directory at the settings filename makes the real atomic replacement
    // fail after snapshot serialization, without touching any live app data.
    fs::remove_file(&settings).unwrap();
    fs::create_dir(&settings).unwrap();
    let session = crate::session::SessionManager::new();
    let policy = crate::communication_policy::CommunicationPolicy::new(
        crate::communication_profile::CommunicationProfile::default(),
        crate::ergonomic_profile::ErgonomicProfile::default(),
    );
    let now = chrono::Utc::now();
    let failed = crate::session_persistence::run_checkpoint_attempt(app.handle(), now, || {
        crate::session_persistence::PersistedEngineState::from_session_with_policy(
            &session, None, &policy, now, std::time::Instant::now(),
        ).save(&store)
    });
    assert!(failed.is_err(), "the production checkpoint attempt returns the actual persistence error");
    {
        let status = lifecycle.checkpoint_status.lock().unwrap();
        assert!(status.last_success_utc.is_none());
        assert!(status.last_error.as_deref().is_some_and(|error| error.contains("replace settings file")));
    }

    fs::remove_dir(&settings).unwrap();
    let recovered_at = now + chrono::Duration::seconds(1);
    crate::session_persistence::run_checkpoint_attempt(app.handle(), recovered_at, || {
        crate::session_persistence::PersistedEngineState::from_session_with_policy(
            &session, None, &policy, recovered_at, std::time::Instant::now(),
        ).save(&store)
    }).expect("the same writer succeeds once the disposable target is available");
    {
        let status = lifecycle.checkpoint_status.lock().unwrap();
        assert!(status.last_success_utc.is_some());
        assert!(status.last_error.is_none(), "successful retry clears the current failure");
    }

    let day = chrono::Local::now().format("%Y-%m-%d").to_string();
    let events = fs::read_to_string(temp.path().join("logs").join(day).join("events.log")).unwrap();
    assert_eq!(events.matches("stage=persistence_checkpoint_failed").count(), 1);
    assert_eq!(events.matches("stage=persistence_checkpoint_recovered").count(), 1);

    let fresh_app = mock_app();
    let fresh_store = crate::app_store::initialize_at(fresh_app.handle(), &settings).unwrap();
    let snapshot = crate::session_persistence::PersistedEngineState::load(&fresh_store).unwrap();
    assert!(snapshot.engine.is_some(), "retry publishes a complete snapshot visible to a fresh Store reader");
    assert_eq!(fresh_store.get("unknown_key").unwrap()["keep"], true);
}
