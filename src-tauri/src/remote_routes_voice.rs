//! remote_routes_voice.rs — `POST /display/voice`, the dictation inlet (E021-T06).
//!
//! The phone dictates a sentence; this route decides what it meant, keeps it,
//! and answers with an acknowledgement the phone can render immediately.
//!
//! ## The pipeline, in order, and why the order matters
//! 1. **Event log** first, so a missing or unwritable database still leaves a
//!    record that a note arrived. 2. **Database** ([`crate::db_voice_notes`]).
//! 3. **Side effect** — only [`Intent::Snooze`] has one; `Walk*` and `Note`
//!    deliberately do nothing, because voice never writes to `session_*.rs`
//!    (E021-D3 / ADR 021) and a misheard sentence must not rewrite the day's
//!    counters. 4. **AI reply**, optional and best-effort ([`crate::voice_ai`]) —
//!    a failure is dropped, the note is already stored. 5. **Broadcast +
//!    webhook**, so another display and an unattended phone both learn of it.
//!
//! Token required (same guard as the health inlet), 4 KiB body cap, transcript
//! capped at [`MAX_TRANSCRIPT_CHARS`] characters.

use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::db_voice_notes::{insert_voice_note, set_reply, NewVoiceNote};
use crate::notify_webhook::{Notification, WebhookNotifier};
use crate::remote_auth::require_token;
use crate::remote_server::RemoteState;
use crate::voice_intent::{self, Intent};
use crate::ws_broadcaster::{self, DisplayEvent};

/// Largest accepted body — four times the health inlet's cap, because a
/// transcript is free text rather than five numbers.
pub const MAX_BODY_BYTES: usize = 4096;

/// Longest transcript accepted, in characters.
pub const MAX_TRANSCRIPT_CHARS: usize = 2000;
/// Longest accepted `lang` tag — BCP-47 tags are far shorter than this.
const MAX_LANG_LEN: usize = 35;

/// ntfy priority for an acknowledgement: below an alert, above silent.
const VOICE_PRIORITY: u8 = 3;

/// Request body of `POST /display/voice`.
#[derive(Debug, Deserialize)]
pub struct VoicePostBody {
    pub transcript: String,
    #[serde(default)]
    pub lang: Option<String>,
    pub captured_at_ms: i64,
}

/// A validated request; `None` from [`VoicePostBody::validate`] means the body
/// parsed but cannot be used.
#[derive(Debug, PartialEq, Eq)]
pub struct ValidVoicePost {
    pub transcript: String,
    pub lang: Option<String>,
    pub captured_at_ms: i64,
}

impl VoicePostBody {
    /// Applies the constraints `serde` cannot express.
    pub fn validate(self) -> Option<ValidVoicePost> {
        let transcript = self.transcript.trim().to_string();
        if transcript.is_empty() || transcript.chars().count() > MAX_TRANSCRIPT_CHARS {
            return None;
        }
        if self.captured_at_ms <= 0 {
            return None;
        }
        let lang = self.lang.map(|l| l.trim().to_string());
        let lang = lang.filter(|l| !l.is_empty() && l.len() <= MAX_LANG_LEN);
        let captured_at_ms = self.captured_at_ms;
        Some(ValidVoicePost { transcript, lang, captured_at_ms })
    }
}

/// What the phone gets back, and what goes out on the WS stream. Each `Option`
/// distinguishes "did not happen" from a zero value: no snooze, no reply, and —
/// when the note could not be stored — no id, rather than a made-up one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoiceAck {
    /// The transcript as stored — trimmed, otherwise unchanged.
    pub transcript: String,
    /// [`Intent::label`] — `snooze`, `note`, `walk_start`, `walk_end`.
    pub intent: String,
    pub snooze_minutes: Option<u16>,
    pub reply: Option<String>,
    pub note_id: Option<i64>,
}

/// The voice inlet's routes, ready to `merge` into the remote server's router.
pub fn routes() -> Router<RemoteState> {
    Router::new()
        .route("/display/voice", post(voice_handler))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
}

/// Accepts one dictated note: authenticates, validates, then runs the pipeline.
async fn voice_handler(
    State(state): State<RemoteState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<VoiceAck>, StatusCode> {
    require_token(&headers, state.remote_token.as_deref())?;

    // The `DefaultBodyLimit` layer already refuses oversized bodies; this
    // repeats the check so the handler is safe when called directly.
    if body.len() > MAX_BODY_BYTES {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    let parsed: VoicePostBody =
        serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    let post = parsed.validate().ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;

    Ok(Json(run_pipeline(&state, post).await))
}

/// The five steps, in the order the module docs explain.
async fn run_pipeline(state: &RemoteState, post: ValidVoicePost) -> VoiceAck {
    let intent = voice_intent::parse(&post.transcript);

    if let Some(logger) = &state.voice.event_logger {
        logger.log(&format!(
            "VOICE {} {}",
            intent.label(),
            voice_intent::truncate_for_log(&post.transcript)
        ));
    }

    let note_id = store_note(state, &post, intent);
    let snooze_minutes = apply_intent(state, intent);
    let reply = ai_reply(state, &post.transcript).await;
    if let (Some(id), Some(text)) = (note_id, reply.as_deref()) {
        with_conn(state, |conn| set_reply(conn, id, text));
    }

    let ack = VoiceAck {
        transcript: post.transcript,
        intent: intent.label().to_string(),
        snooze_minutes,
        reply,
        note_id,
    };

    let event = DisplayEvent::VoiceAck {
        transcript: ack.transcript.clone(),
        intent: ack.intent.clone(),
        reply: ack.reply.clone(),
    };
    ws_broadcaster::broadcast_event(&state.ws_tx, &event);
    push_webhook(state, &ack);
    ack
}

/// Runs one statement against the app database, when there is one open. `None`
/// means it did not run — no database yet, or it failed — and the request
/// continues either way: an unstored note is still parsed and acknowledged.
fn with_conn<T>(
    state: &RemoteState,
    f: impl FnOnce(&Connection) -> Result<T, rusqlite::Error>,
) -> Option<T> {
    let db = state.voice.db.as_ref()?;
    let guard = db.lock().unwrap_or_else(|e| e.into_inner());
    match f(guard.as_ref()?) {
        Ok(value) => Some(value),
        Err(e) => {
            log::warn!("voice: database write failed: {e}");
            None
        }
    }
}

/// Inserts the note, returning its row id.
fn store_note(state: &RemoteState, post: &ValidVoicePost, intent: Intent) -> Option<i64> {
    with_conn(state, |conn| {
        insert_voice_note(
            conn,
            &NewVoiceNote {
                captured_at_ms: post.captured_at_ms,
                transcript: &post.transcript,
                lang: post.lang.as_deref(),
                intent: intent.label(),
            },
        )
    })
}

/// Applies the intent's side effect and reports the snooze length, if any.
///
/// Walk boundaries are recorded, not enacted: the sensor and the activity
/// tracker decide what state the user is in, and a dictated sentence must not
/// overrule them.
fn apply_intent(state: &RemoteState, intent: Intent) -> Option<u16> {
    match intent {
        Intent::Snooze(mins) => {
            let mut policy = state.comm_policy.lock().unwrap_or_else(|e| e.into_inner());
            policy.snooze_for(mins);
            Some(mins)
        }
        Intent::Note | Intent::WalkStart | Intent::WalkEnd => None,
    }
}

/// Asks the model for a reply, when one is configured. Every failure — no key,
/// timeout, rejected key — collapses to `None`. The session snapshot is taken
/// and its lock released before the `await`: a `std::sync` guard may not cross
/// one.
async fn ai_reply(state: &RemoteState, transcript: &str) -> Option<String> {
    let ai = state.voice.ai.as_ref()?;
    let snapshot = {
        let session = state.session.lock().unwrap_or_else(|e| e.into_inner());
        session.snapshot()
    };
    match ai.reply(transcript, &snapshot).await {
        Ok(reply) => reply,
        Err(e) => {
            log::warn!("voice: AI reply unavailable: {e:?}");
            None
        }
    }
}

/// Mirrors the acknowledgement to the notification webhook, when configured.
fn push_webhook(state: &RemoteState, ack: &VoiceAck) {
    let webhook = WebhookNotifier::from_env_or_config(
        state.voice.webhook_enabled,
        state.voice.webhook_url.as_deref(),
    );
    if let Some(notifier) = webhook {
        notifier.send(voice_notification(ack));
    }
}

/// The webhook body for one acknowledgement: the AI reply when there is one and
/// the transcript otherwise, so a phone that only sees the push still learns
/// what was heard.
pub fn voice_notification(ack: &VoiceAck) -> Notification {
    Notification {
        title: format!("Voice note ({})", ack.intent),
        message: ack.reply.clone().unwrap_or_else(|| ack.transcript.clone()),
        priority: VOICE_PRIORITY,
        tags: vec!["speech_balloon".to_string()],
    }
}

// Tests live in remote_routes_voice_tests.rs.
