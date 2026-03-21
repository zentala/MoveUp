# T024 — Investigate + fix notifications

**Status:** open
**Priority:** P1 (bug — user never sees any notification)
**Branch:** feat/T024-notifications-fix

---

## Problem

User has never seen a notification from this app despite the notification infrastructure being implemented.

---

## Current Notification Implementation

### Where notifications fire: `src-tauri/src/serial.rs`

Notifications are checked every 60 seconds (`last_notification_check`). Two types from `session.check_notification_conditions()`:

```rust
// session.rs line ~308
pub fn check_notification_conditions(&mut self, config: &AppConfig) -> Vec<NotificationEvent> {
    // 1. Inactivity: no position change for >= 90 minutes (fires max 1x/hour)
    if config.notify_inactivity && !self.notify_inactivity_fired {
        if elapsed_secs >= 90 * 60 { ... }
    }
    // 2. Posture balance: sitting > 2× standing (fires max 1x/day)
    if config.notify_daily_posture_balance && !self.notify_posture_balance_fired {
        if self.state.sitting_seconds > self.state.standing_seconds * 2 { ... }
    }
}
```

Also in `serial.rs`:
- `Praise` — fires on Sitting→Standing transition if standing_secs >= stand_limit/2 (once/day)
- `StandLimitReached` — when standing exceeds stand_limit_secs

### Notification sending: `src-tauri/src/serial.rs` + `src-tauri/src/lib.rs`

Uses `tauri_plugin_notification::NotificationExt`:
```rust
handle.notification()
    .builder()
    .title("zntlDesk")
    .body("...")
    .show()
```

---

## Likely Causes of Notifications Not Appearing

1. **Thresholds too high**: 90 min inactivity is a long time; posture balance fires once/day max
2. **Windows Focus Assist / Do Not Disturb**: automatically suppresses toasts during:
   - Fullscreen apps (games, videos)
   - Presentation mode
   - Custom user schedules
3. **App not in Windows notification allowlist**: Windows 11 may have the app blocked
4. **`tauri_plugin_notification` manifest/permission**: may need `<requestedExecutionLevel>` in manifest or specific permission in `tauri.conf.json`
5. **Notifications fire but are silently dropped**: no error propagation (uses `let _ = ...`)

---

## Fix Plan

### Step 1 — Test notification command (production-ready, with rate-limit)

**Important decision (eng review 2026-03-21):** This command is NOT debug-only. It is exposed in
production builds so users can test notifications from the welcome popup (T025) and settings panel.
Rate-limited to prevent spam.

Add to `src-tauri/src/commands.rs`:

```rust
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static LAST_TEST_NOTIFICATION: AtomicI64 = AtomicI64::new(0);

/// Sends a test notification to verify notifications are working.
/// Rate-limited: max once per 60 seconds to prevent spam.
#[tauri::command]
pub fn trigger_test_notification(app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_notification::NotificationExt;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let last = LAST_TEST_NOTIFICATION.load(Ordering::Relaxed);
    if now - last < 60 {
        return Err(format!("Rate limited: wait {}s", 60 - (now - last)));
    }
    LAST_TEST_NOTIFICATION.store(now, Ordering::Relaxed);
    app.notification()
        .builder()
        .title("zntlDesk — Test")
        .body("Powiadomienia działają! 🎉")
        .show()
        .map_err(|e| format!("Notification error: {}", e))
}
```

Register in `lib.rs` invoke_handler **without** `#[cfg(debug_assertions)]`.

**If `show()` returns Ok but user sees nothing:** Windows may be suppressing toasts silently
(Focus Assist, app not registered). In that case, auto-switch notification backend to popup:

The command returns `Ok(())` even when toast is suppressed. T025 welcome popup button shows:
"Przetestuj powiadomienia 🔔" — if user clicks and nothing appears, they manually switch via
Settings (`notification_backend = "popup"`). No automatic detection possible at this layer.

Add a "Test notifications" button in Settings panel (always visible, not dev-only):
```tsx
<button onClick={async () => {
  try {
    await invoke('trigger_test_notification');
    setTestStatus('sent');
  } catch (e) {
    setTestStatus('rate_limited');
  }
}}>
  🔔 Przetestuj powiadomienia
</button>
{testStatus === 'sent' && <p>Wysłano! Czy widzisz powiadomienie?</p>}
```

### Step 2 — Lower thresholds for testing

Change thresholds to be more realistic:
```rust
// session.rs — check_notification_conditions()
// Inactivity: 90 min → 60 min
if elapsed_secs >= 60 * 60 {
```

### Step 3 — Add missing notification events

Add `StandingTargetReached` event — when user stands for full standing target (15 min):

In `session.rs` `NotificationEvent` enum:
```rust
pub enum NotificationEvent {
    Inactivity,
    PostureBalance,
    Praise,
    StandLimitReached,
    StandingTargetReached,  // ← NEW: user completed full standing session
}
```

Fire it in `serial.rs` when `standing_secs >= stand_limit_secs && !standing_target_reached_fired`.

Notification body: `"Świetna przerwa! 🏆 Stałeś pełne 15 minut."`

### Step 4 — Feature flag for notification backend

Add to `AppConfig`:
```rust
/// Which notification backend to use.
/// "toast" = Windows native toast (tauri_plugin_notification)
/// "popup" = Custom WinAPI popup (same as AlertPopup)
/// "both"  = Show both (dev A/B comparison)
#[serde(default = "default_notification_backend")]
pub notification_backend: String,

fn default_notification_backend() -> String { "toast".to_string() }
```

In `send_notification()` helper function (extract from serial.rs):
```rust
fn send_notification(app: &AppHandle, title: &str, body: &str, config: &AppConfig) {
    match config.notification_backend.as_str() {
        "popup" => { /* use AlertPopup infrastructure */ }
        "both" => { send_toast(app, title, body); send_popup(app, body); }
        _ => { send_toast(app, title, body); } // default: toast
    }
}
```

Note: "popup" and "both" are for development/testing. Long-term plan is custom popup design (T018).

---

## Key Files

| File | Role |
|------|------|
| `src-tauri/src/serial.rs` | Notification check loop, sends notifications |
| `src-tauri/src/session.rs` | `check_notification_conditions()`, `NotificationEvent` enum, thresholds |
| `src-tauri/src/commands.rs` | Add `trigger_test_notification` debug command |
| `src-tauri/src/config.rs` | Add `notification_backend` field to `AppConfig` |
| `src-tauri/src/lib.rs` | Register debug command in invoke_handler |
| `src/App.tsx` or dev panel | Add "Test notification" button (dev mode only) |
| `tauri.conf.json` | Check notification permissions |

## `AppState` structure (for commands.rs reference)

```rust
pub struct AppState {
    pub session: Arc<Mutex<SessionManager>>,
    pub overlay: Arc<OverlayRenderer>,
    pub alert_manager: Arc<Mutex<AlertManager>>,
    pub alert_popup: Arc<Mutex<AlertPopup>>,
}
```

---

## Tests (~5)

- `trigger_test_notification` command exists in debug builds
- `check_notification_conditions` fires Inactivity after 60 min (not 90)
- `StandingTargetReached` event fires when `standing_secs >= stand_limit_secs`
- `StandingTargetReached` does NOT fire twice (flag resets on daily reset)
- `notification_backend` field deserializes correctly with default "toast"

---

## Acceptance Criteria

- [ ] Click "Test notification" in debug UI → Windows notification appears
- [ ] `trigger_test_notification` IPC command returns Ok (not Err)
- [ ] Inactivity threshold lowered to 60 min
- [ ] `StandingTargetReached` fires when standing target complete
- [ ] `notification_backend` config field added with default "toast"
- [ ] `cargo test` passes
