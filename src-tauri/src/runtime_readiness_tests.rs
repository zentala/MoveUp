use crate::{app_lifecycle::AppLifecycle,
    health_probe::{validate_readiness, ExpectedReadiness, ReadinessError},
    runtime_readiness::{self, ReadinessStatus, ReadinessState}};
use std::sync::{Arc, atomic::Ordering};
use tower::ServiceExt;

struct CountHealth(std::sync::atomic::AtomicUsize);
#[async_trait::async_trait]
impl crate::health_source::HealthSource for CountHealth {
    fn id(&self) -> &str { "fixture" }
    async fn view(&self) -> crate::health_models::HealthView {
        self.0.fetch_add(1, Ordering::SeqCst);
        crate::health_models::HealthView::unconfigured()
    }
    async fn refresh(&self) -> crate::health_models::HealthView { self.view().await }
}

fn fixture() -> ReadinessStatus {
    let lifecycle = AppLifecycle::new("0.6.0", std::path::PathBuf::new());
    lifecycle.tray_initialized.store(true, Ordering::Release);
    lifecycle.persistence_initialized.store(true, Ordering::Release);
    lifecycle.remote_listening.store(true, Ordering::Release);
    lifecycle.setup_finished();
    let mut status = ReadinessStatus::snapshot(&lifecycle);
    status.supervisor_run_id = Some("supervisor-fixture".into());
    status.compute_state();
    status
}
fn expected() -> ExpectedReadiness<'static> {
    ExpectedReadiness { version: "0.6.0", pid: None, run_id: None,
        supervisor_run_id: Some("supervisor-fixture") }
}
fn validate(status: &ReadinessStatus) -> Result<ReadinessStatus, ReadinessError> {
    validate_readiness(&serde_json::to_string(status).unwrap(), &expected())
}

#[test]
fn each_missing_prerequisite_prevents_readiness() {
    for prerequisite in ["tray", "persistence", "setup", "quit"] {
        let mut status = fixture();
        match prerequisite {
            "tray" => status.tray_initialized = false,
            "persistence" => status.persistence_initialized = false,
            "setup" => status.setup_complete = false,
            _ => status.quit_requested = true,
        }
        // Validator must reject even before the producer recomputes its stale boolean.
        assert_eq!(validate(&status).unwrap_err(), ReadinessError::NotReady);
        status.compute_state();
        assert!(!status.ready, "{prerequisite}");
        assert_eq!(status.state, ReadinessState::Initializing);
    }
}

#[test]
fn sensor_absence_is_ready_degraded_and_not_restart_eligible() {
    let mut status = fixture();
    assert!(status.ready);
    assert_eq!(status.state, ReadinessState::Degraded);
    assert_eq!(status.reason.as_deref(), Some("sensor_missing"));
    assert!(validate(&status).is_ok());
    status.sensor_present = true;
    status.compute_state();
    assert_eq!(status.state, ReadinessState::Ready);
    assert!(validate(&status).is_ok());
}

#[test]
fn failure_is_sticky_even_with_all_prerequisites() {
    let mut status = fixture();
    status.state = ReadinessState::Failed;
    status.compute_state();
    assert_eq!(status.state, ReadinessState::Failed);
    assert!(!status.ready);
    assert_eq!(validate(&status).unwrap_err(), ReadinessError::NotReady);
}

#[test]
fn validator_rejects_arbitrary_json_identity_mismatch_and_forged_flags() {
    for body in ["{}", "null", "[]", r#"{"ready":true}"#] {
        assert_eq!(validate_readiness(body, &expected()).unwrap_err(), ReadinessError::InvalidSchema);
    }
    let mut status = fixture();
    status.app_id = "another-app".into();
    assert_eq!(validate(&status).unwrap_err(), ReadinessError::WrongApp);
    status = fixture(); status.version = "9.0.0".into();
    assert_eq!(validate(&status).unwrap_err(), ReadinessError::WrongVersion);
    status = fixture(); status.run_id = "not-an-id".into();
    assert_eq!(validate(&status).unwrap_err(), ReadinessError::InvalidSchema);
    status = fixture(); status.supervisor_run_id = None;
    assert_eq!(validate(&status).unwrap_err(), ReadinessError::WrongRun);
    status = fixture(); status.remote_listening = false;
    assert!(validate(&status).is_ok());
    status = fixture(); status.ready = false;
    assert_eq!(validate(&status).unwrap_err(), ReadinessError::NotReady);
    status = fixture();
    let body = serde_json::to_string(&status).unwrap();
    let mut identity = expected();
    identity.pid = Some(status.pid + 1);
    assert_eq!(validate_readiness(&body, &identity).unwrap_err(), ReadinessError::WrongRun);
    identity.pid = Some(status.pid);
    identity.run_id = Some("different-run");
    assert_eq!(validate_readiness(&body, &identity).unwrap_err(), ReadinessError::WrongRun);
    identity.run_id = Some(&status.run_id);
    assert!(validate_readiness(&body, &identity).is_ok());
}

#[tokio::test]
async fn endpoint_is_flat_bounded_and_has_no_aggregate_dependencies() {
    let router = runtime_readiness::routes(Arc::new(fixture));
    let response = tokio::time::timeout(std::time::Duration::from_millis(500),
        router.oneshot(axum::http::Request::builder().uri(runtime_readiness::PATH)
            .body(axum::body::Body::empty()).unwrap())).await.unwrap().unwrap();
    assert_eq!(response.status(), 200);
    let body = axum::body::to_bytes(response.into_body(), 4096).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["pid"].is_number());
    assert!(json["run_id"].is_string());
    assert_eq!(json["state"], "degraded");
    assert_eq!(json["ready"], true);
    assert!(json.as_object().unwrap().values().all(|v| !v.is_object() && !v.is_array()));
    assert!(validate_readiness(std::str::from_utf8(&body).unwrap(), &expected()).is_ok());
}

#[tokio::test]
async fn real_router_keeps_api_shape_and_readiness_never_reads_health() {
    use std::sync::Mutex;
    let session = Arc::new(Mutex::new(crate::session::SessionManager::new()));
    let mut state = crate::remote_routes_health_tests::remote_state(
        session.clone(),
        Arc::new(Mutex::new(crate::communication_policy::CommunicationPolicy::new(
            Default::default(), Default::default()))),
        Arc::new(Mutex::new(crate::db::TodaySummary::default())),
    ).await;
    let health = Arc::new(CountHealth(std::sync::atomic::AtomicUsize::new(0)));
    state.health = Arc::new(crate::health_source::HealthAggregator::new(vec![health.clone()]));
    let baseline = crate::remote_server::build_router(state.clone());
    let local = runtime_readiness::routes(Arc::new(fixture));
    let combined = crate::remote_server::build_router(state);
    let held = session.lock().unwrap();
    let response = local.oneshot(axum::http::Request::builder()
        .uri(runtime_readiness::PATH).body(axum::body::Body::empty()).unwrap()).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(health.0.load(Ordering::SeqCst), 0);
    drop(held);
    let mut results = Vec::new();
    for router in [baseline, combined] {
        let response = router.oneshot(axum::http::Request::builder().uri("/display/api")
            .body(axum::body::Body::empty()).unwrap()).await.unwrap();
        assert_eq!(response.status(), 200);
        let body = axum::body::to_bytes(response.into_body(), 64 * 1024).await.unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(value.get("readiness").is_none());
        results.push(value.as_object().unwrap().keys().cloned().collect::<Vec<_>>());
    }
    assert_eq!(results[0], results[1]);
    assert_eq!(health.0.load(Ordering::SeqCst), 2); // spy is actually on the aggregate path
}
