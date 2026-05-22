---
id: E004-T02
epic: E004
status: completed
created: 2026-03-20
completed: 2026-03-20
original_id: T013+T014
title: AlertManager + Stage1 (bar pulse) + Stage2 (popup)
---
# E004-T02: AlertManager + Stage1 (bar pulse) + Stage2 (popup)

**CEO review:** Ship together — bar pulse alone too subtle.
**Depends on:** T-OVR-010 (split overlay_renderer.rs)

## New files
- `alert_manager.rs` (~150 lines) — `AlertStage` enum, `AlertAction` enum, `AlertManager` struct
- `alert_popup.rs` (~200 lines) — WinAPI popup window, own thread

## Modified files
- `overlay_renderer.rs` — add `set_variant(&self, variant: u8)` method
- `tray_controller.rs` — wire AlertManager tick + execute AlertActions
- `lib.rs` + `commands.rs` — add AlertManager + AlertPopup to AppState

## Eng review decisions (2026-03-20)
- Bar pulse: `set_variant(2)` method on OverlayRenderer (explicit, same pattern as update/show/hide)
- Popup thread: own thread, spawn on show, `AtomicBool` dismiss flag, join on dismiss
- Time source: `Instant` (monotonic), `stage_entered_at` field
- Dismiss returns to Snoozed (T015 fills in full snooze logic; T013 implements stub: instant re-entry to Stage1)
- Safety: if progress drops below 1.0, also dismiss popup (missed event mitigation)

## AlertManager API
```rust
tick(progress: f32) -> Vec<AlertAction>
on_standing() -> Vec<AlertAction>   // reset + StopPulse + DismissPopup
dismiss()                          // -> Snoozed (stub in T013, filled by T015)
```

## AlertPopup API
```rust
show(msg: String)    // spawns WinAPI thread
dismiss()            // sets AtomicBool, thread exits
is_visible() -> bool
```

## Stage 1 (at limit)
`overlay.set_variant(2)` — bar starts pulsing

## Stage 2 (+2min)
Popup at right-bottom (notification area): ~400x200px, always-on-top, non-modal, arrow cursor.
Auto-dismiss on `DeskState::Standing` OR `progress < 1.0`.

## Tests (~15)
- AlertManager: idle->stage1->stage2 transitions, timing, on_standing reset
- dismiss() returns to Idle, progress oscillation debounce, rapid sit/stand
- set_variant() updates state, popup show/dismiss lifecycle
