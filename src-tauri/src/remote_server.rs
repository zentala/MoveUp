//! remote_server.rs — Embedded HTTP + WebSocket server for remote display.
//!
//! Serves the React frontend as static files and provides a WebSocket
//! endpoint for real-time event streaming to phone/browser clients.
//!
//! Outgoing messages travel in the v1 envelope from
//! [`crate::remote_protocol`] — the one the relay uses too, so a single React
//! client reads both transports. The socket is read-only: the LAN carries no
//! authentication, so nothing a client sends is ever acted on.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::extract::ws::{self, WebSocketUpgrade};
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::{Json, Router};
use tokio::sync::broadcast;
use crate::communication_policy::CommunicationPolicy;
use crate::db::TodaySummary;
use crate::health_source::HealthState;
use crate::remote_protocol::{Envelope, MessageType, PROTOCOL_VERSION};
use crate::remote_routes_health::PushHealthSource;
use crate::session::SessionManager;
use crate::ws_broadcaster::{DisplayEvent, RemoteDisplayState};

/// Shared state for the remote display server.
#[derive(Clone)]
pub struct RemoteState {
    /// Broadcast sender — each WS client subscribes here.
    pub ws_tx: broadcast::Sender<String>,
    /// Reference to session manager for snapshot on connect.
    pub session: Arc<Mutex<SessionManager>>,
    /// Communication policy engine (holds ergonomic + communication profiles).
    pub comm_policy: Arc<Mutex<CommunicationPolicy>>,
    /// Cached today summary (refreshed on state transitions, not per-tick).
    pub today_cache: Arc<Mutex<TodaySummary>>,
    /// Active WS client counter (max 10).
    pub active_clients: Arc<AtomicUsize>,
    /// Merged health view, read by `/display/api` and the WS handshake.
    pub health: HealthState,
    /// Write handle of the LAN push inlet (E021-T03). Separate from `health`
    /// because reading and writing are different privileges: every route may
    /// read the merge, only the authenticated inlet may write a reading.
    pub health_push: Arc<PushHealthSource>,
    /// Shared secret the write endpoints require; `None` closes them.
    pub remote_token: Option<String>,
    /// Dependencies of the voice inlet (E021-T06). Every leg is optional; a
    /// default value still leaves `POST /display/voice` parsing and
    /// acknowledging intents.
    pub voice: VoiceState,
}

/// Everything the voice inlet ([`crate::remote_routes_voice`]) needs that
/// [`RemoteState`] does not already carry. Every leg is independently
/// absent-able — no logger, no database, no AI key and no webhook still leaves
/// a route that parses an intent and acknowledges it — which is why these are
/// `Option`s rather than required fields.
#[derive(Clone, Default)]
pub struct VoiceState {
    /// Writes the `VOICE` line to `logs/YYYY-MM-DD/events.log`.
    pub event_logger: Option<Arc<crate::event_logger::EventLogger>>,
    /// The app database handle — `None` before `ensure_initialized` has run.
    pub db: Option<Arc<Mutex<Option<rusqlite::Connection>>>>,
    /// BYOK OpenRouter client, when the user brought a key.
    pub ai: Option<Arc<crate::voice_ai::VoiceAi>>,
    /// Mirrors [`crate::notify_webhook`]'s configuration.
    pub webhook_enabled: bool,
    pub webhook_url: Option<String>,
}

/// Max simultaneous WS clients. Protects against accidental DoS on LAN.
pub(crate) const MAX_WS_CLIENTS: usize = 10;

/// Key in the persisted `AppConfig` that switches the LAN server off.
pub(crate) const LAN_ENABLED_KEY: &str = "remote_lan_enabled";

/// The keep-alive, pre-serialized as a [`DisplayEvent`] so the heartbeat and
/// every other message share one payload contract.
const HEARTBEAT_EVENT: &str = r#"{"event":"heartbeat","payload":null}"#;

/// Reads the LAN toggle out of a serialized `AppConfig`.
///
/// Read through JSON rather than a typed field on purpose: the field lands
/// with the rest of the remote settings (E022-T06), and a config written by an
/// older build simply does not carry the key. An absent key must keep today's
/// users working, so only an explicit `false` stops the server.
pub(crate) fn lan_enabled_in(config: Option<&serde_json::Value>) -> bool {
    config
        .and_then(|c| c.get(LAN_ENABLED_KEY))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true)
}

/// Wraps one already-serialized [`DisplayEvent`] in the v1 envelope.
///
/// The event's own `{event, payload}` shape travels unchanged as the
/// envelope's `payload`, so a client receives byte-comparable `event` messages
/// whether they arrived over the LAN or the relay.
pub(crate) fn wrap_event(event_json: &str) -> String {
    let payload = serde_json::from_str::<serde_json::Value>(event_json).unwrap_or_else(|e| {
        log::warn!("Remote display: unparseable event forwarded as null: {}", e);
        serde_json::Value::Null
    });
    let envelope = Envelope {
        v: PROTOCOL_VERSION,
        msg_type: MessageType::Event,
        id: uuid::Uuid::new_v4().to_string(),
        ts: chrono::Utc::now().timestamp_millis(),
        payload,
    };
    serde_json::to_string(&envelope).unwrap_or_else(|_| event_json.to_string())
}

/// What a message from a LAN client leads to.
///
/// There is deliberately no `Execute` variant: the LAN listener is
/// unauthenticated, so it stays read-only and commands travel over the relay,
/// which authenticates every viewer.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Inbound {
    Ignored,
    Close,
}

/// Classifies an inbound frame. Anything that is not a close is dropped.
pub(crate) fn classify_inbound(msg: &ws::Message) -> Inbound {
    match msg {
        ws::Message::Close(_) => Inbound::Close,
        _ => {
            log::debug!("Remote display: inbound frame ignored — the LAN transport is read-only");
            Inbound::Ignored
        }
    }
}

/// Default port for the remote display server.
pub const DEFAULT_PORT: u16 = 3390;

/// Starts the HTTP+WS server on the given port.
/// Spawned as a tokio task from lib.rs setup (non-blocking).
/// Gracefully handles port binding failure (logs error, does NOT crash app).
pub async fn start(state: RemoteState, port: u16) {
    let app = build_router(state);
    let addr = format!("0.0.0.0:{}", port);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            log::error!(
                "Remote display server failed to bind on {}: {}. \
                 Remote display will be unavailable. \
                 Check if port {} is already in use.",
                addr,
                e,
                port
            );
            return;
        }
    };

    log::info!("Remote display server listening on {}", addr);
    if let Err(e) = axum::serve(listener, app).await {
        log::error!("Remote display server stopped: {}", e);
    }
}

/// Builds the axum router. Separated for testability.
pub(crate) fn build_router(state: RemoteState) -> Router {
    let router = Router::new()
        .route("/display/ws", get(ws_handler))
        .route("/display/api", get(api_handler))
        .merge(crate::remote_routes_health::routes())
        .merge(crate::remote_routes_voice::routes());

    #[cfg(debug_assertions)]
    let router = router.fallback(get(dev_fallback));

    #[cfg(not(debug_assertions))]
    let router = router.fallback_service(frontend_service());

    router.with_state(state)
}

/// WebSocket upgrade handler. Enforces max client limit.
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<RemoteState>,
) -> Result<impl IntoResponse, axum::http::StatusCode> {
    let current = state.active_clients.load(Ordering::Relaxed);
    if current >= MAX_WS_CLIENTS {
        log::warn!(
            "Remote display: rejected WS connection (max {} clients reached)",
            MAX_WS_CLIENTS
        );
        return Err(axum::http::StatusCode::SERVICE_UNAVAILABLE);
    }
    Ok(ws.on_upgrade(|socket| handle_ws_client(socket, state)))
}

/// Handles a single WebSocket client connection.
async fn handle_ws_client(mut socket: ws::WebSocket, state: RemoteState) {
    let count = state.active_clients.fetch_add(1, Ordering::Relaxed) + 1;
    log::info!("Remote display client connected ({} active)", count);

    // Send current state snapshot immediately on connect
    let initial = build_remote_display_state(&state).await;
    let event = DisplayEvent::Snapshot(initial);
    if let Ok(json) = serde_json::to_string(&event) {
        let _ = socket
            .send(ws::Message::Text(wrap_event(&json).into()))
            .await;
    }

    // Subscribe to broadcast channel
    let mut rx = state.ws_tx.subscribe();
    let mut heartbeat = tokio::time::interval(std::time::Duration::from_secs(5));

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(msg) => {
                        let framed = wrap_event(&msg);
                        if socket.send(ws::Message::Text(framed.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        log::debug!("Remote display client lagged, skipped {} msgs", n);
                    }
                    Err(_) => break,
                }
            }
            _ = heartbeat.tick() => {
                let hb = wrap_event(HEARTBEAT_EVENT);
                if socket.send(ws::Message::Text(hb.into())).await.is_err() {
                    break;
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(m)) => {
                        if classify_inbound(&m) == Inbound::Close {
                            break;
                        }
                    }
                    _ => break,
                }
            }
        }
    }

    let count = state.active_clients.fetch_sub(1, Ordering::Relaxed) - 1;
    log::info!("Remote display client disconnected ({} active)", count);
}

/// REST endpoint returning current session state as JSON.
async fn api_handler(State(state): State<RemoteState>) -> Json<serde_json::Value> {
    let display = build_remote_display_state(&state).await;
    Json(serde_json::to_value(&display).unwrap_or_default())
}

/// Builds the full [`RemoteDisplayState`] for initial WS handshake or REST.
/// The derivation itself lives in [`crate::remote_display_state`], shared with
/// the tray tick so the two paths cannot drift apart.
///
/// Async because it reads the health aggregator through
/// [`view`](crate::health_source::HealthAggregator::view) — the HTTP path can
/// afford a merge across sources, and taking the freshest possible reading
/// here is also what keeps the tray tick's cached read current.
pub(crate) async fn build_remote_display_state(state: &RemoteState) -> RemoteDisplayState {
    let health = state.health.view().await;
    crate::remote_display_state::build(
        &state.session,
        &state.comm_policy,
        &state.today_cache,
        health,
    )
}

/// Dev mode fallback — returns helpful HTML when Vite proxy not yet set up.
#[cfg(debug_assertions)]
async fn dev_fallback() -> Html<&'static str> {
    Html(
        "<h1>Remote Display — Dev Mode</h1>\
         <p>Vite proxy not yet implemented (T05).</p>\
         <p>Use <code>/display/ws</code> for WebSocket or \
         <code>/display/api</code> for REST.</p>",
    )
}

/// Default frontend dist directory, resolved relative to the executable's
/// own location — never the process's current working directory, which
/// varies with how the exe was launched (script, shortcut, scheduled task).
pub(crate) fn default_dist_path(exe_path: &std::path::Path) -> std::path::PathBuf {
    exe_path
        .parent()
        .map(|dir| dir.join("dist"))
        .unwrap_or_else(|| std::path::PathBuf::from("dist"))
}

/// Serves the built React frontend from dist/ in release builds.
#[cfg(not(debug_assertions))]
fn frontend_service() -> tower_http::services::ServeDir {
    let dist = std::env::var("DESK_REMOTE_DIST").unwrap_or_else(|_| {
        std::env::current_exe()
            .map(|exe| default_dist_path(&exe))
            .unwrap_or_else(|_| std::path::PathBuf::from("dist"))
            .to_string_lossy()
            .into_owned()
    });
    tower_http::services::ServeDir::new(dist)
}

// Tests in remote_server_tests.rs
