# T021 — Fix: Dismiss snooze not triggered when sensor disconnected

**Status:** Open
**Priority:** P3
**Depends on:** T013+T014, T015 (both done)

---

## The Bug

When the user clicks Dismiss in the alert popup, snooze is triggered via this chain:

```
user clicks button
  → alert_popup_window.rs sets user_dismissed = true
  → next desk:distance event fires (~1/s)
  → tray_controller::update_overlay_progress() checks take_user_dismissed()
  → calls alert_manager.dismiss()
  → snooze takes effect
```

**Problem:** If the sensor is disconnected (no `desk:distance` events), `take_user_dismissed()` is never checked. The user clicked Dismiss, popup is gone visually, but `AlertManager` stays in `Stage2`. When sensor reconnects, the popup immediately re-appears without any snooze delay.

---

## Root Cause

`take_user_dismissed()` is only polled inside `update_overlay_progress()`, which is gated on `desk:distance` events. No events = no poll.

---

## Recommended Fix

Add a dedicated background watcher thread in `alert_popup.rs` that monitors `user_dismissed` and emits a Tauri app event when triggered. This avoids coupling `AlertPopup` to `AppHandle` directly.

### Option A: Watcher thread in AlertPopup (recommended)

```rust
// In AlertPopup::show(), alongside the popup thread:
let ud = Arc::clone(&self.user_dismissed);
let app_handle = self.app_handle.clone(); // need to store AppHandle in AlertPopup
std::thread::spawn(move || {
    loop {
        if ud.load(Ordering::SeqCst) {
            app_handle.emit("alert:user-dismissed", ()).ok();
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
});
```

In `tray_controller.rs`, register a listener for `alert:user-dismissed`:
```rust
app.listen("alert:user-dismissed", move |_| {
    let actions = alert_manager.lock().unwrap().dismiss();
    execute_alert_actions(&handle, &actions);
});
```

Remove the `take_user_dismissed()` check from `update_overlay_progress()`.

**Pros:** Event-driven, no polling delay, clean separation.
**Cons:** AlertPopup needs AppHandle stored at construction time (pass in `new()` or `setup()`).

### Option B: Poll in on_state_changed too

In `tray_controller::on_state_changed()`, add the same `take_user_dismissed()` check. This covers state-change events (desk reconnect triggers a state event). Simple but still has a gap if no state change fires.

**Verdict:** Option A is cleaner. Option B is a 2-line quick patch if Option A scope is too large.

---

## Files to Modify

| File | Change |
|------|--------|
| `src-tauri/src/alert_popup.rs` | Store `AppHandle`, spawn watcher thread in `show()` |
| `src-tauri/src/commands.rs` | Pass `AppHandle` when constructing `AlertPopup` |
| `src-tauri/src/lib.rs` | Update `AlertPopup::new()` call |
| `src-tauri/src/tray_controller.rs` | Add `alert:user-dismissed` listener, remove `take_user_dismissed()` from `update_overlay_progress()` |

---

## Acceptance Criteria

- [ ] `cargo test --lib` still passes (129+ tests)
- [ ] Manual test: disconnect sensor while popup visible → click Dismiss → reconnect sensor → snooze delay is respected (popup does not immediately reappear)
- [ ] `take_user_dismissed()` removed from `update_overlay_progress()` (no longer needed)
