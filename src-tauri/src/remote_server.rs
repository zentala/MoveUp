//! remote_server.rs — Embedded HTTP + WebSocket server for remote display.
//!
//! Serves the React frontend as static files and provides a WebSocket
//! endpoint for real-time event streaming to phone/browser clients.

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
use crate::metrics::MetricEngine;
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
}

/// Max simultaneous WS clients. Protects against accidental DoS on LAN.
pub(crate) const MAX_WS_CLIENTS: usize = 10;

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
fn build_router(state: RemoteState) -> Router {
    let router = Router::new()
        .route("/display/ws", get(ws_handler))
        .route("/display/api", get(api_handler));

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
    let initial = build_remote_display_state(&state);
    let event = DisplayEvent::Snapshot(initial);
    if let Ok(json) = serde_json::to_string(&event) {
        let _ = socket.send(ws::Message::Text(json.into())).await;
    }

    // Subscribe to broadcast channel
    let mut rx = state.ws_tx.subscribe();
    let mut heartbeat = tokio::time::interval(std::time::Duration::from_secs(5));

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(msg) => {
                        if socket.send(ws::Message::Text(msg.into())).await.is_err() {
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
                let hb = r#"{"event":"heartbeat","payload":null}"#;
                if socket.send(ws::Message::Text(hb.to_string().into())).await.is_err() {
                    break;
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(_)) => {}
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
    let display = build_remote_display_state(&state);
    Json(serde_json::to_value(&display).unwrap_or_default())
}

/// Builds the full [`RemoteDisplayState`] for initial WS handshake or REST.
/// Uses `unwrap_or_else` for mutex poison recovery (read-only, safe).
fn build_remote_display_state(state: &RemoteState) -> RemoteDisplayState {
    let session_mgr = state
        .session
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let snapshot = session_mgr.snapshot();

    let now = chrono::Utc::now();
    let mut raw = session_mgr.state.clone();
    raw.sitting_seconds_total = session_mgr.get_live_sitting_seconds_total(now);
    raw.standing_seconds = session_mgr.get_live_standing_seconds(now);
    drop(session_mgr);

    let ergo = state.comm_policy
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .ergo_profile()
        .clone();

    let metrics = MetricEngine::with_defaults().compute_all(&raw, &ergo);
    let today = state
        .today_cache
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();

    RemoteDisplayState {
        session: snapshot,
        metrics,
        today,
    }
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
