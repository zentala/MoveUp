# T023 — Fix tooltip while standing

**Status:** open
**Priority:** P1 (bug)
**Branch:** feat/T023-fix-tooltip-standing

---

## Problem Description

When user stands up, the tray tooltip freezes and shows wrong information:

1. **Timer frozen**: tooltip only updates on `desk:state-changed` events (state transitions). While standing, no event fires every second → tooltip stays stuck at the moment of transition.
2. **Wrong counter**: tooltip always shows `sitting_secs` even when standing. So "Standing (2:03)" means "you had been sitting for 2:03 when you stood up" — not the standing duration.

User report: "shows 2:03 the whole time I'm standing, never changes."

---

## Root Cause in Code

### File: `src-tauri/src/tray_controller.rs`

**Problem 1 — Early return blocks standing updates (line ~128):**
```rust
fn update_overlay_progress(app: &AppHandle) {
    // ...
    if snapshot.state != DeskState::Sitting {
        return;  // ← BUG: exits before updating tooltip when standing
    }
    // ... rest of function only runs for Sitting
}
```
This function is called every second via `desk:distance` listener. But it returns early for non-Sitting states, so tooltip never updates while standing.

**Problem 2 — Tooltip always shows `sitting_secs` (line ~82-86):**
```rust
fn on_state_changed(app: &AppHandle, payload: &StateChangedPayload) {
    // ...
    let label = format!(
        "↕ {:.1} cm — {} ({})",
        payload.desk_height_cm,
        state_str,
        format_duration(sitting_secs),  // ← BUG: always sitting_secs, even when standing
    );
```

---

## Fix

### 1. Update `on_state_changed()` — fix label for non-sitting states

The `StateChangedPayload` struct already has `standing_seconds: i64`. Use it:

```rust
// Current:
format_duration(sitting_secs)

// Fix — pick duration based on state:
let duration_secs = match payload.state {
    DeskState::Sitting => payload.sitting_seconds,
    DeskState::Standing => payload.standing_seconds,
    DeskState::Walking => payload.break_seconds,
    DeskState::Away => 0,
};
format_duration(duration_secs)
```

### 2. Update `update_overlay_progress()` — remove early return for tooltip

Move tooltip update BEFORE the early return:

```rust
fn update_overlay_progress(app: &AppHandle) {
    // ... existing dismiss check ...

    let session = app_state.session.lock().unwrap();
    let snapshot = session.snapshot();

    // ← NEW: Update tooltip every second regardless of state
    update_tooltip(app, &snapshot);

    if snapshot.state != DeskState::Sitting {
        return;  // keep early return for overlay/alert logic
    }
    // ... rest unchanged
}

fn update_tooltip(app: &AppHandle, snapshot: &SessionStateDto) {
    let state_str = match snapshot.state {
        DeskState::Sitting => "Sitting",
        DeskState::Standing => "Standing",
        DeskState::Walking => "Walking",
        DeskState::Away => "Away",
    };
    let duration_secs = match snapshot.state {
        DeskState::Sitting => snapshot.sitting_seconds,
        DeskState::Standing => snapshot.standing_seconds,
        DeskState::Walking => snapshot.break_seconds,
        DeskState::Away => 0,
    };
    let label = format!(
        "↕ {:.1} cm — {} ({})",
        snapshot.desk_height_cm,
        state_str,
        format_duration(duration_secs),
    );
    let _ = tray::update_tray_tooltip(app, &label);
}
```

You'll also need `tray::update_tray_tooltip()` — a new helper in `tray.rs` that only sets the tooltip (no icon change), OR reuse `update_tray()` if it's cheap.

### 3. Check `tray.rs` — separate tooltip from icon update (optional)

`update_tray()` in `tray.rs` currently sets both tooltip AND icon. Calling it every second for tooltip-only is fine (icon doesn't change during standing), but you may want a lighter `update_tray_tooltip(app, label)` function:

```rust
pub fn update_tray_tooltip(app: &AppHandle, label: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(label));
    }
    Ok(())
}
```

---

## Key Data Structures (for reference)

```rust
// session.rs — SessionStateDto (returned by session.snapshot())
pub struct SessionStateDto {
    pub state: DeskState,
    pub sitting_seconds: i64,   // total sitting today
    pub standing_seconds: i64,  // total standing today
    pub break_seconds: i64,     // current break duration
    pub session_limit_secs: i64,
    pub stand_limit_secs: i64,
    pub desk_height_cm: f32,
    pub position_changes: u32,
}

// session.rs — StateChangedPayload (event payload)
pub struct StateChangedPayload {
    pub state: DeskState,
    pub sitting_seconds: i64,
    pub standing_seconds: i64,  // ← available, just not used in tooltip
    pub break_seconds: i64,
    pub desk_height_cm: f32,
    pub position_changes: u32,
}

// session.rs — DeskState
pub enum DeskState { Sitting, Standing, Walking, Away }
```

---

## Files to Modify

| File | Change |
|------|--------|
| `src-tauri/src/tray_controller.rs` | Extract `update_tooltip()`, move before early return, fix duration logic |
| `src-tauri/src/tray.rs` | Add `update_tray_tooltip()` function |

---

## Tests (~6)

In `tray_controller.rs` tests:
- `format_duration(0)` → `"00:00"` (existing, keep)
- `format_duration(60)` → `"01:00"` (existing, keep)
- New: tooltip label for Standing state shows `standing_seconds` (not sitting)
- New: tooltip label for Sitting state shows `sitting_seconds`
- New: tooltip label for Walking shows `break_seconds`

Verify by inspection (or integration test): run app, stand up, tooltip should increment every second.

---

## Acceptance Criteria

- [ ] Tooltip updates every second while standing
- [ ] While standing: shows `↕ 114.0 cm — Standing 7:32` (incrementing)
- [ ] While sitting: shows `↕ 72.0 cm — Sitting 12:34` (unchanged behavior)
- [ ] `cargo test` passes
