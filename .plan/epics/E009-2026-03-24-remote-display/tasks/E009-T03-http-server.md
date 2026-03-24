---
id: E009-T03
epic: E009
status: pending
created: 2026-03-24
branch: feat/E009-T03-http-server
depends_on: [E009-T02]
---
# E009-T03: Embedded HTTP + WebSocket Server (axum)

## What
Add an axum HTTP server to the Tauri backend that:
1. Serves the built React frontend as static files at `/display`
2. Exposes a WebSocket endpoint at `/display/ws` for real-time events
3. Exposes a REST endpoint at `/display/api` for debugging/fallback

## Context
This is the core server that phones (and browsers) connect to.
It runs alongside Tauri on a separate port (default 3390).

The React frontend is already built by Vite into `src-tauri/target/...` during
`pnpm tauri:build`. For development, we'll serve from the Vite dist directory.

## Dependencies
- **E009-T02** (ws_broadcaster) must be done first — we need `ws_tx` in AppState

## Implementation

### New dependencies in `Cargo.toml`
```toml
axum = "0.8"
tower-http = { version = "0.6", features = ["fs", "cors"] }
reqwest = { version = "0.12", features = ["rustls-tls"], optional = true }  # dev proxy only
```

### CEO Review Fixes Applied
1. **Dev mode proxy** — in `cfg(debug_assertions)`, reverse proxy `/display/*` to Vite
   dev server (localhost:1443) instead of serving static files. Phone gets HMR.
2. **Max 10 WS clients** — `AtomicUsize` counter, reject with 503 if exceeded.
3. **Graceful port binding** — `match` instead of `expect()`, log error and continue.
4. **Log active client count** — on connect/disconnect.
5. **Mutex poison recovery** — use `.lock().unwrap_or_else(|e| e.into_inner())` for
   read-only access from WS handlers.

### New file: `src-tauri/src/remote_server.rs` (~120 lines)

```rust
use std::sync::Arc;
use axum::{
    Router,
    extract::{State, WebSocketUpgrade, ws},
    response::IntoResponse,
    routing::get,
    Json,
};
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

/// Shared state for the remote display server.
#[derive(Clone)]
pub struct RemoteState {
    /// Broadcast receiver factory — each WS client subscribes here
    pub ws_tx: broadcast::Sender<String>,
    /// Reference to session manager for snapshot on connect
    pub session: Arc<Mutex<SessionManager>>,
    /// Reference to DB for today summary on connect
    pub db: Arc<Mutex<Option<Connection>>>,
    /// Reference to config for metric computation on connect
    pub config: Arc<Mutex<Option<AppConfig>>>,
    /// Active WS client counter (max 10)
    pub active_clients: Arc<AtomicUsize>,
}

/// Max simultaneous WS clients. Protects against accidental DoS on LAN.
const MAX_WS_CLIENTS: usize = 10;

/// Starts the HTTP+WS server on the given port.
/// Call from lib.rs setup, spawned as a tokio task (non-blocking).
/// Gracefully handles port binding failure (logs error, does NOT crash app).
pub async fn start(state: RemoteState, port: u16) {
    let app = Router::new()
        .route("/display/ws", get(ws_handler))
        .route("/display/api", get(api_handler))
        // Dev mode: proxy to Vite; Prod: serve static files
        .fallback_service(frontend_service())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            log::error!("Remote display server failed to start on {}: {}. \
                         Remote display will be unavailable. \
                         Check if port {} is already in use.", addr, e, port);
            return; // Don't crash the app — remote display is optional
        }
    };

    log::info!("Remote display server listening on {}", addr);
    if let Err(e) = axum::serve(listener, app).await {
        log::error!("Remote display server stopped: {}", e);
    }
}

/// Returns the frontend serving strategy:
/// - Debug builds: reverse proxy to Vite dev server (localhost:1443)
/// - Release builds: serve static files from dist/
fn frontend_service() -> /* impl Service */ {
    #[cfg(debug_assertions)]
    {
        // Reverse proxy to Vite dev server.
        // If Vite is not running, return helpful error page.
        // Implementation: use axum handler that proxies requests to
        // http://localhost:1443/ — or return HTML error if unreachable:
        // "Vite dev server not running. Start with: pnpm dev"
    }
    #[cfg(not(debug_assertions))]
    {
        ServeDir::new(frontend_dist_path())
    }
}
```

### WebSocket handler

```rust
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<RemoteState>,
) -> Result<impl IntoResponse, axum::http::StatusCode> {
    // Enforce max client limit
    let current = state.active_clients.load(Ordering::Relaxed);
    if current >= MAX_WS_CLIENTS {
        log::warn!("Remote display: rejected WS connection (max {} clients reached)", MAX_WS_CLIENTS);
        return Err(axum::http::StatusCode::SERVICE_UNAVAILABLE);
    }
    Ok(ws.on_upgrade(|socket| handle_ws_client(socket, state)))
}

async fn handle_ws_client(mut socket: ws::WebSocket, state: RemoteState) {
    // Track active clients
    let count = state.active_clients.fetch_add(1, Ordering::Relaxed) + 1;
    log::info!("Remote display client connected ({} active)", count);

    // 1. Send current state snapshot immediately on connect
    // Build full RemoteDisplayState (session + metrics + today) — same as broadcast
    let initial_state = build_remote_display_state(&state);
    let initial = DisplayEvent::Snapshot(initial_state);
    if let Ok(json) = serde_json::to_string(&initial) {
        let _ = socket.send(ws::Message::Text(json)).await;
    }

    // 2. Subscribe to broadcast channel
    let mut rx = state.ws_tx.subscribe();

    // 3. Forward events to client + send heartbeat
    let mut heartbeat = tokio::time::interval(std::time::Duration::from_secs(5));

    loop {
        tokio::select! {
            // Broadcast event received → forward to client
            result = rx.recv() => {
                match result {
                    Ok(msg) => {
                        if socket.send(ws::Message::Text(msg)).await.is_err() {
                            break; // Client disconnected
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        log::debug!("Remote display client lagged, skipped {} messages", n);
                        // Continue — client will get next message
                    }
                    Err(_) => break, // Channel closed
                }
            }
            // Heartbeat tick
            _ = heartbeat.tick() => {
                let hb = r#"{"event":"heartbeat","payload":null}"#;
                if socket.send(ws::Message::Text(hb.into())).await.is_err() {
                    break;
                }
            }
            // Client sent something (we ignore, but need to detect close)
            msg = socket.recv() => {
                match msg {
                    Some(Ok(_)) => {} // Ignore client messages
                    _ => break, // Closed or error
                }
            }
        }
    }

    let count = state.active_clients.fetch_sub(1, Ordering::Relaxed) - 1;
    log::info!("Remote display client disconnected ({} active)", count);
}

/// Builds the full RemoteDisplayState for initial WS handshake.
/// Uses unwrap_or_else for mutex poison recovery (read-only, safe to recover).
fn build_remote_display_state(state: &RemoteState) -> RemoteDisplayState {
    let session = state.session.lock()
        .unwrap_or_else(|e| e.into_inner())
        .snapshot();
    // Compute metrics + today summary — same pattern as get_dashboard_state
    // See commands.rs for the exact calling convention.
    // ...
    RemoteDisplayState { session, metrics: vec![], today: Default::default() }
}
```

### REST fallback handler

```rust
async fn api_handler(State(state): State<RemoteState>) -> Json<serde_json::Value> {
    // Use poison-recovery lock for read-only access
    let snapshot = state.session.lock()
        .unwrap_or_else(|e| e.into_inner())
        .snapshot();
    Json(serde_json::to_value(&snapshot).unwrap_or_default())
}
```

### Static file serving & dev proxy

**Production** (`cfg(not(debug_assertions))`):
- Serve from `../../dist` (relative to `src-tauri/`), or `DESK_REMOTE_DIST` env var override
- Files built by `pnpm build` before `pnpm tauri:build`

**Development** (`cfg(debug_assertions)`):
- Reverse proxy all `/display/*` requests to Vite dev server at `localhost:1443`
- Phone gets HMR — change React code, phone auto-updates
- If Vite is not running, return HTML error page:
  `"<h1>Vite dev server not running</h1><p>Start with: pnpm dev</p>"`
- Implementation: axum handler that uses `reqwest` to proxy to `http://localhost:1443/`
  and forwards the response. On error, return the helpful HTML above.

### Changes to `lib.rs`

In the Tauri setup closure, after existing initialization:
```rust
// Start remote display server (non-blocking)
let remote_state = remote_server::RemoteState {
    ws_tx: app_state.ws_tx.clone(),
    session: app_state.session.clone(),
    db: app_state.db.clone(),
    config: app_state.config.clone(),
    active_clients: Arc::new(AtomicUsize::new(0)),
};
let port = std::env::var("DESK_REMOTE_PORT")
    .ok()
    .and_then(|p| p.parse().ok())
    .unwrap_or(3390);
tauri::async_runtime::spawn(async move {
    remote_server::start(remote_state, port).await;
});
```

### Port configuration

| Env var | Default | Purpose |
|---------|---------|---------|
| `DESK_REMOTE_PORT` | `3390` | HTTP+WS server port |
| `DESK_REMOTE_DIST` | `../../dist` | Path to frontend dist (prod override) |
| `DESK_VITE_PORT` | `1443` | Vite dev server port for proxy (dev only) |

## Testing

### Manual testing:
1. `pnpm tauri:dev` → verify log line "Remote display server listening on 0.0.0.0:3390"
2. `curl http://localhost:3390/display/api` → returns JSON session state
3. `curl http://localhost:3390/display` → returns HTML (React app)
4. Use `websocat ws://localhost:3390/display/ws` → see events streaming

### Unit tests (in `remote_server.rs`):
1. `RemoteState` can be cloned (needed for axum)
2. Frontend dist path resolution works
3. **Port binding failure → returns gracefully (no panic)**
4. **11th WS connection → rejected with 503**
5. **Active client counter increments/decrements correctly**

### Integration tests:
- Server starts without panicking
- API endpoint returns valid JSON with `session`, `metrics`, `today` fields
- WS endpoint accepts connection and sends full snapshot (session + metrics + today)
- WS connection rejected when max clients reached

## Definition of Done
- [ ] `remote_server.rs` handles HTTP + WS
- [ ] Server starts on `pnpm tauri:dev` (log line visible)
- [ ] **Graceful error if port in use (log, don't crash)**
- [ ] **Max 10 WS clients (AtomicUsize counter, 503 on overflow)**
- [ ] **Log active client count on connect/disconnect**
- [ ] **Dev mode: reverse proxy to Vite (:1443) with fallback error page**
- [ ] **Prod mode: serve static files from dist/**
- [ ] **Mutex access uses poison recovery (unwrap_or_else)**
- [ ] `GET /display` serves React app (or proxies to Vite in dev)
- [ ] `GET /display/api` returns JSON snapshot
- [ ] `WS /display/ws` streams events in real-time
- [ ] WS initial snapshot includes session + metrics + today summary
- [ ] Heartbeat every 5s
- [ ] Multiple clients work simultaneously
- [ ] Port configurable via `DESK_REMOTE_PORT`
- [ ] All existing tests pass
