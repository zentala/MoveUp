//! Wiremock-backed HTTP integration tests for `GoogleFitClient`.
//!
//! Each test spins up an isolated wiremock server, points the client at it
//! via `Endpoints::at_base`, and asserts behaviour for one slice of the
//! request/response surface.

#![cfg(test)]

use crate::google_fit::{
    Credentials, Endpoints, ErrorKind, GoogleFitClient,
};
use wiremock::matchers::{body_string_contains, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn fake_creds() -> Credentials {
    Credentials {
        client_id: "test-client".into(),
        client_secret: "test-secret".into(),
        refresh_token: "test-refresh".into(),
        steps_source_override: None,
    }
}

async fn client_for(server: &MockServer) -> GoogleFitClient {
    GoogleFitClient::with_endpoints(fake_creds(), Endpoints::at_base(&server.uri()))
}

/// Mount a successful /token mock that returns a fake access token.
/// Reduces 7-line setup repeated across most HTTP tests.
async fn mount_token_ok(server: &MockServer, access_token: &str) {
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": access_token,
            "expires_in": 3600,
            "token_type": "Bearer"
        })))
        .mount(server)
        .await;
}

#[tokio::test]
async fn fetch_steps_happy_path_sums_aggregate() {
    let server = MockServer::start().await;

    mount_token_ok(&server, "fake-access").await;

    Mock::given(method("POST"))
        .and(path("/aggregate"))
        .and(header("authorization", "Bearer fake-access"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "bucket": [{
                "dataset": [{
                    "point": [
                        {"value": [{"intVal": 1500}]},
                        {"value": [{"intVal": 250}]}
                    ]
                }]
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let steps = client
        .fetch_steps(0, 86_400_000, "derived:test:source")
        .await
        .expect("fetch_steps should succeed");
    assert_eq!(steps, 1750);
}

#[tokio::test]
async fn token_refresh_invalid_grant_classified_as_auth_revoked() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": "invalid_grant",
            "error_description": "Token has been expired or revoked."
        })))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let err = client
        .fetch_steps(0, 86_400_000, "derived:test:source")
        .await
        .expect_err("should fail");
    assert_eq!(err.kind, ErrorKind::AuthRevoked);
}

#[tokio::test]
async fn aggregate_401_classified_as_auth_revoked() {
    let server = MockServer::start().await;

    mount_token_ok(&server, "fake-access").await;

    Mock::given(method("POST"))
        .and(path("/aggregate"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let err = client
        .fetch_steps(0, 86_400_000, "derived:test:source")
        .await
        .expect_err("should fail");
    assert_eq!(err.kind, ErrorKind::AuthRevoked);
}

#[tokio::test]
async fn aggregate_5xx_classified_as_transient() {
    let server = MockServer::start().await;

    mount_token_ok(&server, "fake-access").await;

    Mock::given(method("POST"))
        .and(path("/aggregate"))
        .respond_with(ResponseTemplate::new(503).set_body_string("upstream"))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let err = client
        .fetch_steps(0, 86_400_000, "derived:test:source")
        .await
        .expect_err("should fail");
    assert_eq!(err.kind, ErrorKind::Transient);
    assert!(err.message.contains("503"));
}

#[tokio::test]
async fn list_step_sources_returns_data_stream_ids() {
    let server = MockServer::start().await;

    mount_token_ok(&server, "fake-access").await;

    Mock::given(method("GET"))
        .and(path("/dataSources"))
        .and(query_param("dataTypeName", "com.google.step_count.delta"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "dataSource": [
                {"dataStreamId": "derived:com.google.step_count.delta:com.google.android.gms:estimated_steps"},
                {"dataStreamId": "raw:com.google.step_count.delta:samsung:SM-N9005:abc"}
            ]
        })))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let sources = client.list_step_sources().await.expect("list_step_sources");
    assert_eq!(sources.len(), 2);
    assert!(sources[0].contains("estimated_steps"));
}

#[tokio::test]
async fn aggregate_malformed_json_classified_as_transient() {
    let server = MockServer::start().await;

    mount_token_ok(&server, "fake-access").await;

    Mock::given(method("POST"))
        .and(path("/aggregate"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let err = client
        .fetch_steps(0, 86_400_000, "derived:test:source")
        .await
        .expect_err("should fail to parse");
    assert_eq!(err.kind, ErrorKind::Transient);
}

#[tokio::test]
async fn fetch_heart_rate_averages_fp_vals() {
    let server = MockServer::start().await;

    mount_token_ok(&server, "fake-access").await;

    Mock::given(method("POST"))
        .and(path("/aggregate"))
        .and(body_string_contains("com.google.heart_rate.bpm"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "bucket": [{
                "dataset": [{
                    "point": [
                        {"value": [{"fpVal": 71.0}]},
                        {"value": [{"fpVal": 73.0}]}
                    ]
                }]
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let bpm = client
        .fetch_heart_rate(0, 86_400_000, "derived:hr:source")
        .await
        .expect("fetch_heart_rate should succeed");
    assert_eq!(bpm, Some(72));
}

#[tokio::test]
async fn fetch_heart_rate_returns_none_when_no_points() {
    let server = MockServer::start().await;

    mount_token_ok(&server, "fake-access").await;

    // An account with no heart-rate data gets an empty bucket list, not an
    // error — and must not be reported as 0 bpm.
    Mock::given(method("POST"))
        .and(path("/aggregate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"bucket": []})))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let bpm = client
        .fetch_heart_rate(0, 86_400_000, "derived:hr:source")
        .await
        .expect("empty data is not an error");
    assert_eq!(bpm, None);
}

#[tokio::test]
async fn list_heart_rate_sources_queries_the_heart_rate_data_type() {
    let server = MockServer::start().await;

    mount_token_ok(&server, "fake-access").await;

    Mock::given(method("GET"))
        .and(path("/dataSources"))
        .and(query_param("dataTypeName", "com.google.heart_rate.bpm"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "dataSource": [
                {"dataStreamId": "derived:com.google.heart_rate.bpm:com.google.android.gms:merge_heart_rate_bpm"}
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let sources = client
        .list_heart_rate_sources()
        .await
        .expect("list_heart_rate_sources");
    assert_eq!(sources.len(), 1);
    assert!(sources[0].contains("merge_heart_rate_bpm"));
}

#[tokio::test]
async fn heart_rate_401_classified_as_auth_revoked() {
    let server = MockServer::start().await;

    mount_token_ok(&server, "fake-access").await;

    Mock::given(method("POST"))
        .and(path("/aggregate"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let client = client_for(&server).await;
    let err = client
        .fetch_heart_rate(0, 86_400_000, "derived:hr:source")
        .await
        .expect_err("should fail");
    assert_eq!(err.kind, ErrorKind::AuthRevoked);
}
