use crate::ui_diagnostics::*;
use crate::ui_diagnostics_types::*;
use std::sync::{mpsc, Arc};
#[test]
fn r6_old_collection_failure_cannot_clear_new_navigation_probe() {
    let handle = Diagnostics::start(identity(), |_| Ok(())).unwrap();
    handle.submit(event(1, EventKind::Created));
    handle.submit(event(2, EventKind::ProbeStarted { id: 1 }));
    let mut navigation = event(3, EventKind::NavigationStarted); navigation.generation.navigation = 1;
    handle.submit(navigation.clone());
    navigation.kind = EventKind::ProbeStarted { id: 2 }; handle.submit(navigation);
    handle.collection_failed_at(WindowLabel::Main, Some((event(1, EventKind::Created).generation, Some(1))));
    let state = handle.snapshot().unwrap().recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    assert_eq!(state.windows[0].active_probe, Some(2));
}
#[test]
fn r6_incomplete_publication_keeps_path_and_explicit_collection_warning() {
    let result = written_report(std::path::PathBuf::from("report.json"), true);
    let (title, body) = confirmation(&result);
    assert!(title.contains("incomplete")); assert!(body.contains("report.json"));
}
#[test]
fn r6_collection_failure_reaches_actual_background_sink() {
    let lines = Arc::new(std::sync::Mutex::new(Vec::<String>::new())); let sink = lines.clone();
    let handle = Diagnostics::start(identity(), move |line| { sink.lock().unwrap().push(line.into()); Ok(()) }).unwrap();
    handle.collection_failed(WindowLabel::Main);
    let _ = handle.snapshot().unwrap().recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    assert!(lines.lock().unwrap().iter().any(|line| line.contains("dispatch") || line.contains("capture_failed")));
}
#[cfg(windows)]
#[test]
fn r6_show_focus_results_survive_missing_com_entry() {
    let handle = Diagnostics::start(identity(), |_| Ok(())).unwrap();
    crate::ui_diagnostics_windows::publish_operation(&handle, WindowLabel::Main, WindowOperation::Show, false,
        Some((WindowFailureCategory::Io, Some(5))));
    let state = handle.snapshot().unwrap().recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    assert!(state.recent.iter().any(|e| matches!(e.first.kind, EventKind::Show { succeeded: false })));
    assert!(state.recent.iter().any(|e| matches!(e.first.kind, EventKind::WindowOperationFailed { code: Some(5), .. })));
}
#[test]
fn welcome_activation_uses_each_bounded_show_and_focus_result() {
    let success = WindowActionResults { show: Ok(()), focus: Ok(()) };
    assert!(success.succeeded());

    let show_failed = WindowActionResults {
        show: Err(tauri::Error::Io(std::io::Error::from_raw_os_error(5))),
        focus: Ok(()),
    };
    assert!(!show_failed.succeeded());
    assert_eq!(native_failure(show_failed.show.as_ref().unwrap_err()), (WindowFailureCategory::Io, Some(5)));

    let focus_failed = WindowActionResults {
        show: Ok(()),
        focus: Err(tauri::Error::Io(std::io::Error::from_raw_os_error(5))),
    };
    assert!(!focus_failed.succeeded());
    assert_eq!(native_failure(focus_failed.focus.as_ref().unwrap_err()), (WindowFailureCategory::Io, Some(5)));
}
#[test]
fn r6_actual_probe_settles_navigation_and_power_waiters_without_releasing_physical_slot() {
    use crate::ui_diagnostics_probe::{Probe, Request};
    use std::time::Duration;
    for power in [false, true] {
        let mut probe = Probe::new(Generation { window: 1, navigation: 0 });
        let Request::Start(ticket) = probe.request(Duration::ZERO, true) else { panic!() };
        let (tx, mut rx) = tokio::sync::oneshot::channel(); probe.watch(tx);
        if power { probe.power_observed(true, |_| {}); probe.power_observed(false, |_| {}); }
        else { probe.invalidate_observed(Generation { window: 1, navigation: 1 }, |_| {}); }
        assert_eq!(rx.try_recv().unwrap(), (None, false));
        assert_eq!(probe.request(Duration::from_secs(4), true), Request::Reuse(ticket));
        assert!(!probe.timeout(ticket, Duration::from_secs(4)));
        assert!(probe.complete(ticket, Collection::Timeout).is_none());
        assert!(matches!(probe.request(Duration::from_secs(5), true), Request::Start(_)));
    }
}
#[test]
fn interruption_is_queued_before_capture_can_publish_actual_report() {
    use crate::ui_diagnostics_probe::{Probe, Request};
    use std::time::Duration;
    for transition in 0..3 {
        let temp = tempfile::tempdir().unwrap();
        let handle = Diagnostics::start(identity(), |_| Ok(())).unwrap();
        handle.submit(event(1, EventKind::Created));
        let mut probe = Probe::new(event(1, EventKind::Created).generation);
        let Request::Start(ticket) = probe.request(Duration::ZERO, true) else { panic!() };
        handle.submit(event(2, EventKind::ProbeStarted { id: ticket.id }));
        let (tx, mut rx) = tokio::sync::oneshot::channel(); probe.watch(tx);
        let mut premature_report = None;
        let mut publish = |generation| {
            // Exercise the exact capture/report schedule available when settlement is early.
            if rx.try_recv().is_ok() {
                premature_report = Some(handle.report(temp.path().into(), CheckpointObservation::Unavailable, None)
                    .unwrap().recv_timeout(Duration::from_secs(5)).unwrap().unwrap());
            }
            let kind = match transition { 0 => EventKind::NavigationStarted, 1 => EventKind::Suspend, _ => EventKind::Resume };
            handle.submit(Event { generation, kind, ..event(3, EventKind::Resume) });
        };
        if transition == 0 { probe.invalidate_observed(Generation { navigation: 1, ..ticket.generation }, &mut publish); }
        else { probe.power_observed(transition == 1, &mut publish); }
        assert!(premature_report.is_none(), "capture woke before the actual interruption reached the report queue");
        assert_eq!(rx.try_recv().unwrap(), (None, false));
        let path = handle.report(temp.path().into(), CheckpointObservation::Unavailable, None)
            .unwrap().recv_timeout(Duration::from_secs(5)).unwrap().unwrap();
        let report: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(report["facts"]["windows"][0]["collection"]["state"], "interrupted");
        assert!(report["facts"]["windows"][0]["active_probe"].is_null());
        assert_eq!(probe.request(Duration::from_secs(4), true),
            if transition == 1 { Request::Suspended } else { Request::Reuse(ticket) });
        assert!(probe.complete(ticket, Collection::Timeout).is_none());
    }
}
#[test]
fn r6_failed_sink_and_stale_capture_provenance_survive_real_report() {
    let temp = tempfile::tempdir().unwrap();
    let handle = Diagnostics::start(identity(), |_| Err(std::io::ErrorKind::PermissionDenied.into())).unwrap();
    handle.submit(event(1, EventKind::Created));
    handle.submit(event(2, EventKind::ProbeStarted { id: 1 }));
    handle.submit(event(3, EventKind::Resume));
    handle.collection_failed_at(WindowLabel::Main, Some((event(1, EventKind::Created).generation, Some(1))));
    let mut recreated = event(4, EventKind::Created); recreated.generation.window = 2;
    handle.submit(recreated);
    handle.collection_failed_at(WindowLabel::Main, Some((event(1, EventKind::Created).generation, Some(1))));
    let path = handle.report(temp.path().into(), CheckpointObservation::Unavailable, None).unwrap()
        .recv_timeout(std::time::Duration::from_secs(5)).unwrap().unwrap();
    let report: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(report["facts"]["windows"][0]["generation"]["window"], 2);
    assert_eq!(report["facts"]["windows"][0]["collection"]["state"], "not_requested");
    assert_eq!(report["facts"]["writer"]["state"], "failed");
    assert!(report["facts"]["counters"]["write_failures"].as_u64().unwrap() >= 4);
    assert!(report["facts"]["recent"].as_array().unwrap().iter().any(|e|
        e["first"]["kind"]["event"] == "capture_failed" && e["applied"] == false));
}
#[test]
fn tray_report_action_is_reachable_without_a_working_frontend() {
    assert!(crate::tray::tray_menu_item_ids().contains(&"save-ui-diagnostics"));
}
#[test]
fn initial_presence_is_unknown_until_actual_absence_is_observed() {
    let mut state = DiagnosticState::new(identity());
    assert!(serde_json::to_value(&state.windows[0]).unwrap()["present"].is_null());
    assert_eq!(state.windows[0].collection, Collection::NotRequested);
    let mut absence = event(1, EventKind::Destroyed); absence.generation = Generation::default(); state.observe(absence);
    assert_eq!(serde_json::to_value(&state.windows[0]).unwrap()["present"], false); assert_eq!(state.windows[0].collection, Collection::NotPresent);
}
fn event(sequence: u64, kind: EventKind) -> Event {
    Event { sequence, utc: chrono::Utc::now(), window: WindowLabel::Main,
        generation: Generation { window: 1, navigation: 0 }, kind }
}
#[test]
fn blocked_writer_does_not_block_callbacks_and_overflow_is_exact() {
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (drained_tx, drained_rx) = mpsc::channel();
    let mut first = true;
    let mut writes = 0;
    let handle = Diagnostics::start(identity(), move |_: &str| {
        if first {
            first = false;
            entered_tx.send(()).unwrap();
            release_rx.recv().unwrap();
        }
        writes += 1;
        if writes == QUEUE_CAPACITY + 1 { drained_tx.send(()).unwrap(); }
        Ok(())
    }).unwrap();
    assert!(handle.submit(event(1, EventKind::Created)));
    entered_rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    // The consumer is held until after all producer calls have completed.
    let producer = handle.clone();
    let (finished_tx, finished_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let admitted: Vec<_> = (0..QUEUE_CAPACITY).map(|n| producer.submit(event(n as u64 + 2,
            EventKind::NativeFailure { kind: n as u32, code: None }))).collect();
        let overflow = producer.submit(event(200, EventKind::Show { succeeded: false }));
        let _ = finished_tx.send((admitted, overflow, producer.dropped()));
    });
    let finished = finished_rx.recv_timeout(std::time::Duration::from_secs(5));
    // Release even on failure so a blocking-send regression cannot strand a worker.
    release_tx.send(()).unwrap();
    let (admitted, overflow, dropped) = finished.expect("producer waited for blocked writer");
    assert!(admitted.into_iter().all(|ok| ok));
    assert!(!overflow);
    assert_eq!(dropped, 1);
    drained_rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    let snapshot = handle.snapshot().unwrap().recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    assert_eq!(snapshot.recent.len(), RECENT_CAPACITY);
    assert_eq!(snapshot.counters.dropped, 1);
}
fn identity() -> crate::app_lifecycle::RunId {
    crate::app_lifecycle::RunId { run_id: "test-run".into(), pid: 1,
        version: "test".into(), build_mode: "test".into(), supervisor_run_id: None }
}
#[test]
fn repeated_events_retain_first_receipt_and_truthful_counts() {
    let mut state = DiagnosticState::new(identity());
    state.observe(event(1, EventKind::Created));
    for n in 2..=100 { state.observe(event(n, EventKind::Show { succeeded: false })); }
    assert_eq!(state.recent.len(), 2);
    let group = state.recent.back().unwrap();
    assert_eq!((group.first.sequence, group.last_sequence, group.occurrences), (2, 100, 99));
    assert_eq!(state.counters.coalesced, 98);
}

#[test]
fn old_navigation_and_probe_ids_cannot_replace_current_observations() {
    let mut state = DiagnosticState::new(identity());
    state.observe(event(1, EventKind::Created));
    state.observe(event(2, EventKind::ProbeStarted { id: 1 }));
    let mut navigation = event(3, EventKind::NavigationStarted);
    navigation.generation.navigation = 1;
    state.observe(navigation.clone());
    state.observe(event(4, EventKind::ProbeFinished { id: 1, result: Collection::Timeout }));
    assert_eq!(state.windows[0].collection, Collection::Interrupted);
    navigation.sequence = 5;
    navigation.kind = EventKind::ProbeStarted { id: 2 };
    state.observe(navigation.clone());
    navigation.kind = EventKind::ProbeFinished { id: 1, result: Collection::Timeout };
    state.observe(navigation.clone());
    assert_eq!(state.counters.stale, 2);
    assert_eq!(state.windows[0].active_probe, Some(2));
    navigation.kind = EventKind::ProbeFinished { id: 2, result: Collection::Timeout };
    state.observe(navigation);
    assert_eq!(state.windows[0].collection, Collection::Timeout);
    let mut next = event(7, EventKind::NavigationStarted);
    next.generation.navigation = 2;
    state.observe(next);
    assert_eq!(state.windows[0].collection, Collection::NotRequested);
}

#[test]
fn sink_failure_is_recorded_and_later_success_is_recovery() {
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let seen = calls.clone();
    let handle = Diagnostics::start(identity(), move |_: &str| {
        if seen.fetch_add(1, std::sync::atomic::Ordering::Relaxed) == 0 {
            Err(std::io::ErrorKind::PermissionDenied.into())
        } else { Ok(()) }
    }).unwrap();
    handle.submit(event(1, EventKind::Created));
    let failed = handle.snapshot().unwrap().recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    assert_eq!(failed.writer, WriteState::Failed { kind: IoFailure::PermissionDenied });
    assert_eq!(failed.counters.write_failures, 1);
    handle.submit(event(2, EventKind::Show { succeeded: true }));
    let recovered = handle.snapshot().unwrap().recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    assert_eq!(recovered.writer, WriteState::Written);
    assert_eq!(recovered.counters.write_failures, 1);
}

#[test]
fn every_collection_and_capability_state_stays_distinct_in_reports() {
    let variants = [Collection::Received(PageObservation { schema_version: 1, ready: DocumentReady::Complete,
        visible: true, diagnostics_available: true, boot: BootStage::Mounted, view: ViewCategory::Widget,
        root_present: true, root_children: 1, width_px: 420, height_px: 240, display_none: false,
        visibility_hidden: false, zero_opacity: false, queries: Queries { dashboard: QueryHint::unobserved(), today_summary: QueryHint::unobserved() }, last_error: None }),
        Collection::Timeout, Collection::NotRequested, Collection::NotPresent,
        Collection::Unsupported, Collection::Interrupted, Collection::CollectionFailed { code: Some(-1) }];
    let mut encoded = std::collections::HashSet::new();
    for result in variants {
        let mut state = DiagnosticState::new(identity());
        state.observe(event(1, EventKind::Created));
        state.observe(event(2, EventKind::ProbeStarted { id: 1 }));
        state.observe(event(3, EventKind::ProbeFinished { id: 1, result: result.clone() }));
        assert_eq!(state.windows[0].collection, result);
        encoded.insert(serde_json::to_string(&state.windows[0].collection).unwrap());
    }
    assert_eq!(encoded.len(), 7);
    for capability in [Capability::NotRequested, Capability::Supported, Capability::Unsupported,
        Capability::RegistrationFailed { code: Some(-2) }] {
        let mut state = DiagnosticState::new(identity());
        state.observe(event(1, EventKind::Created));
        state.observe(event(2, EventKind::ProcessCapability { capability }));
        assert_eq!(state.windows[0].process_events, capability);
    }
}

#[test]
fn suspend_and_recreation_invalidate_late_callbacks() {
    let mut state = DiagnosticState::new(identity());
    state.observe(event(1, EventKind::Created));
    state.observe(event(2, EventKind::ProbeStarted { id: 10 }));
    state.observe(event(3, EventKind::Suspend));
    state.observe(event(4, EventKind::ProbeFinished { id: 10, result: Collection::Timeout }));
    assert_eq!(state.windows[0].collection, Collection::Interrupted);
    let mut recreated = event(5, EventKind::Created);
    recreated.generation.window = 2;
    state.observe(recreated);
    state.observe(event(6, EventKind::Destroyed));
    assert_eq!(state.windows[0].present, Some(true));
    assert_eq!(state.windows[0].collection, Collection::NotRequested);
    assert_eq!(state.counters.stale, 2);
}

#[test]
fn report_publication_never_overwrites_and_cleans_only_its_temporary_file() {
    use crate::ui_diagnostics_store::publish;
    let temp = tempfile::tempdir().unwrap();
    let foreign = temp.path().join("foreign.tmp");
    std::fs::write(&foreign, "keep").unwrap();
    let path = publish(temp.path(), "report.json", b"first").unwrap();
    let error = publish(temp.path(), "report.json", b"second").unwrap_err();
    assert_eq!(error.stage, "publish");
    assert_eq!(error.kind, IoFailure::AlreadyExists);
    assert_eq!(std::fs::read(path).unwrap(), b"first");
    assert_eq!(std::fs::read(foreign).unwrap(), b"keep");
    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 2);
    assert!(publish(&temp.path().join("absent"), "report.json", b"{}").is_err());
}

#[test]
fn serialization_caps_bytes_before_publication() {
    use crate::ui_diagnostics_store::{serialize, save_report};
    assert_eq!(serialize(&"x".repeat(REPORT_LIMIT - 2)).unwrap().len(), REPORT_LIMIT);
    assert_eq!(serialize(&"x".repeat(REPORT_LIMIT - 1)).unwrap_err().stage, "report_too_large");
    let temp = tempfile::tempdir().unwrap();
    let report = Report { collected_utc: chrono::Utc::now(), facts: DiagnosticState::new(identity()),
        checkpoint: CheckpointObservation::Unavailable, webview_runtime_version: None,
        hypotheses: vec!["x".repeat(REPORT_LIMIT)] };
    assert_eq!(save_report(temp.path(), &report).unwrap_err().stage, "report_too_large");
    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 0);
}

#[test]
fn existing_logger_failure_reaches_worker_without_claiming_success() {
    let temp = tempfile::tempdir().unwrap();
    let blocker = temp.path().join("file");
    std::fs::write(&blocker, b"block").unwrap();
    let logger = Arc::new(crate::event_logger::EventLogger::new(blocker.join("logs")));
    let handle = Diagnostics::with_logger(identity(), logger).unwrap();
    handle.submit(event(1, EventKind::Created));
    let state = handle.snapshot().unwrap().recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    assert!(matches!(state.writer, WriteState::Failed { .. }));
    assert_eq!(state.counters.write_failures, 1);
}

#[test]
fn background_report_writes_actual_json_and_retains_export_failures() {
    let temp = tempfile::tempdir().unwrap();
    let handle = Diagnostics::start(identity(), |_: &str| Ok(())).unwrap();
    let path = handle.report(temp.path().into(), CheckpointObservation::Unavailable, None)
        .unwrap().recv_timeout(std::time::Duration::from_secs(5)).unwrap().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(json["facts"]["windows"][1]["collection"]["state"], "not_requested");
    assert_eq!(json["facts"]["identity"]["run_id"], "test-run");
    assert_eq!(path.parent().unwrap().parent().unwrap(), temp.path());
    let error = handle.report(path.join("invalid"), CheckpointObservation::Contended, None)
        .unwrap().recv_timeout(std::time::Duration::from_secs(5)).unwrap().unwrap_err();
    assert_eq!(error.stage, "day_directory");
    let state = handle.snapshot().unwrap().recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    assert_eq!(state.counters.report_failures, 1);
    assert!(matches!(state.report_writer, WriteState::Failed { .. }));
    assert!(matches!(handle.report(temp.path().into(), CheckpointObservation::Unavailable,
        Some("x".repeat(129))), Err(RequestFailure::InvalidMetadata)));
}

#[test]
fn concurrent_report_publication_has_exactly_one_winner() {
    let temp = tempfile::tempdir().unwrap();
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let workers: Vec<_> = [b"first".as_slice(), b"second".as_slice()].into_iter().map(|bytes| {
        let directory = temp.path().to_path_buf(); let barrier = barrier.clone();
        std::thread::spawn(move || { barrier.wait();
            crate::ui_diagnostics_store::publish(&directory, "race.json", bytes) })
    }).collect();
    let results: Vec<_> = workers.into_iter().map(|worker| worker.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
    assert!(matches!(std::fs::read(temp.path().join("race.json")).unwrap().as_slice(), b"first" | b"second"));
}

#[test]
fn checkpoint_reader_is_nonblocking_and_does_not_expose_raw_errors() {
    let temp = tempfile::tempdir().unwrap();
    let lifecycle = crate::app_lifecycle::AppLifecycle::new("test", temp.path().join("early.log"));
    let mut guard = lifecycle.checkpoint_status.lock().unwrap();
    guard.last_error = Some("private path and raw error".into());
    assert!(matches!(checkpoint(Some(&lifecycle)), CheckpointObservation::Contended));
    drop(guard);
    let json = serde_json::to_string(&checkpoint(Some(&lifecycle))).unwrap();
    assert!(json.contains("\"last_save_failed\":true"));
    assert!(!json.contains("private"));
    assert!(matches!(checkpoint(None), CheckpointObservation::Unavailable));
}

#[test]
fn worker_never_persists_rejected_current_or_stale_probe_payloads() {
    for stale in [0, 1, 2, 3] {
        let valid = stale == 3;
        let temp = tempfile::tempdir().unwrap();
        let lines = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let sink = lines.clone();
        let handle = Diagnostics::start(identity(), move |line| { sink.lock().unwrap().push(line.into()); Ok(()) }).unwrap();
        assert!(handle.submit(event(1, EventKind::Created)));
        assert!(handle.submit(event(2, EventKind::ProbeStarted { id: 1 })));
        let mut page = PageObservation { schema_version: 1, ready: DocumentReady::Complete, visible: true, diagnostics_available: true,
            boot: BootStage::Mounted, view: ViewCategory::Widget, root_present: true, root_children: 100003,
            width_px: 100001, height_px: 100002, display_none: false, visibility_hidden: false,
            zero_opacity: false, queries: Queries { dashboard: QueryHint { outcome: QueryOutcome::Failed, last_result_ms: Some(8_640_000_000_000_001), last_success_ms: None }, today_summary: QueryHint::unobserved() }, last_error: None };
        if valid { page.root_children = 1; page.width_px = 420; page.height_px = 240; page.queries.dashboard.last_result_ms = Some(123); }
        let mut rejected = event(3, EventKind::ProbeFinished { id: if stale == 2 { 2 } else { 1 }, result: Collection::Received(page) });
        if stale == 1 { rejected.generation.window = 0; }
        assert!(handle.submit(rejected));
        let path = handle.report(temp.path().into(), CheckpointObservation::Unavailable, None)
            .unwrap().recv_timeout(std::time::Duration::from_secs(5)).unwrap().unwrap();
        let report = std::fs::read_to_string(path).unwrap();
        let sink = lines.lock().unwrap().join("\n");
        for persisted in [&report, &sink] {
            assert_eq!(persisted.contains("\"root_children\""), valid, "payload validation lost valid data or retained rejected data");
            for invalid in ["100001", "100002", "100003", "8640000000000001"] { assert!(!persisted.contains(invalid)); }
            assert!(persisted.contains(if valid { "\"width_px\":420" } else { "collection_failed" }));
            assert!(persisted.contains(if valid { "\"payload_rejected\":false" } else { "\"payload_rejected\":true" }));
        }
        let json: serde_json::Value = serde_json::from_str(&report).unwrap();
        assert_eq!(json["facts"]["counters"]["invalid"], u64::from(!valid));
        assert_eq!(json["facts"]["counters"]["stale"], u64::from(stale == 1 || stale == 2));
        assert_eq!(json["facts"]["recent"][2]["applied"], valid);
        assert_eq!(json["facts"]["windows"][0]["collection"]["state"], if valid { "received" } else if stale == 0 { "collection_failed" } else { "not_requested" });
    }
}
