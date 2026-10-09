use std::sync::{Arc, Mutex, atomic::Ordering};
use crate::{app_lifecycle::AppLifecycle, readiness_server,
    runtime_readiness::{self, ReadinessStatus}, remote_server};
use tower::ServiceExt;

fn lifecycle() -> Arc<AppLifecycle> {
    let lifecycle = Arc::new(AppLifecycle::new("0.6.0", std::path::PathBuf::new()));
    lifecycle.tray_initialized.store(true, Ordering::Release);
    lifecycle.persistence_initialized.store(true, Ordering::Release);
    lifecycle.setup_finished();
    lifecycle
}

fn reader(lifecycle: Arc<AppLifecycle>) -> runtime_readiness::ReadinessReader {
    Arc::new(move || {
        let mut status = ReadinessStatus::snapshot(&lifecycle);
        status.compute_state();
        status
    })
}

async fn remote_state() -> remote_server::RemoteState {
    crate::remote_routes_health_tests::remote_state(
        Arc::new(Mutex::new(crate::session::SessionManager::new())),
        Arc::new(Mutex::new(crate::communication_policy::CommunicationPolicy::new(
            Default::default(), Default::default()))),
        Arc::new(Mutex::new(crate::db::TodaySummary::default())),
    ).await
}

#[test]
fn port_is_explicit_and_invalid_values_never_fall_back() {
    assert_eq!(readiness_server::parse_port(None).unwrap(), 3391);
    assert_eq!(readiness_server::parse_port(Some("54321")).unwrap(), 54321);
    for value in ["", "0", "65536", "-1", "+3391", " 3391", "port"] {
        assert!(readiness_server::parse_port(Some(value)).is_err(), "{value}");
    }
}

#[tokio::test]
async fn lan_disabled_still_serves_loopback_readiness_without_sensor() {
    assert!(!remote_server::lan_enabled_in(Some(&serde_json::json!({"remote_lan_enabled": false}))));
    let lifecycle = lifecycle();
    let listener = readiness_server::bind(0, &lifecycle).await.unwrap();
    let address = listener.local_addr().unwrap();
    assert_eq!(address.ip(), std::net::Ipv4Addr::LOCALHOST);
    let router = runtime_readiness::routes(reader(lifecycle));
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let response = reqwest::Client::builder().no_proxy().build().unwrap()
        .get(format!("http://{address}{}", runtime_readiness::PATH))
        .timeout(std::time::Duration::from_secs(2)).send().await.unwrap();
    assert_eq!(response.status(), 200);
    let status: ReadinessStatus = response.json().await.unwrap();
    assert!(status.ready);
    assert!(!status.remote_listening);
    assert!(!status.sensor_present);
    assert_eq!(status.reason.as_deref(), Some("sensor_missing"));
    server.abort();
    let _ = server.await;
}

#[tokio::test]
async fn busy_lan_port_does_not_invalidate_essential_readiness() {
    let occupied = tokio::net::TcpListener::bind((std::net::Ipv4Addr::UNSPECIFIED, 0)).await.unwrap();
    let lifecycle = lifecycle();
    remote_server::start(remote_state().await, occupied.local_addr().unwrap().port(),
        lifecycle.clone()).await;
    assert!(!lifecycle.remote_listening.load(Ordering::Acquire));
    let local = readiness_server::bind(0, &lifecycle).await.unwrap();
    assert!(reader(lifecycle)().ready);
    drop(local);
}

#[tokio::test]
async fn busy_readiness_port_fails_without_fallback_and_prevents_readiness() {
    let occupied = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).await.unwrap();
    let lifecycle = lifecycle();
    let result = readiness_server::bind(occupied.local_addr().unwrap().port(), &lifecycle).await;
    assert!(result.is_err());
    assert!(lifecycle.failed.load(Ordering::Acquire));
    assert!(!reader(lifecycle)().ready);
}

#[tokio::test]
async fn lan_router_has_no_readiness_metadata_and_keeps_display_routes() {
    let router = remote_server::build_router(remote_state().await);
    for path in [runtime_readiness::PATH, "/display/api", "/display"] {
        let response = router.clone().oneshot(axum::http::Request::builder().uri(path)
            .body(axum::body::Body::empty()).unwrap()).await.unwrap();
        assert_eq!(response.status(), 200); // development fallback remains HTML
        let content_type = response.headers()["content-type"].to_str().unwrap().to_string();
        let body = axum::body::to_bytes(response.into_body(), 64 * 1024).await.unwrap();
        if path == "/display/api" {
            let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
            assert!(value.get("app_id").is_none());
        } else {
            assert!(content_type.starts_with("text/html"));
            assert!(!std::str::from_utf8(&body).unwrap().contains("supervisor_run_id"));
        }
    }
}
