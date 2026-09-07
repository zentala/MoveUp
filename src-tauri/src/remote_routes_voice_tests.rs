//! Tests for the dictation inlet (E021-T06).
//!
//! Like the health inlet's tests, these drive the **real seam**: a request
//! goes through the real router, the real auth guard, the real parser, the
//! real [`CommunicationPolicy`] and a real temp-file SQLite database, and the
//! assertions are made on what came out the far end — the stored row, the
//! policy's snooze state, the broadcast message. Nothing between the route and
//! those is stubbed. The only stubbed thing is the OpenRouter endpoint, which
//! is outside the process and therefore a legitimate boundary.

#![cfg(test)]

use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use rusqlite::Connection;
use tower::ServiceExt;

use crate::communication_policy::CommunicationPolicy;
use crate::db::TodaySummary;
use crate::db_voice_notes::{date_local_of, list_voice_notes};
use crate::event_logger::EventLogger;
use crate::health_source::{HealthAggregator, HealthState};
use crate::remote_routes_voice::{
    voice_notification, VoiceAck, VoicePostBody, MAX_BODY_BYTES, MAX_TRANSCRIPT_CHARS,
};
use crate::remote_server::{build_router, RemoteState, VoiceState};
use crate::session::SessionManager;

const TOKEN: &str = "test-token";

/// A fixed capture instant (noon local on 2026-09-06), so the day bucket is
/// unambiguous whatever timezone the test machine runs in.
fn captured_at_ms() -> i64 {
    use chrono::TimeZone;
    chrono::Local
        .with_ymd_and_hms(2026, 9, 6, 12, 0, 0)
        .single()
        .expect("noon exists")
        .timestamp_millis()
}

/// Everything a test needs to look inside the pipeline afterwards.
struct Harness {
    state: RemoteState,
    db: Arc<Mutex<Option<Connection>>>,
    logs: tempfile::TempDir,
}

impl Harness {
    async fn new() -> Self {
        Self::with(true, true).await
    }

    /// `with_db` / `with_logger` off exercises the degraded legs.
    async fn with(with_db: bool, with_logger: bool) -> Self {
        let conn = Connection::open_in_memory().expect("db opens");
        crate::db::init_schema(&conn).expect("schema applies");
        let db = Arc::new(Mutex::new(Some(conn)));

        let logs = tempfile::TempDir::new().expect("temp dir");
        let health: HealthState = Arc::new(HealthAggregator::new(vec![]));
        let health_push = crate::remote_routes_health::register(&health).await;

        let state = RemoteState {
            ws_tx: crate::ws_broadcaster::create_channel(),
            session: Arc::new(Mutex::new(SessionManager::new())),
            comm_policy: Arc::new(Mutex::new(CommunicationPolicy::new(
                Default::default(),
                Default::default(),
            ))),
            today_cache: Arc::new(Mutex::new(TodaySummary::default())),
            active_clients: Arc::new(AtomicUsize::new(0)),
            health,
            health_push,
            remote_token: Some(TOKEN.to_string()),
            voice: VoiceState {
                event_logger: with_logger
                    .then(|| Arc::new(EventLogger::new(logs.path().to_path_buf()))),
                db: with_db.then(|| db.clone()),
                ai: None,
                webhook_enabled: false,
                webhook_url: None,
            },
        };
        Self { state, db, logs }
    }

    async fn post(&self, token: Option<&str>, body: &str) -> axum::http::Response<Body> {
        let mut builder = Request::builder()
            .method("POST")
            .uri("/display/voice")
            .header("content-type", "application/json");
        if let Some(token) = token {
            builder = builder.header(crate::remote_auth::TOKEN_HEADER, token);
        }
        build_router(self.state.clone())
            .oneshot(builder.body(Body::from(body.to_string())).expect("request"))
            .await
            .expect("router responds")
    }

    async fn post_ok(&self, transcript: &str) -> VoiceAck {
        let response = self.post(Some(TOKEN), &body_json(transcript)).await;
        assert_eq!(response.status(), StatusCode::OK, "transcript {transcript:?}");
        let bytes = axum::body::to_bytes(response.into_body(), 1 << 20)
            .await
            .expect("body");
        serde_json::from_slice(&bytes).expect("ack json")
    }

    async fn status(&self, token: Option<&str>, body: &str) -> StatusCode {
        self.post(token, body).await.status()
    }

    fn stored_today(&self) -> Vec<crate::db_voice_notes::VoiceNoteRow> {
        let guard = self.db.lock().expect("db lock");
        let conn = guard.as_ref().expect("db open");
        list_voice_notes(conn, &date_local_of(captured_at_ms())).expect("list")
    }

    fn event_log(&self) -> String {
        let day = chrono::Local::now().format("%Y-%m-%d").to_string();
        std::fs::read_to_string(self.logs.path().join(day).join("events.log")).unwrap_or_default()
    }
}

fn body_json(transcript: &str) -> String {
    serde_json::json!({
        "transcript": transcript,
        "lang": "pl-PL",
        "captured_at_ms": captured_at_ms(),
    })
    .to_string()
}

// ── The happy path, end to end ───────────────────────────────────────────────

#[tokio::test]
async fn e021_t06_a_snooze_note_is_parsed_stored_and_applied_to_the_policy() {
    let h = Harness::new().await;
    assert!(
        !h.state.comm_policy.lock().unwrap().is_snoozed(),
        "the policy must not start snoozed"
    );

    let ack = h.post_ok("drzemka 5").await;
    assert_eq!(ack.intent, "snooze");
    assert_eq!(ack.snooze_minutes, Some(5));
    assert_eq!(ack.transcript, "drzemka 5");

    // Read back through the real policy, not through the parser the route used.
    assert!(
        h.state.comm_policy.lock().unwrap().is_snoozed(),
        "a snooze intent must reach CommunicationPolicy"
    );

    // Read back through the real query, not through the insert's return value.
    let rows = h.stored_today();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].transcript, "drzemka 5");
    assert_eq!(rows[0].intent, "snooze");
    assert_eq!(rows[0].lang.as_deref(), Some("pl-PL"));
    assert_eq!(ack.note_id, Some(rows[0].id));
}

#[tokio::test]
async fn e021_t06_a_plain_note_is_stored_without_touching_the_policy() {
    let h = Harness::new().await;
    let ack = h.post_ok("kupić mleko po pracy").await;

    assert_eq!(ack.intent, "note");
    assert_eq!(ack.snooze_minutes, None);
    assert!(
        !h.state.comm_policy.lock().unwrap().is_snoozed(),
        "only a snooze intent may silence escalation"
    );
    assert_eq!(h.stored_today().len(), 1);
}

#[tokio::test]
async fn e021_t06_walk_intents_are_recorded_but_change_no_state() {
    let h = Harness::new().await;

    let start = h.post_ok("idę na spacer").await;
    assert_eq!(start.intent, "walk_start");
    let end = h.post_ok("wracam ze spaceru").await;
    assert_eq!(end.intent, "walk_end");

    // E021-D3: voice never writes to the session engine.
    let session = h.state.session.lock().unwrap().snapshot();
    assert_eq!(session.position_changes, 0);
    assert_eq!(session.standing_seconds, 0);
    assert!(!h.state.comm_policy.lock().unwrap().is_snoozed());

    let intents: Vec<String> = h.stored_today().iter().map(|r| r.intent.clone()).collect();
    assert_eq!(intents, vec!["walk_start", "walk_end"]);
}

#[tokio::test]
async fn e021_t06_the_event_log_line_is_written_before_the_note_is_stored() {
    let h = Harness::new().await;
    h.post_ok("drzemka 5").await;

    let log = h.event_log();
    assert!(
        log.contains("VOICE snooze drzemka 5"),
        "expected a VOICE line, got: {log:?}"
    );
}

#[tokio::test]
async fn e021_t06_the_ack_is_broadcast_to_websocket_clients() {
    let h = Harness::new().await;
    let mut rx = h.state.ws_tx.subscribe();

    h.post_ok("drzemka 5").await;

    let msg = rx.try_recv().expect("an ack must be published");
    let parsed: serde_json::Value = serde_json::from_str(&msg).expect("json");
    assert_eq!(parsed["event"], "desk:voice-ack");
    assert_eq!(parsed["payload"]["intent"], "snooze");
    assert_eq!(parsed["payload"]["transcript"], "drzemka 5");
    assert_eq!(parsed["payload"]["reply"], serde_json::Value::Null);
}

// ── Auth and body limits ─────────────────────────────────────────────────────

#[tokio::test]
async fn e021_t06_an_unauthenticated_post_is_rejected_and_stores_nothing() {
    let h = Harness::new().await;

    assert_eq!(
        h.status(None, &body_json("drzemka 5")).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        h.status(Some("wrong"), &body_json("drzemka 5")).await,
        StatusCode::UNAUTHORIZED
    );
    assert!(h.stored_today().is_empty());
    assert!(!h.state.comm_policy.lock().unwrap().is_snoozed());
}

#[tokio::test]
async fn e021_t06_the_inlet_is_closed_when_no_token_is_configured() {
    let mut h = Harness::new().await;
    h.state.remote_token = None;
    assert_eq!(
        h.status(Some(TOKEN), &body_json("drzemka 5")).await,
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert!(h.stored_today().is_empty());
}

#[tokio::test]
async fn e021_t06_an_oversized_body_is_refused() {
    let h = Harness::new().await;
    let padded = body_json(&"x".repeat(MAX_BODY_BYTES));
    assert!(padded.len() > MAX_BODY_BYTES);

    assert_eq!(
        h.status(Some(TOKEN), &padded).await,
        StatusCode::PAYLOAD_TOO_LARGE
    );
    assert!(h.stored_today().is_empty());
}

#[tokio::test]
async fn e021_t06_malformed_and_unusable_bodies_are_distinguished() {
    let h = Harness::new().await;

    // Not JSON, and JSON missing a required field: malformed.
    assert_eq!(
        h.status(Some(TOKEN), "not json").await,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        h.status(Some(TOKEN), r#"{"transcript":"drzemka 5"}"#).await,
        StatusCode::BAD_REQUEST
    );

    // Well-formed JSON, unusable values: unprocessable.
    assert_eq!(
        h.status(Some(TOKEN), &body_json("   ")).await,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        h.status(
            Some(TOKEN),
            &serde_json::json!({"transcript":"ok","captured_at_ms":0}).to_string()
        )
        .await,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert!(h.stored_today().is_empty());
}

#[test]
fn e021_t06_transcript_length_boundary_is_inclusive() {
    let at_limit = "a".repeat(MAX_TRANSCRIPT_CHARS);
    let over = "a".repeat(MAX_TRANSCRIPT_CHARS + 1);
    let body = |t: String| VoicePostBody {
        transcript: t,
        lang: None,
        captured_at_ms: 1,
    };
    assert!(body(at_limit).validate().is_some());
    assert!(body(over).validate().is_none());
}

#[test]
fn e021_t06_a_blank_lang_reads_as_unknown_not_as_empty_string() {
    let valid = VoicePostBody {
        transcript: "  drzemka 5  ".into(),
        lang: Some("   ".into()),
        captured_at_ms: 1,
    }
    .validate()
    .expect("valid");
    assert_eq!(valid.lang, None);
    assert_eq!(valid.transcript, "drzemka 5", "transcript is stored trimmed");
}

// ── Degraded legs ────────────────────────────────────────────────────────────

#[tokio::test]
async fn e021_t06_without_a_database_the_note_is_still_parsed_and_acknowledged() {
    let h = Harness::with(false, true).await;
    let ack = h.post_ok("drzemka 5").await;

    assert_eq!(ack.intent, "snooze");
    assert_eq!(
        ack.note_id, None,
        "an unstored note must say so rather than report a made-up id"
    );
    assert!(
        h.state.comm_policy.lock().unwrap().is_snoozed(),
        "the side effect must not depend on the database"
    );
    assert!(h.event_log().contains("VOICE snooze"));
}

#[tokio::test]
async fn e021_t06_without_an_event_logger_the_note_is_still_stored() {
    let h = Harness::with(true, false).await;
    let ack = h.post_ok("drzemka 5").await;
    assert!(ack.note_id.is_some());
    assert_eq!(h.stored_today().len(), 1);
}

#[tokio::test]
async fn e021_t06_without_an_ai_client_the_reply_is_absent_not_empty() {
    let h = Harness::new().await;
    let ack = h.post_ok("drzemka 5").await;
    assert_eq!(
        ack.reply, None,
        "no key configured must read as no reply, never as an empty reply"
    );
    assert_eq!(h.stored_today()[0].reply, None);
}

// ── Webhook body ─────────────────────────────────────────────────────────────

#[test]
fn e021_t06_the_webhook_body_prefers_the_reply_and_falls_back_to_the_transcript() {
    let ack = |reply: Option<&str>| VoiceAck {
        transcript: "drzemka 5".into(),
        intent: "snooze".into(),
        snooze_minutes: Some(5),
        reply: reply.map(str::to_string),
        note_id: Some(1),
    };

    let with_reply = voice_notification(&ack(Some("Dobrze, 5 minut przerwy.")));
    assert_eq!(with_reply.message, "Dobrze, 5 minut przerwy.");
    assert!(with_reply.title.contains("snooze"));

    let without = voice_notification(&ack(None));
    assert_eq!(
        without.message, "drzemka 5",
        "a push with no AI reply must still say what was heard"
    );
}
