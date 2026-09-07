//! relay_auth_tests.rs — credentials and the relay REST client (E022-T06).
//!
//! The HTTP half runs against `wiremock`, so the status codes, the bearer
//! header and the error body are the real wire behaviour rather than a mock of
//! what we assume it is. The credential half runs against `keyring`'s mock
//! store, which [`KeyringStore::new`] installs itself under `cfg(test)` — no
//! test can reach the real Windows Credential Manager.

use serde_json::json;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::relay_auth::{
    effective_url, qr_payload, CredentialStore, KeyringStore, RelayApi, RELAY_DEFAULT_URL,
};

const TOKEN: &str = "mu_d_0123456789012345678901234567890123456789012";

// ─── Credential store ───────────────────────────────────────────────────────

#[test]
fn keyring_store_round_trips_a_token_and_forgets_it() {
    let store = KeyringStore::new().expect("mock credential store opens");
    assert_eq!(store.read(), None, "a fresh entry holds no token");

    store.write(TOKEN).expect("token stored");
    assert_eq!(store.read().as_deref(), Some(TOKEN));

    store.delete().expect("token deleted");
    assert_eq!(store.read(), None);
}

#[test]
fn deleting_a_token_that_was_never_stored_is_success() {
    // "No token" is the state the caller asked for; reaching it from an empty
    // store must not read as a failure and leave the relay half-disabled.
    let store = KeyringStore::new().expect("mock credential store opens");
    store.delete().expect("deleting nothing succeeds");
}

// ─── URL helpers ────────────────────────────────────────────────────────────

#[test]
fn an_empty_configured_url_falls_back_to_the_default_relay() {
    assert_eq!(effective_url(""), RELAY_DEFAULT_URL);
    assert_eq!(effective_url("   "), RELAY_DEFAULT_URL);
    assert_eq!(effective_url("https://staging.example/"), "https://staging.example");
}

#[test]
fn the_qr_payload_deep_links_the_viewer_app() {
    assert_eq!(
        qr_payload("https://relay.example/", "desk-1", "ABCD2345"),
        "https://relay.example/app/#/pair?d=desk-1&c=ABCD2345"
    );
}

// ─── REST — happy paths ─────────────────────────────────────────────────────

#[tokio::test]
async fn register_posts_the_license_and_returns_the_desk_identity() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/desks/register"))
        .and(body_json(json!({
            "license_key": "LIC-1",
            "desk_name": "mATX",
            "app_version": "9.9.9",
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "desk_id": "desk-1",
            "desk_token": TOKEN,
            "plan": "founder",
            "expires_at": "2027-01-01T00:00:00Z",
        })))
        .mount(&server)
        .await;

    let got = RelayApi::new(&server.uri())
        .register("LIC-1", "mATX", "9.9.9")
        .await
        .expect("registration accepted");

    assert_eq!(got.desk_id, "desk-1");
    assert_eq!(got.desk_token, TOKEN);
    assert_eq!(got.plan.as_deref(), Some("founder"));
}

#[tokio::test]
async fn start_pairing_returns_the_code_and_builds_the_qr_payload() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/desks/desk-1/pairings"))
        .and(header("authorization", format!("Bearer {TOKEN}")))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "code": "ABCD2345",
            "expires_at": "2026-09-07T12:05:00Z",
        })))
        .mount(&server)
        .await;

    let got = RelayApi::new(&server.uri())
        .start_pairing("desk-1", TOKEN)
        .await
        .expect("pairing code issued");

    assert_eq!(got.code, "ABCD2345");
    assert_eq!(got.desk_id, "desk-1");
    assert_eq!(
        got.qr_payload,
        format!("{}/app/#/pair?d=desk-1&c=ABCD2345", server.uri())
    );
}

#[tokio::test]
async fn revoking_a_viewer_and_deleting_the_desk_accept_an_empty_204() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/v1/desks/desk-1/viewers/v-1"))
        .and(header("authorization", format!("Bearer {TOKEN}")))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/v1/desks/desk-1"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let api = RelayApi::new(&server.uri());
    api.revoke_viewer("desk-1", "v-1", TOKEN)
        .await
        .expect("viewer revoked");
    api.delete_desk("desk-1", TOKEN)
        .await
        .expect("desk deleted");
}

// ─── REST — nil / empty / error ─────────────────────────────────────────────

#[tokio::test]
async fn list_viewers_reads_a_populated_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/desks/desk-1/viewers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{
            "viewer_id": "v-1",
            "device_name": "Pixel",
            "paired_at": "2026-09-01T10:00:00Z",
            "last_seen": null,
            "online": true,
        }])))
        .mount(&server)
        .await;

    let got = RelayApi::new(&server.uri())
        .list_viewers("desk-1", TOKEN)
        .await
        .expect("viewers listed");

    assert_eq!(got.len(), 1);
    assert_eq!(got[0].device_name, "Pixel");
    assert_eq!(got[0].last_seen, None, "never-seen must stay null, not a zero date");
    assert!(got[0].online);
}

#[tokio::test]
async fn no_paired_phones_is_an_empty_list_not_an_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/desks/desk-1/viewers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;

    let got = RelayApi::new(&server.uri())
        .list_viewers("desk-1", TOKEN)
        .await
        .expect("an empty roster is a valid answer");
    assert!(got.is_empty());
}

#[tokio::test]
async fn a_rejected_license_surfaces_the_relays_own_error_code() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/desks/register"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({
            "error": { "code": "invalid_license", "message": "unknown license key" },
        })))
        .mount(&server)
        .await;

    let err = RelayApi::new(&server.uri())
        .register("nope", "mATX", "9.9.9")
        .await
        .expect_err("a rejected license must not read as success");

    assert_eq!(err.code, "invalid_license");
    assert_eq!(err.message, "unknown license key");
}

#[tokio::test]
async fn an_error_body_that_is_not_the_schema_still_fails_loudly() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/desks/desk-1/viewers"))
        .respond_with(ResponseTemplate::new(502).set_body_string("<html>bad gateway</html>"))
        .mount(&server)
        .await;

    let err = RelayApi::new(&server.uri())
        .list_viewers("desk-1", TOKEN)
        .await
        .expect_err("502 is a failure whatever the body looks like");
    assert_eq!(err.code, "http_error");
    assert!(err.message.contains("502"), "got: {}", err.message);
}

#[tokio::test]
async fn a_success_status_with_an_unreadable_body_is_an_error_not_an_empty_list() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/desks/desk-1/viewers"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;

    let err = RelayApi::new(&server.uri())
        .list_viewers("desk-1", TOKEN)
        .await
        .expect_err("garbage must not be read as zero paired phones");
    assert_eq!(err.code, "bad_response");
}

#[tokio::test]
async fn an_unreachable_relay_is_reported_as_unreachable() {
    // Port 1 is reserved and never served — the closest thing to "the relay
    // is down" that does not depend on a port staying free.
    let err = RelayApi::new("http://127.0.0.1:1")
        .start_pairing("desk-1", TOKEN)
        .await
        .expect_err("a dead relay cannot issue a pairing code");
    assert_eq!(err.code, "unreachable");
}
