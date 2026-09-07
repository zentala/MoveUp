//! remote_routes_health.rs — LAN push inlet for health metrics (E021-T03).
//!
//! `POST /display/health` is the interface an Android companion writes to.
//! It is deliberately dumber than an integration: a phone (or a `curl`, or a
//! Tasker task, or the Health Connect companion sketched as candidate E022)
//! posts today's numbers, and the app registers them as one more
//! [`HealthSource`] behind [`HealthAggregator`](crate::health_source::HealthAggregator).
//! Nothing downstream — the widget, the IPC commands, the phone snapshot —
//! learns that a source arrived by push rather than by pull.
//!
//! Three limits, each protecting something specific:
//!
//! - **Token required** ([`crate::remote_auth`]) — this is the one endpoint on
//!   the remote server that writes state, so it is the one that authenticates.
//! - **1 KiB body cap** — the schema's largest legal body is well under 200
//!   bytes; anything larger is a mistake or an attack, and is refused before
//!   `serde` allocates.
//! - **1 h staleness** — a phone that stops pushing must stop *claiming*. After
//!   an hour the source keeps reporting `configured`, but withdraws its
//!   snapshot, so the merge falls back to whatever else is live instead of
//!   showing yesterday's step count as today's.

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::health_models::{HealthSnapshot, HealthView};
use crate::health_source::{HealthSource, HealthState};
use crate::remote_auth::require_token;
use crate::remote_server::RemoteState;

/// [`HealthSource::id`] of the inlet. Distinct from the per-reading
/// `source_id` in the body, which names the *pusher* (`phone`, `curl`, …).
pub const PUSH_SOURCE_ID: &str = "push";

/// Largest accepted request body.
pub const MAX_BODY_BYTES: usize = 1024;

/// A pushed reading older than this stops being offered to the merge.
pub const STALE_AFTER_MS: i64 = 60 * 60 * 1000;

/// Upper bound on `source_id`, matching the schema in `docs/REMOTE_DISPLAY.md`.
const MAX_SOURCE_ID_LEN: usize = 64;

/// Wall-clock unix milliseconds.
fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

/// A [`HealthSource`] fed from outside instead of polled.
///
/// Holds exactly one reading — the latest. There is no history here on
/// purpose: history belongs in the database, and a source that accumulated
/// would have to answer "which of these is today's", which is the pusher's
/// job to decide.
pub struct PushHealthSource {
    latest: RwLock<Option<HealthSnapshot>>,
    stale_after_ms: i64,
}

impl PushHealthSource {
    /// Inlet with the production 1 h staleness window.
    pub fn new() -> Self {
        Self::with_stale_after(STALE_AFTER_MS)
    }

    /// Inlet with a custom staleness window — tests drive the boundary.
    pub fn with_stale_after(stale_after_ms: i64) -> Self {
        Self {
            latest: RwLock::new(None),
            stale_after_ms,
        }
    }

    /// Records a reading, replacing whatever was held before.
    pub async fn push(&self, snapshot: HealthSnapshot) {
        *self.latest.write().await = Some(snapshot);
    }

    /// The view as of `now_ms`, so staleness is testable without sleeping.
    pub async fn view_at(&self, now_ms: i64) -> HealthView {
        let latest = self.latest.read().await.clone();
        let Some(snapshot) = latest else {
            return HealthView::unconfigured();
        };
        let fresh = now_ms.saturating_sub(snapshot.fetched_at_ms) <= self.stale_after_ms;
        HealthView {
            configured: true,
            snapshot: fresh.then_some(snapshot),
            error_kind: None,
            error_message: None,
        }
    }
}

impl Default for PushHealthSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HealthSource for PushHealthSource {
    fn id(&self) -> &str {
        PUSH_SOURCE_ID
    }

    async fn view(&self) -> HealthView {
        self.view_at(now_ms()).await
    }

    /// There is nothing to fetch — the pusher decides when data arrives — so
    /// a refresh reports the same thing a read does.
    async fn refresh(&self) -> HealthView {
        self.view().await
    }
}

/// Creates the push source and registers it with the aggregator.
///
/// Returns the handle the route writes through; the aggregator holds its own
/// clone for reads. Both the app setup and the tests go through here, so the
/// wiring under test is the wiring that ships.
pub async fn register(aggregator: &HealthState) -> Arc<PushHealthSource> {
    let source = Arc::new(PushHealthSource::new());
    aggregator.register(source.clone()).await;
    source
}

/// Request body of `POST /display/health`.
#[derive(Debug, Deserialize)]
pub struct HealthPushBody {
    pub steps_today: u32,
    #[serde(default)]
    pub heart_rate_bpm: Option<u16>,
    #[serde(default)]
    pub hrv_rmssd_ms: Option<f32>,
    pub source_id: String,
    pub measured_at_ms: i64,
}

impl HealthPushBody {
    /// Validates the semantic constraints `serde` cannot express and converts
    /// to the storage shape. `None` means the body parsed but is unusable.
    pub fn into_snapshot(self) -> Option<HealthSnapshot> {
        let source_id = self.source_id.trim().to_string();
        if source_id.is_empty() || source_id.len() > MAX_SOURCE_ID_LEN {
            return None;
        }
        if self.measured_at_ms <= 0 {
            return None;
        }
        if self.hrv_rmssd_ms.is_some_and(|v| !v.is_finite() || v < 0.0) {
            return None;
        }
        Some(HealthSnapshot {
            steps_today: i64::from(self.steps_today),
            heart_rate_bpm: self.heart_rate_bpm,
            hrv_rmssd_ms: self.hrv_rmssd_ms,
            source_id,
            fetched_at_ms: self.measured_at_ms,
        })
    }
}

/// The health inlet's routes, ready to `merge` into the remote server's router.
pub fn routes() -> Router<RemoteState> {
    Router::new()
        .route("/display/health", post(push_handler))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
}

/// Accepts one pushed reading and answers with the merged view.
///
/// Answering with the *merged* view rather than an empty `200` gives the
/// pusher the same picture the app now holds, which is what makes a bare
/// `curl` a usable diagnostic.
async fn push_handler(
    State(state): State<RemoteState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<HealthView>, StatusCode> {
    require_token(&headers, state.remote_token.as_deref())?;

    // The `DefaultBodyLimit` layer already refuses oversized bodies; this
    // repeats the check so the handler is safe when called directly.
    if body.len() > MAX_BODY_BYTES {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }

    let parsed: HealthPushBody =
        serde_json::from_slice(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    let snapshot = parsed
        .into_snapshot()
        .ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;

    log::debug!(
        "health push accepted: source={} steps={}",
        snapshot.source_id,
        snapshot.steps_today
    );
    state.health_push.push(snapshot).await;

    // Reading through the aggregator also refreshes its cached merge, which
    // is what the ~1 s tray broadcast reads — so the phone sees the push on
    // the next tick without the tick having to await anything.
    Ok(Json(state.health.view().await))
}

// Tests in remote_routes_health_tests.rs
