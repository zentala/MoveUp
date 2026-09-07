//! Wiremock-backed tests for [`crate::notify_webhook`].
//!
//! Covers the four paths every outbound path owes: happy (the receiver takes
//! the JSON), nil (nothing configured), empty (a blank URL configured), and
//! error (5xx, timeout, and a 4xx that must NOT be retried).

#![cfg(test)]

use std::sync::Mutex;
use std::time::Duration;

use crate::notify_webhook::{
    Notification, WebhookError, WebhookNotifier, PRIORITY_ALERT, WEBHOOK_URL_ENV,
};
use wiremock::matchers::{header, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// `std::env` is process-global; the env-reading tests take this first.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn sample() -> Notification {
    Notification::alert("MoveUp", "40 minutes sitting — stand up")
}

/// A notifier with a short timeout so the timeout path stays fast.
fn notifier_for(server: &MockServer) -> WebhookNotifier {
    WebhookNotifier::at_url(server.uri()).with_timeout(Duration::from_millis(150))
}

// ─── happy ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn posts_json_body_with_all_four_fields() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(header("content-type", "application/json"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let result = notifier_for(&server).deliver(&sample()).await;
    assert_eq!(result, Ok(()));

    let requests = server.received_requests().await.expect("recorded requests");
    assert_eq!(requests.len(), 1, "one attempt on the happy path");

    let body: serde_json::Value =
        serde_json::from_slice(&requests[0].body).expect("body is JSON");
    assert_eq!(body["title"], "MoveUp");
    assert_eq!(body["message"], "40 minutes sitting — stand up");
    assert_eq!(body["priority"], PRIORITY_ALERT);
    assert_eq!(body["tags"], serde_json::json!(["chair"]));
}

#[tokio::test]
async fn treats_a_redirect_as_delivered() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(302))
        .mount(&server)
        .await;

    assert_eq!(notifier_for(&server).deliver(&sample()).await, Ok(()));
}

// ─── nil / empty configuration ───────────────────────────────────────────────

#[test]
fn disabled_toggle_yields_no_notifier_even_with_a_url() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    assert!(WebhookNotifier::from_env_or_config(false, Some("https://ntfy.sh/desk")).is_none());
}

#[test]
fn nil_and_empty_urls_yield_no_notifier() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::remove_var(WEBHOOK_URL_ENV);

    assert!(
        WebhookNotifier::from_env_or_config(true, None).is_none(),
        "nil: nothing configured anywhere"
    );
    assert!(
        WebhookNotifier::from_env_or_config(true, Some("")).is_none(),
        "empty: a blank URL is not a destination"
    );
    assert!(
        WebhookNotifier::from_env_or_config(true, Some("   ")).is_none(),
        "empty: whitespace is not a destination"
    );
}

#[test]
fn falls_back_to_the_environment_when_config_has_no_url() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::set_var(WEBHOOK_URL_ENV, "https://ntfy.sh/from-env");

    let from_env = WebhookNotifier::from_env_or_config(true, None);
    let from_config = WebhookNotifier::from_env_or_config(true, Some("https://ntfy.sh/from-config"));

    std::env::remove_var(WEBHOOK_URL_ENV);

    assert!(from_env.is_some(), "env supplies the URL when config is empty");
    assert!(from_config.is_some(), "config URL wins when both are set");
}

// ─── error ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn retries_once_after_a_server_error_and_succeeds() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(503))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    assert_eq!(notifier_for(&server).deliver(&sample()).await, Ok(()));
    assert_eq!(
        server.received_requests().await.expect("recorded").len(),
        2,
        "one failed attempt plus one retry"
    );
}

#[tokio::test]
async fn gives_up_after_a_single_retry() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    assert_eq!(
        notifier_for(&server).deliver(&sample()).await,
        Err(WebhookError::Server(500))
    );
    assert_eq!(
        server.received_requests().await.expect("recorded").len(),
        2,
        "exactly two attempts — the retry is not a loop"
    );
}

#[tokio::test]
async fn retries_once_after_a_timeout() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(600)))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    assert_eq!(notifier_for(&server).deliver(&sample()).await, Ok(()));
    assert_eq!(
        server.received_requests().await.expect("recorded").len(),
        2,
        "the timed-out attempt still reached the server, then the retry did"
    );
}

#[tokio::test]
async fn does_not_retry_a_rejected_request() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&server)
        .await;

    assert_eq!(
        notifier_for(&server).deliver(&sample()).await,
        Err(WebhookError::Rejected(403))
    );
    assert_eq!(
        server.received_requests().await.expect("recorded").len(),
        1,
        "a 4xx is a misconfiguration; retrying it only doubles the noise"
    );
}
