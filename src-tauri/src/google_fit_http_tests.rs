//! Wiremock-backed HTTP integration tests for `GoogleFitClient`.
//!
//! Each test spins up an isolated wiremock server, points the client at it
//! via `Endpoints::at_base`, and asserts behaviour for one slice of the
//! request/response surface.

#![cfg(test)]

use crate::google_fit::{
    Credentials, Endpoints, ErrorKind, GoogleFitClient,
};
use wiremock::matchers::{header, method, path, query_param};
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
