---
id: E001-T10
epic: E001
status: cancelled
original_id: "0010"
title: T010 — Dynamic Tray Icon by Session State
---
# T010 — Dynamic Tray Icon by Session State

**Priority**: P3
**Status**: CANCELLED — replaced by T016 (tray icon redesign)
**Depends on**: T002 (session state events reliable)

## Goal
Change the system tray icon color to reflect the current session urgency:
- Green: sitting, session < 60% of limit
- Amber: sitting, session 60–85% of limit
- Red: sitting, session > 85% (or limit exceeded)
- Default (neutral): not sitting / disconnected

Makes the desk state visible at a glance in the Windows taskbar without opening the app.

## Scope

### Icon assets
Create 4 PNG variants in `src-tauri/icons/tray/`:
- `tray-ok.png` — green dot / green symbol
- `tray-warn.png` — amber dot
- `tray-alert.png` — red dot
- `tray-idle.png` — current default icon

All 32x32px (Tauri tray icon standard on Windows).

### `tray_controller.rs`
Add `update_tray_icon(app: &AppHandle, ratio: f32, state: &DeskState)`:
```rust
pub fn update_tray_icon(app: &AppHandle, ratio: f32, state: &DeskState) {
    let icon_name = match state {
        DeskState::Sitting if ratio >= 0.85 => "tray-alert",
        DeskState::Sitting if ratio >= 0.60 => "tray-warn",
        DeskState::Sitting => "tray-ok",
        _ => "tray-idle",
    };
    // load icon bytes, call tray.set_icon()
}
```

### `session.rs` / serial reader
Call `update_tray_icon()` on every `desk:state-changed` emission and on every
`tick()` when state is Sitting (so the icon transitions smoothly at 60%/85% thresholds).

## Tests
- Unit: `update_tray_icon` selects correct icon name for each ratio/state combination
- Unit: non-Sitting state always returns "tray-idle"

## Acceptance criteria
- [ ] Tray icon turns amber when sitting > 60% of limit
- [ ] Tray icon turns red when sitting > 85% of limit
- [ ] Tray icon returns to idle (neutral) when standing/away
- [ ] No crash if icon file missing (log error, keep previous icon)
