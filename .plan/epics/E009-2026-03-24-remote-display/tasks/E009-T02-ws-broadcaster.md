---
id: E009-T02
epic: E009
status: done
completed: 2026-03-25
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

In `update_overlay_progress()` — after snapshot is computed, broadcast lightweight
snapshot with metrics (all from memory, **no SQLite queries**):
```rust
// Broadcast session + metrics to remote clients (~1/s)
// All data is from memory — zero DB access in the hot path.
let app_state = app.state::<AppState>();
let ws_tx = app_state.ws_tx.clone();

let remote_state = {
    let config_guard = app_state.config.lock().unwrap();
    let config = config_guard.as_ref().cloned().unwrap_or_default();
    let session_state = app_state.session.lock().unwrap().state_snapshot();

    // Compute metrics from in-memory state (no DB)
    let metrics = MetricEngine::compute_all(&snapshot, &session_state, &config);

    // Today summary: use cached value (see below)
    let today = app_state.today_cache.lock().unwrap().clone();

    RemoteDisplayState { session: snapshot.clone(), metrics, today }
};
ws_broadcaster::broadcast_event(&ws_tx, &DisplayEvent::Snapshot(remote_state));
```

### IMPORTANT: TodaySummaryDto caching strategy (Eng Review fix)

**DO NOT query SQLite every second.** `TodaySummaryDto` (list of today's sessions)
changes only on **state transitions** (new session row inserted). Strategy:

1. **Add to `AppState`:** `pub today_cache: Arc<Mutex<TodaySummaryDto>>`
2. **Load on startup:** populate from DB (same as `load_today_totals`)
3. **Refresh on state change:** in `on_state_changed()`, re-query DB and update cache
4. **Broadcast uses cache:** hot path reads `today_cache` from memory (no DB)
5. **Daily reset:** clear cache

This means the ~1/s broadcast does: 1 mutex lock (session) + 1 mutex lock (config) +
1 mutex lock (today_cache) + metric computation (pure CPU, ~0.1ms) + serialize.
**Zero SQLite in the hot path.**

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
- [ ] `AppState` has `ws_tx` field + `today_cache: Arc<Mutex<TodaySummaryDto>>`
- [ ] `tray_controller.rs` broadcasts `RemoteDisplayState` (session + metrics + cached today) ~1/s
- [ ] **Zero SQLite queries in the hot path** — today_cache refreshed only on state transitions
- [ ] `tray_controller.rs` broadcasts `StateChanged` on state transitions + refreshes today_cache
- [ ] `tray_controller.rs` broadcasts `DailyReset` on midnight reset + clears today_cache
- [ ] `tray_controller.rs` broadcasts `DeviceConnected`/`DeviceLost` on sensor events
- [ ] Unit tests pass
- [ ] All existing tests still pass (`cargo test`)
