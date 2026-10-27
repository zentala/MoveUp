use crate::ui_diagnostics_types::*;

fn script_snapshot(memory: serde_json::Value) -> serde_json::Value {
    script_snapshot_with_setup(memory, "")
}
fn script_snapshot_with_setup(memory: serde_json::Value, setup: &str) -> serde_json::Value {
    let runner = r#"const vm=require('node:vm');const memory=JSON.parse(process.argv[2]);
      const window={getComputedStyle:()=>({display:'block',visibility:'visible',opacity:'1'})};
      if(memory!==null)window.__MOVEUP_UI_DIAGNOSTICS__=memory;
      vm.runInNewContext(process.argv[3],{window});
      const document={readyState:'complete',visibilityState:'visible',getElementById:()=>({childElementCount:1,getBoundingClientRect:()=>({width:420,height:240})})};
      process.stdout.write(JSON.stringify(vm.runInNewContext(process.argv[1],{window,document})));"#;
    let output = std::process::Command::new("node").args(["-e", runner,
        include_str!("ui_diagnostics_probe.js"), &memory.to_string(), setup]).output().unwrap();
    assert!(output.status.success(), "fixed script fixture failed");
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn process_power_capability_survives_window_order_and_history_eviction() {
    use crate::ui_diagnostics::Diagnostics;
    for before in [true, false] {
        for capability in [Capability::Supported, Capability::Unsupported, Capability::RegistrationFailed { code: Some(5) }] {
            let temp = tempfile::tempdir().unwrap();
            let diagnostics = Diagnostics::start(crate::app_lifecycle::RunId::new("test"), |_| Ok(())).unwrap();
            let created = Event { sequence: 0, utc: chrono::Utc::now(), window: WindowLabel::Main,
                generation: Generation { window: 1, navigation: 0 }, kind: EventKind::Created };
            let power = Event { generation: Generation::default(), kind: EventKind::PowerCapability { capability }, ..created.clone() };
            for event in if before { [power, created.clone()] } else { [created.clone(), power] } { assert!(diagnostics.submit(event)); }
            for n in 0..70 { assert!(diagnostics.submit(Event { kind: EventKind::NativeFailure { kind: n, code: None }, ..created.clone() })); }
            let path = diagnostics.report(temp.path().into(), CheckpointObservation::Unavailable, None)
                .unwrap().recv_timeout(std::time::Duration::from_secs(5)).unwrap().unwrap();
            let report: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
            assert_eq!(report["facts"]["power"]["capability"], serde_json::to_value(capability).unwrap());
            if let Capability::RegistrationFailed { code } = capability {
                assert_eq!(report["facts"]["power"]["last_failure"]["operation"], "registration");
                assert_eq!(report["facts"]["power"]["last_failure"]["code"], serde_json::to_value(code).unwrap());
                assert_eq!(report["facts"]["power"]["failure_count"], 1);
            }
            assert_eq!(report["facts"]["counters"]["stale"], 0);
            assert_eq!(report["facts"]["recent"].as_array().unwrap().len(), RECENT_CAPACITY);
        }
    }
}

#[test]
fn process_power_failures_survive_creation_recreation_and_real_report() {
    for operation in [PowerOperation::Dispatch, PowerOperation::Unregistration] {
        for before in [true, false] {
            let temp = tempfile::tempdir().unwrap();
            let diagnostics = crate::ui_diagnostics::Diagnostics::start(crate::app_lifecycle::RunId::new("test"), |_| Ok(())).unwrap();
            let event = Event { sequence: 0, utc: chrono::Utc::now(), window: WindowLabel::Main,
                generation: Generation { window: 1, navigation: 0 }, kind: EventKind::Created };
            assert!(diagnostics.submit(Event { generation: Generation::default(), kind: EventKind::PowerCapability { capability: Capability::Supported }, ..event.clone() }));
            if !before { assert!(diagnostics.submit(event.clone())); }
            crate::ui_diagnostics_power::publish_failure(&diagnostics, operation, Some(5));
            if before { assert!(diagnostics.submit(event.clone())); }
            assert!(diagnostics.submit(Event { kind: EventKind::Destroyed, ..event.clone() }));
            let recreated = Event { generation: Generation { window: 2, navigation: 0 }, ..event };
            assert!(diagnostics.submit(recreated.clone()));
            for n in 0..70 { assert!(diagnostics.submit(Event { kind: EventKind::NativeFailure { kind: n, code: None }, ..recreated.clone() })); }
            let path = diagnostics.report(temp.path().into(), CheckpointObservation::Unavailable, None)
                .unwrap().recv_timeout(std::time::Duration::from_secs(5)).unwrap().unwrap();
            let report: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
            assert_eq!(report["facts"]["power"]["last_failure"]["operation"], serde_json::to_value(operation).unwrap());
            assert_eq!(report["facts"]["power"]["last_failure"]["code"], 5);
            assert_eq!(report["facts"]["power"]["failure_count"], 1);
            assert_eq!(report["facts"]["counters"]["stale"], 0);
            assert!(report["facts"]["recent"].as_array().unwrap().iter().all(|e| e["first"]["kind"]["event"] != "power_failure"));
            assert_eq!(report["facts"]["power"]["capability"]["state"], if operation == PowerOperation::Dispatch { "supported" } else { "registration_failed" });
        }
    }
}

#[cfg(windows)]
#[test]
fn rejected_deadline_dispatch_is_visible_without_timeout_or_physical_release() {
    use crate::{ui_diagnostics::Diagnostics, ui_diagnostics_probe::{Probe, Request}};
    let diagnostics = Diagnostics::start(crate::app_lifecycle::RunId::new("test"), |_| Ok(())).unwrap();
    let generation = Generation { window: 1, navigation: 0 };
    let mut probe = Probe::new(generation);
    let Request::Start(ticket) = probe.request(std::time::Duration::ZERO, true) else { panic!() };
    for kind in [EventKind::Created, EventKind::ProbeStarted { id: ticket.id }] {
        assert!(diagnostics.submit(Event { sequence: 0, utc: chrono::Utc::now(), window: WindowLabel::Main, generation, kind }));
    }
    crate::ui_diagnostics_windows::finish_deadline_dispatch(&diagnostics, WindowLabel::Main, ticket, Err(()));
    let state = diagnostics.snapshot().unwrap().recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    assert!(matches!(state.windows[0].collection, Collection::CollectionFailed { .. }));
    assert!(state.recent.iter().any(|e| matches!(e.first.kind, EventKind::ProbeDeadlineDispatchFailed { id } if id == ticket.id)));
    assert!(!state.recent.iter().any(|e| matches!(e.first.kind, EventKind::ProbeFinished { result: Collection::Timeout, .. })));
    assert_eq!(probe.request(std::time::Duration::from_secs(30), true), Request::Reuse(ticket));
    let received = crate::ui_diagnostics_probe::parse_response(&script_snapshot(memory()).to_string());
    let completed = probe.complete(ticket, received).unwrap();
    assert!(diagnostics.submit(Event { sequence: 0, utc: chrono::Utc::now(), window: WindowLabel::Main,
        generation, kind: EventKind::ProbeFinished { id: ticket.id, result: completed } }));
    let after = diagnostics.snapshot().unwrap().recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    assert!(matches!(after.windows[0].collection, Collection::CollectionFailed { .. }));
    assert_eq!(after.counters.stale, 1);
    assert!(matches!(probe.request(std::time::Duration::from_secs(31), true), Request::Start(_)));
    let count = after.recent.len();
    crate::ui_diagnostics_windows::finish_deadline_dispatch(&diagnostics, WindowLabel::Main, ticket, Ok::<(), ()>(()));
    assert_eq!(diagnostics.snapshot().unwrap().recv_timeout(std::time::Duration::from_secs(5)).unwrap().recent.len(), count);
}

#[test]
fn actual_script_rejects_nonfinite_nullable_values_before_json_coercion() {
    use crate::ui_diagnostics_probe::parse_response;
    for nonfinite in ["NaN", "Infinity", "-Infinity"] {
        for field in ["queries.dashboard.last_result_ms", "queries.dashboard.last_success_ms",
            "queries.today_summary.last_result_ms", "queries.today_summary.last_success_ms",
            "last_error.location.line", "last_error.location.column"] {
            let mut memory = memory();
            memory["queries"]["dashboard"] = serde_json::json!({"outcome":"not_observed","last_result_ms":null,"last_success_ms":null});
            memory["last_error"]["location"]["line"] = serde_json::Value::Null;
            memory["last_error"]["location"]["column"] = serde_json::Value::Null;
            let setup = format!("window.__MOVEUP_UI_DIAGNOSTICS__.{field}={nonfinite};");
            let output = script_snapshot_with_setup(memory, &setup);
            assert!(matches!(parse_response(&output.to_string()), Collection::CollectionFailed { .. }), "accepted nonfinite {field}={nonfinite}");
        }
    }
    assert!(matches!(parse_response(&script_snapshot(memory()).to_string()), Collection::Received(_)));
    let mut nullable = memory();
    nullable["last_error"]["location"]["line"] = serde_json::Value::Null;
    nullable["last_error"]["location"]["column"] = serde_json::Value::Null;
    assert!(matches!(parse_response(&script_snapshot(nullable).to_string()), Collection::Received(_)));
}

#[test]
fn fixed_probe_initial_query_matches_native_outcome() {
    let snapshot = script_snapshot(serde_json::Value::Null);
    assert_eq!(snapshot["queries"]["dashboard"]["outcome"], serde_json::to_value(QueryOutcome::NotObserved).unwrap());
}

#[test]
fn physical_script_slot_survives_timeout_until_its_callback() {
    use crate::ui_diagnostics_probe::{Probe, Request, TIMEOUT, THROTTLE};
    use std::time::Duration;
    let mut probe = Probe::new(Generation { window: 1, navigation: 0 });
    let Request::Start(ticket) = probe.request(Duration::ZERO, false) else { panic!() };
    assert!(!probe.timeout(ticket, TIMEOUT - Duration::from_millis(1)));
    assert!(probe.timeout(ticket, TIMEOUT));
    assert!(!probe.timeout(ticket, TIMEOUT));
    for n in 3..1000 { assert_eq!(probe.request(Duration::from_secs(n), true), Request::Reuse(ticket)); }
    assert_eq!(probe.complete(ticket, Collection::Timeout), None);
    assert_eq!(probe.request(TIMEOUT, false), Request::Throttled);
    assert!(matches!(probe.request(THROTTLE, false), Request::Start(_)));
}

#[test]
fn navigation_power_and_late_callbacks_do_not_release_a_new_script() {
    use crate::ui_diagnostics_probe::{Probe, Request};
    use std::time::Duration;
    let mut probe = Probe::new(Generation { window: 1, navigation: 0 });
    let Request::Start(old) = probe.request(Duration::ZERO, true) else { panic!() };
    probe.invalidate(Generation { window: 1, navigation: 1 });
    assert_eq!(probe.request(Duration::from_secs(1), true), Request::Reuse(old));
    assert_eq!(probe.complete(old, Collection::CollectionFailed { code: Some(1) }), None);
    let Request::Start(current) = probe.request(Duration::from_secs(1), true) else { panic!() };
    assert_ne!(current.id, old.id);
    assert_eq!(probe.complete(old, Collection::Timeout), None);
    assert_eq!(probe.request(Duration::from_secs(2), true), Request::Reuse(current));
    probe.power(true);
    assert_eq!(probe.request(Duration::from_secs(2), true), Request::Suspended);
    probe.power(false);
    assert_eq!(probe.request(Duration::from_secs(4), true), Request::Reuse(current));
    assert_eq!(probe.complete(current, Collection::Timeout), None);
    let Request::Start(next) = probe.request(Duration::from_secs(4), true) else { panic!() };
    assert_eq!(probe.complete(next, Collection::CollectionFailed { code: Some(-9) }), Some(Collection::CollectionFailed { code: Some(-9) }));
}

#[test]
fn power_signal_and_registration_cleanup_are_explicit_and_idempotent() {
    use crate::ui_diagnostics_power::{PowerSignal, PowerLease};
    let signal = PowerSignal::default();
    assert_eq!(signal.receive(9999), None);
    assert_eq!(signal.receive(4), Some(true));
    assert_eq!(signal.receive(4), None);
    assert_eq!(signal.receive(18), Some(false));
    assert_eq!(signal.receive(7), None);
    signal.deactivate();
    assert_eq!(signal.receive(4), None);
    let mut lease = PowerLease::new(123);
    assert_eq!(lease.close(|id| { assert_eq!(id, 123); 5 }), Err(5));
    assert_eq!(lease.close(|id| { assert_eq!(id, 123); 0 }), Ok(()));
    assert_eq!(lease.close(|_| panic!("unregistered twice")), Ok(()));
}

fn memory() -> serde_json::Value {
    serde_json::json!({ "schema_version":1,"boot":"mounted","view":"settings",
        "queries": { "dashboard": { "outcome":"failed","last_result_ms":1,"last_success_ms":9 },
            "today_summary": { "outcome":"not_observed","last_result_ms":null,"last_success_ms":null } },
        "last_error": { "source":"render","kind":"TypeError",
            "location": { "module":"App.tsx","line":1,"column":2 } } })
}

#[test]
fn actual_fixed_script_and_native_schema_keep_only_version1_safe_fields() {
    use crate::ui_diagnostics_probe::parse_response;
    let mut memory = memory();
    memory["last_error"]["message"] = "secret query contents".into();
    let result = script_snapshot(memory);
    assert!(!result.to_string().contains("secret"));
    let Collection::Received(page) = parse_response(&result.to_string()) else { panic!("valid script response rejected") };
    assert_eq!(page.queries.dashboard.last_success_ms, Some(9)); // rollback is not inferred failure
    assert_eq!(page.queries.dashboard.last_result_ms, Some(1));
    assert_eq!(page.last_error.unwrap().location.unwrap().module, "App.tsx");
    let Collection::Received(absent) = parse_response(&script_snapshot(serde_json::Value::Null).to_string()) else { panic!() };
    assert!(!absent.diagnostics_available);
    assert_eq!(absent.boot, BootStage::Unknown);
    assert!(absent.root_present);
}

#[test]
fn malformed_present_snapshot_and_wrong_schema_never_become_valid_defaults() {
    use crate::ui_diagnostics_probe::parse_response;
    let mut cases = vec![serde_json::json!({"schema_version":2}), serde_json::json!({"schema_version":1})];
    for field in ["boot", "queries", "last_error"] { let mut value = memory(); value.as_object_mut().unwrap().remove(field); cases.push(value); }
    for memory in cases { assert!(matches!(parse_response(&script_snapshot(memory).to_string()), Collection::CollectionFailed { .. })); }
    let valid = script_snapshot(memory());
    for (pointer, invalid) in [("/width_px", serde_json::json!(100001)), ("/queries/dashboard/last_result_ms", serde_json::json!(-1)),
        ("/last_error/location/module", serde_json::json!("https://secret/path.js")),
        ("/last_error/location/column", serde_json::Value::Null), ("/last_error/kind", serde_json::json!("secret error text"))] {
        let mut value = valid.clone(); *value.pointer_mut(pointer).unwrap() = invalid;
        assert!(matches!(parse_response(&value.to_string()), Collection::CollectionFailed { .. }));
    }
    let mut value = valid.clone(); value.as_object_mut().unwrap().remove("last_error");
    assert!(matches!(parse_response(&value.to_string()), Collection::CollectionFailed { .. }));
    assert!(matches!(parse_response(&"x".repeat(REPORT_LIMIT + 1)), Collection::CollectionFailed { code: Some(-1) }));
    let mut maximum = valid; maximum["last_error"]["location"]["module"] = format!("{}.MJSX", "A".repeat(128)).into();
    assert!(matches!(parse_response(&maximum.to_string()), Collection::Received(_)));
}

#[cfg(windows)]
#[test]
fn native_acquisition_failure_reaches_actual_diagnostics_state() {
    let diagnostics = crate::ui_diagnostics::Diagnostics::start(crate::app_lifecycle::RunId::new("test"), |_| Ok(())).unwrap();
    crate::ui_diagnostics_windows::acquisition_failed(&diagnostics, WindowLabel::Main, 1, -42);
    let state = diagnostics.snapshot().unwrap().recv_timeout(std::time::Duration::from_secs(5)).unwrap();
    assert_eq!(state.windows[0].present, Some(true));
    assert_eq!(state.windows[0].process_events, Capability::RegistrationFailed { code: Some(-42) });
    assert!(state.recent.iter().any(|event| matches!(event.first.kind, EventKind::AdapterFailure { operation: AdapterOperation::Acquisition, code: Some(-42) })));
}

#[cfg(windows)]
#[test]
fn shared_environment_membership_survives_other_window_and_stale_cleanup() {
    use crate::ui_diagnostics_windows::EnvironmentMembers;
    let mut members = EnvironmentMembers::default();
    members.attach(WindowLabel::Main, 1);
    members.attach(WindowLabel::Main, 1);
    members.attach(WindowLabel::Analyst, 1);
    assert_eq!(members.owners().len(), 2);
    assert!(!members.detach(WindowLabel::Main, 1));
    members.attach(WindowLabel::Analyst, 2);
    assert!(!members.detach(WindowLabel::Analyst, 1));
    assert_eq!(members.owners(), vec![(WindowLabel::Analyst, 2)]);
    assert!(members.detach(WindowLabel::Analyst, 2));
}

#[cfg(windows)]
#[test]
fn unsupported_interface_is_distinct_from_registration_failure() {
    use crate::ui_diagnostics_windows::registration_failure;
    assert_eq!(registration_failure(-2147467262), Capability::Unsupported);
    assert_eq!(registration_failure(-2147467263), Capability::Unsupported);
    assert_eq!(registration_failure(-42), Capability::RegistrationFailed { code: Some(-42) });
}
