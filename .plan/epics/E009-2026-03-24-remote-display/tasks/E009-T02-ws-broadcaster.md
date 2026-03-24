---
id: E009-T02
epic: E009
status: pending
created: 2026-03-24
branch: feat/E009-T02-ws-broadcaster
---
# E009-T02: WebSocket Broadcaster Module

## What
Create a tokio broadcast channel that fans out desk events to all connected
WebSocket clients. This is the core pub/sub mechanism — event producers send
to the channel, WS clients receive from it.

**CRITICAL (from CEO review):** The broadcast must send `RemoteDisplayState`
(session + metrics + today summary), NOT just `SessionStateDto`. Without
metrics, the phone won't show KPI badges. Without today summary, no timeline.

## Context
Currently, all desk events flow through Tauri's `app.emit()` system which is
local-only (Tauri IPC). We need a parallel broadcast path for remote clients.

The existing event flow in `tray_controller.rs` listens to `desk:state-changed`
and `desk:distance` events. We'll add a broadcast sender alongside these.

## Implementation

### New file: `src-tauri/src/ws_broadcaster.rs` (~80 lines)

```rust
use tokio::sync::broadcast;
use serde::Serialize;

/// Full state sent to remote display clients on every tick (~1/s).
/// Includes session state + KPI metrics + today's session list.
/// This matches what `get_dashboard_state` + `get_today_summary` return.
#[derive(Clone, Debug, Serialize)]
pub struct RemoteDisplayState {
    pub session: SessionStateDto,
    pub metrics: Vec<MetricSnapshot>,
    pub today: TodaySummaryDto,
}

/// Message types sent to remote display clients.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "event", content = "payload")]
pub enum DisplayEvent {
    /// Full state snapshot (sent on connect + every ~1s)
    /// Contains session + metrics + today summary — everything the UI needs.
    #[serde(rename = "snapshot")]
    Snapshot(RemoteDisplayState),

    /// State transition (sitting→standing, etc.)
    #[serde(rename = "desk:state-changed")]
    StateChanged(serde_json::Value),

    /// Device connected/lost
    #[serde(rename = "desk:device-connected")]
    DeviceConnected { port: String },
    #[serde(rename = "desk:device-lost")]
    DeviceLost,

    /// Daily counter reset
    #[serde(rename = "desk:daily-reset")]
    DailyReset,

    /// Keep-alive
    #[serde(rename = "heartbeat")]
    Heartbeat,
}

/// Creates the broadcast channel. Returns (sender, receiver_factory).
/// Sender goes into AppState. Receivers are created per WS client.
pub fn create_channel() -> broadcast::Sender<String> {
    let (tx, _) = broadcast::channel::<String>(64);
    tx
}

/// Serializes a DisplayEvent and sends it to the broadcast channel.
/// Silently drops if no receivers are listening (fire-and-forget).
pub fn broadcast_event(tx: &broadcast::Sender<String>, event: &DisplayEvent) {
    if let Ok(json) = serde_json::to_string(event) {
        let _ = tx.send(json); // Ignore SendError (no receivers = ok)
    }
}
```

### Changes to `commands.rs` → `AppState`

Add field:
```rust
pub ws_tx: broadcast::Sender<String>,
```

Initialize in `AppState` construction (in `lib.rs` setup):
```rust
ws_tx: ws_broadcaster::create_channel(),
```

### Changes to `tray_controller.rs`

In `on_state_changed()` — after existing overlay/tray logic, add:
```rust
// Broadcast to remote display clients
let ws_tx = app.state::<AppState>().ws_tx.clone();
if let Ok(json) = serde_json::to_string(&payload) {
    ws_broadcaster::broadcast_event(&ws_tx, &DisplayEvent::StateChanged(
        serde_json::to_value(payload).unwrap_or_default()
    ));
}
```

In `update_overlay_progress()` — after snapshot is computed, build the full
`RemoteDisplayState` and broadcast:
```rust
// Broadcast full display state to remote clients (~1/s)
let app_state = app.state::<AppState>();
let ws_tx = app_state.ws_tx.clone();

// Build RemoteDisplayState with session + metrics + today summary
let remote_state = {
    let config_guard = app_state.config.lock().unwrap();
    let config = config_guard.as_ref().cloned().unwrap_or_default();
    let session_state = app_state.session.lock().unwrap().state_snapshot();

    // Compute metrics (same logic as get_dashboard_state command)
    let metrics = MetricEngine::compute_all(&snapshot, &session_state, &config);

    // Get today summary from DB
    let today = app_state.db.lock().unwrap().as_ref()
        .and_then(|conn| crate::db::load_today_summary(conn).ok())
        .unwrap_or_default();

    RemoteDisplayState { session: snapshot.clone(), metrics, today }
};
ws_broadcaster::broadcast_event(&ws_tx, &DisplayEvent::Snapshot(remote_state));
```

**Note:** The above is pseudo-code. The actual implementation depends on how
`MetricEngine::compute_all()` and `load_today_summary()` are called. Look at
`commands.rs` → `get_dashboard_state` and `get_today_summary` for the real
calling pattern and replicate it here.

Also listen for `desk:daily-reset` and broadcast it:
```rust
// In setup() or a new listener in tray_controller.rs:
let handle3 = app.clone();
app.listen("desk:daily-reset", move |_event| {
    let ws_tx = handle3.state::<AppState>().ws_tx.clone();
    ws_broadcaster::broadcast_event(&ws_tx, &DisplayEvent::DailyReset);
});
```

### Changes to `Cargo.toml`

No changes needed here — `tokio` with `full` features is already a dependency,
which includes `broadcast`. `serde` and `serde_json` are also already present.

## Testing

### Unit tests in `ws_broadcaster.rs`:
1. `broadcast_event` serializes correctly
2. Multiple receivers get the same message
3. No panic when no receivers exist (fire-and-forget)
4. Channel capacity (64) doesn't block sender when slow receiver

### Integration verification:
- Start app, verify no errors in logs about broadcast channel
- Existing tests must still pass (broadcast is additive, no breaking changes)

## Definition of Done
- [ ] `ws_broadcaster.rs` exists with `create_channel()`, `broadcast_event()`, `RemoteDisplayState`
- [ ] `AppState` has `ws_tx` field
- [ ] `tray_controller.rs` broadcasts `RemoteDisplayState` (session + metrics + today) on every tick
- [ ] `tray_controller.rs` broadcasts `StateChanged` on state transitions
- [ ] `tray_controller.rs` broadcasts `DailyReset` on midnight reset
- [ ] `tray_controller.rs` broadcasts `DeviceConnected`/`DeviceLost` on sensor events
- [ ] Unit tests pass
- [ ] All existing tests still pass (`cargo test`)
