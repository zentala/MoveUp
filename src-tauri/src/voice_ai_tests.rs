//! Wiremock-backed tests for [`crate::voice_ai`].
//!
//! Every test points the client at an isolated wiremock server via
//! `VoiceAi::at_endpoint`, so no test ever reaches OpenRouter.

#![cfg(test)]

use crate::session::{DeskState, SessionStateDto};
use crate::voice_ai::{VoiceAi, VoiceAiError, API_KEY_ENV, DEFAULT_MODEL, SYSTEM_PROMPT};
use std::sync::Mutex;
use std::time::Duration;
use wiremock::matchers::{body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Serialises the tests that mutate `OPENROUTER_API_KEY` — the process
/// environment is shared by every test thread.
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn session() -> SessionStateDto {
    SessionStateDto {
        state: DeskState::Sitting,
        sitting_seconds: 1800,
        standing_seconds: 600,
        break_seconds: 600,
        session_limit_secs: 2400,
        stand_limit_secs: 0,
        desk_height_cm: 72.0,
        position_changes: 3,
        limit_used_secs: 1200,
        daily_score: 62.0,
        standing_session_secs: 0,
        secs_since_last_break: 900,
        continuous_computer_secs: 2400,
        longest_computer_session_secs: 2400,
        sitting_seconds_total: 1800,
        idle_secs: 0,
        away_bout_secs: 0,
        max_continuous_computer_secs: 3600,
    }
}

fn client_for(server: &MockServer, model: &str) -> VoiceAi {
    VoiceAi::at_endpoint(
        "test-key",
        model,
        format!("{}/chat/completions", server.uri()),
    )
}

/// Mounts a chat-completions mock that answers with `content`.
async fn mount_reply(server: &MockServer, content: &str) {
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{"message": {"role": "assistant", "content": content}}]
        })))
        .expect(1)
        .mount(server)
        .await;
}

// ─── configuration ───────────────────────────────────────────────────────────

#[test]
fn from_env_is_none_without_a_key_and_treats_blank_as_absent() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    std::env::remove_var(API_KEY_ENV);
    assert!(VoiceAi::from_env(None).is_none(), "no key: not configured");

    std::env::set_var(API_KEY_ENV, "   ");
    let blank = VoiceAi::from_env(None);
    std::env::remove_var(API_KEY_ENV);
    assert!(blank.is_none(), "blank key: not configured");
}

#[test]
fn from_env_prefers_the_configured_model_over_the_cheap_default() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::set_var(API_KEY_ENV, "sk-test");

    let defaulted = VoiceAi::from_env(None).expect("a key means configured");
    let blank_model = VoiceAi::from_env(Some("  ")).expect("a key means configured");
    let chosen = VoiceAi::from_env(Some("openai/gpt-4o-mini")).expect("a key means configured");

    std::env::remove_var(API_KEY_ENV);

    assert_eq!(defaulted.model(), DEFAULT_MODEL);
    assert_eq!(blank_model.model(), DEFAULT_MODEL, "blank is not a choice");
    assert_eq!(chosen.model(), "openai/gpt-4o-mini");
}

// ─── replies ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn happy_path_returns_the_model_reply() {
    let server = MockServer::start().await;
    mount_reply(&server, "  Wstan na pieć minut i rozciągnij plecy.  ").await;

    let reply = client_for(&server, DEFAULT_MODEL)
        .reply("czuję sztywność w plecach", &session())
        .await
        .expect("reply should succeed");

    assert_eq!(
        reply.as_deref(),
        Some("Wstan na pieć minut i rozciągnij plecy.")
    );
}

#[tokio::test]
async fn request_carries_key_model_system_prompt_and_session_numbers() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("authorization", "Bearer test-key"))
        .and(body_string_contains("openai/gpt-4o-mini"))
        // A distinctive clause of the fixed system prompt.
        .and(body_string_contains("at most 60 words"))
        // Sitting limit used, in whole minutes: 1200s of 2400s.
        .and(body_string_contains("20 of 40 minutes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{"message": {"content": "ok"}}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let reply = client_for(&server, "openai/gpt-4o-mini")
        .reply("jak mi idzie", &session())
        .await
        .expect("reply should succeed");

    assert_eq!(reply.as_deref(), Some("ok"));
    assert!(SYSTEM_PROMPT.contains("at most 60 words"));
}

#[tokio::test]
async fn blank_transcript_returns_none_without_calling_the_model() {
    let server = MockServer::start().await;
    // Any request at all is a failure: the mock expects zero hits and the
    // server asserts that on drop.
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let reply = client_for(&server, DEFAULT_MODEL)
        .reply("   ", &session())
        .await
        .expect("a blank note is not an error");

    assert_eq!(reply, None);
}

#[tokio::test]
async fn empty_content_returns_none_not_an_empty_string() {
    let server = MockServer::start().await;
    mount_reply(&server, "   ").await;

    let reply = client_for(&server, DEFAULT_MODEL)
        .reply("cokolwiek", &session())
        .await
        .expect("an empty answer is not an error");

    assert_eq!(reply, None);
}

#[tokio::test]
async fn missing_choices_returns_none() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"choices": []})),
        )
        .mount(&server)
        .await;

    let reply = client_for(&server, DEFAULT_MODEL)
        .reply("cokolwiek", &session())
        .await
        .expect("no choices is not an error");

    assert_eq!(reply, None);
}

#[tokio::test]
async fn invalid_key_is_rejected_not_retried() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(401).set_body_string("no auth"))
        .expect(1)
        .mount(&server)
        .await;

    let err = client_for(&server, DEFAULT_MODEL)
        .reply("hej", &session())
        .await
        .expect_err("401 should fail");

    assert_eq!(err, VoiceAiError::Rejected(401));
}

#[tokio::test]
async fn upstream_5xx_is_a_server_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(503).set_body_string("upstream"))
        .mount(&server)
        .await;

    let err = client_for(&server, DEFAULT_MODEL)
        .reply("hej", &session())
        .await
        .expect_err("503 should fail");

    assert_eq!(err, VoiceAiError::Server(503));
}

#[tokio::test]
async fn non_json_200_is_malformed() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;

    let err = client_for(&server, DEFAULT_MODEL)
        .reply("hej", &session())
        .await
        .expect_err("a 200 that is not a chat completion should fail");

    assert_eq!(err, VoiceAiError::Malformed);
}

#[tokio::test]
async fn a_hanging_endpoint_is_a_transport_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .mount(&server)
        .await;

    let err = client_for(&server, DEFAULT_MODEL)
        .with_timeout(Duration::from_millis(50))
        .reply("hej", &session())
        .await
        .expect_err("a timeout should fail");

    assert_eq!(err, VoiceAiError::Transport);
}
