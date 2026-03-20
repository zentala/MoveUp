# Session Alerts — Orchestration Plan

**Date:** 2026-03-20
**Status:** Ready to execute
**Scope:** T-OVR-010 → T013+T014 (bundled) → T015

---

## Execution Order

```
PHASE 1: T-OVR-010 — Split overlay_renderer.rs (P0 blocker)
│   Single agent, worktree: feat/T-OVR-010
│   Pure refactor — zero behavior changes.
│   Must complete before T013 (AlertManager needs set_variant() on clean module).
│
▼ gate: cargo test --lib (101 tests pass), all 5 files < 250 lines
│
PHASE 2: T013+T014 BUNDLED — AlertManager + Stage1 (pulse) + Stage2 (popup)
│   Single agent, worktree: feat/T013-T014
│   CEO decision: ship together — bar pulse alone too subtle.
│   Creates: alert_manager.rs, alert_popup.rs
│   Modifies: overlay_renderer.rs, tray_controller.rs, lib.rs, commands.rs
│
▼ gate: cargo test --lib passes, bar pulses at progress≥1.0, popup shows at +2min,
│        auto-dismiss on stand, dismiss→Snoozed (stub: instant re-entry to Stage1)
│
PHASE 3: T015 — Snooze (depends on T013)
│   Single agent, worktree: feat/T015
│   Extends alert_manager.rs only.
│
▼ gate: dismiss→snooze→re-escalate, deescalating intervals, tone shift at #3
```

---

## Conflict Risk Matrix

| Phase | Files touched | Risk |
|-------|---------------|------|
| 1 | overlay_renderer.rs + 4 new files | NONE (single agent) |
| 2 | alert_manager.rs (new), alert_popup.rs (new), tray_controller.rs, lib.rs, commands.rs | NONE (after Phase 1) |
| 3 | alert_manager.rs only | NONE (sequential) |

---

## Pre-Flight Checklist

Before starting Phase 1:
```bash
cd apps/desk/src-tauri
cargo check                # must compile clean
cargo test --lib           # must pass 101 tests
wc -l src/overlay_renderer.rs   # confirm > 250 lines
```

---

## Phase 1: T-OVR-010 — Split overlay_renderer.rs

**Why first:** 1132 lines, 4.5× over limit. Can't add alert_manager integration to a 1000+ line file.

**Worktree:** `feat/T-OVR-010`

### Target split

```
overlay_renderer.rs  (~100 lines) — DataSource enum, OverlayState, OverlayRenderer API,
                                    run_event_loop() dispatcher
overlay_opaque.rs    (~150 lines) — run_event_loop_opaque(), wnd_proc() (OPAQUE backend)
overlay_layered.rs   (~230 lines) — run_event_loop_layered(), wnd_proc_layered(),
                                    draw_layered_frame(), LayeredBufferState
overlay_variants.rs  ( ~80 lines) — render_variant_gdi() + render_variant_pixels()
                                    (shared by both backends — DRY)
overlay_tests.rs     (~200 lines) — all #[cfg(test)] mod tests (moved from overlay_renderer.rs)
```

### Steps

1. Create `overlay_variants.rs` — extract variant match blocks from WM_PAINT + draw_layered_frame
2. Create `overlay_opaque.rs` — move `run_event_loop_opaque()` + `wnd_proc()`
3. Create `overlay_layered.rs` — move layered code + `LayeredBufferState`
4. Create `overlay_tests.rs` — move `#[cfg(test)] mod tests`
5. Slim `overlay_renderer.rs` to public API + dispatcher
6. Add `pub(crate) mod` declarations in `overlay_renderer.rs` or `lib.rs`
7. Verify all files ≤ 250 lines

### Key decisions

- Variants: shared module, `pub(crate) fn render_variant_gdi()` + `render_variant_pixels()`
- State: passed as `Arc<Mutex<OverlayState>>` parameter (no globals)
- Visibility: `pub(crate)` on all moved functions (not `pub`)
- Tests: separate `overlay_tests.rs` (keeps main module clean)

### Acceptance criteria

- [ ] All 5 files ≤ 250 lines
- [ ] `cargo test --lib` passes (still 101 tests)
- [ ] `cargo check` — 0 warnings
- [ ] Zero behavior change — pure refactor

**Estimated commits:** 3 (variants extract, backends extract, tests+cleanup)

---

## Phase 2: T013+T014 BUNDLED — AlertManager + Stage 1 + Stage 2 Popup

**Why bundled:** CEO decision — bar pulse alone too subtle, users need visual + popup together.

**Worktree:** `feat/T013-T014`

### Files

| Action | File | What |
|--------|------|------|
| CREATE | `src-tauri/src/alert_manager.rs` (~150 lines) | AlertStage, AlertAction, AlertManager, AlertConfig |
| CREATE | `src-tauri/src/alert_popup.rs` (~200 lines) | WinAPI popup window, own thread |
| MODIFY | `src-tauri/src/overlay_renderer.rs` | add `set_variant(&self, variant: u8)` method |
| MODIFY | `src-tauri/src/tray_controller.rs` | wire AlertManager tick + execute AlertActions |
| MODIFY | `src-tauri/src/lib.rs` | add `mod alert_manager`, `mod alert_popup`, inject into AppState |
| MODIFY | `src-tauri/src/commands.rs` | add AlertManager + AlertPopup to AppState struct |

### AlertManager — full spec

```rust
pub enum AlertStage {
    Idle,
    Stage1,   // bar pulsing
    Stage2,   // bar pulsing + popup visible
    Stage3,   // future (T017)
    Stage4,   // future (T017)
    Stage5,   // future (T017)
    Snoozed,  // dismissed, waiting for cooldown
}

pub enum AlertAction {
    PulseBar,            // set overlay variant=2 (pulsing)
    StopPulse,           // set overlay variant=0 (solid)
    ShowPopup(String),   // spawn WinAPI popup thread with message
    DismissPopup,        // set AtomicBool, popup thread exits
    ExpandOverlay,       // future: increase bar height (T017)
    FullScreenNudge,     // future: full overlay (T017)
}

pub struct AlertConfig {
    pub stage1_threshold: f32,        // default: 1.0 (100%)
    pub stage2_delay_secs: u64,       // default: 120 (2 min after Stage1)
    pub stage3_enabled: bool,         // default: false (T017)
    pub stage4_enabled: bool,         // default: false (T017)
    pub stage5_enabled: bool,         // default: false (T017)
    // T015 will add: snooze_durations, neutral_messages, positive_messages
}

pub struct AlertManager {
    stage: AlertStage,
    stage_entered_at: Instant,
    snoozed_until: Option<Instant>,   // None in T013 stub
    config: AlertConfig,
}

impl AlertManager {
    pub fn new(config: AlertConfig) -> Self { ... }

    /// Called on every desk:distance event (~1/s). Returns actions to execute.
    pub fn tick(&mut self, progress: f32) -> Vec<AlertAction> { ... }

    /// Called when DeskState → Standing. Resets everything.
    pub fn on_standing(&mut self) -> Vec<AlertAction> { ... }

    /// Called when user clicks Dismiss in popup.
    /// T013 stub: enters Snoozed with snoozed_until = None → tick() re-enters Stage1 immediately.
    /// T015 will fill in actual snooze durations.
    pub fn dismiss(&mut self) { ... }
}
```

### tick() state machine

```
Idle:
  progress >= config.stage1_threshold  →  Stage1, emit [PulseBar]
  otherwise                            →  nothing

Stage1:
  progress < config.stage1_threshold   →  Idle, emit [StopPulse]
  elapsed >= stage2_delay_secs         →  Stage2, emit [ShowPopup(default_msg)]
  otherwise                            →  nothing

Stage2:
  progress < config.stage1_threshold   →  Idle, emit [StopPulse, DismissPopup]
  (standing handled by on_standing)    →  nothing

Snoozed (T013 stub — snoozed_until always None):
  snoozed_until expired or None        →  Stage1, emit [PulseBar]
  progress < config.stage1_threshold   →  Idle, emit []     ← safety cancel
  otherwise                            →  nothing
```

### on_standing() behavior

```
ANY stage → Idle
Actions: [StopPulse, DismissPopup]
(T015 will also reset snooze_index)
```

### AlertPopup — full spec

```rust
pub struct AlertPopup {
    visible: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
}

impl AlertPopup {
    pub fn show(&mut self, msg: String) {
        // sets visible=true, spawns WinAPI thread
    }
    pub fn dismiss(&mut self) {
        // sets visible=false, joins thread
    }
    pub fn is_visible(&self) -> bool { ... }
}
```

**WinAPI popup window spec:**
- Size: ~400×200px
- Position: right-bottom corner, near system tray (taskbar notification area)
- Always-on-top (`HWND_TOPMOST`)
- Non-modal, arrow cursor (`IDC_ARROW`)
- Layout:
  ```
  ┌────────────────────────────────┐
  │  You've been sitting           │
  │  for 45 minutes.               │
  │  Take a 5-min break!           │
  │                                │
  │       [Dismiss]  [Stand up]    │
  └────────────────────────────────┘
         ↑ right-bottom, near tray
  ```
- `[Dismiss]` → calls `alert_manager.dismiss()`, hides popup
- `[Stand up]` → same as dismiss for now (T015+ can differentiate)
- Auto-dismiss: on `DeskState::Standing` OR `progress < 1.0`
- Thread lifecycle: spawn on `show()`, `AtomicBool` signals exit, join on `dismiss()`

### tray_controller.rs wiring

On every `desk:distance` event:
```rust
let actions = alert_manager.tick(progress);
for action in actions {
    match action {
        AlertAction::PulseBar     => overlay.set_variant(2),
        AlertAction::StopPulse    => overlay.set_variant(0),
        AlertAction::ShowPopup(m) => alert_popup.show(m),
        AlertAction::DismissPopup => alert_popup.dismiss(),
        _ => {}  // Stages 3-5 not implemented yet
    }
}
```

On `DeskState::Standing` event:
```rust
let actions = alert_manager.on_standing();
// execute same match above
```

### Tests (~15)

**AlertManager:**
1. `progress < 1.0` → stays Idle, no actions
2. `progress >= 1.0` → transitions to Stage1, emits PulseBar
3. Stage1 for < 2 min → stays Stage1, no new actions
4. Stage1 for >= 2 min → transitions to Stage2, emits ShowPopup
5. `on_standing()` from Stage1 → Idle, emits StopPulse
6. `on_standing()` from Stage2 → Idle, emits StopPulse + DismissPopup
7. `dismiss()` → Snoozed; subsequent tick() → Stage1 (stub: re-enter immediately if snoozed_until=None)
8. `progress < 1.0` while Stage2 → Idle, emits StopPulse + DismissPopup (safety)
9. Progress oscillation 0.99→1.01→0.99 — no flapping (confirm debounce or stateless behavior)
10. Rapid sit/stand/sit — ends in correct state

**overlay_renderer.rs:**
11. `set_variant(2)` — OverlayState.variant updated to 2
12. `set_variant(0)` — OverlayState.variant updated to 0

**AlertPopup:**
13. `show()` → is_visible() == true
14. `dismiss()` → is_visible() == false
15. Double-dismiss is safe (no panic)

**Estimated commits:** 3 (alert_manager.rs, alert_popup.rs, tray wiring)

---

## Phase 3: T015 — Snooze Logic

**Depends on:** T013 (AlertManager.dismiss() and Snoozed stage exist)

**Worktree:** `feat/T015`

### Files

| Action | File | What |
|--------|------|------|
| MODIFY | `src-tauri/src/alert_manager.rs` | add snooze fields, update dismiss()/tick(), add popup_message() |

**No other files change.**

### Fields to add to AlertManager

```rust
pub struct AlertManager {
    // existing:
    stage: AlertStage,
    stage_entered_at: Instant,
    snoozed_until: Option<Instant>,
    config: AlertConfig,

    // T015 new:
    snooze_index: usize,   // incremented each dismiss, reset on standing
}
```

### Fields to add to AlertConfig

```rust
pub struct AlertConfig {
    // existing fields...

    // T015 new:
    pub snooze_durations: Vec<Duration>,            // default: [5, 15, 30, 60] min
    pub neutral_messages: Vec<String>,               // default: ["Time for a stretch!", "Your body needs a break"]
    pub positive_messages: Vec<String>,              // default: ["Even 2 min standing helps blood flow", "Quick stand = fresh mind"]
    pub tone_shift_threshold: usize,                 // default: 3 (dismiss #3+ → positive)
}
```

### Updated dismiss() behavior

```rust
pub fn dismiss(&mut self) {
    let idx = self.snooze_index.min(self.config.snooze_durations.len() - 1);
    let duration = self.config.snooze_durations[idx];
    self.snoozed_until = Some(Instant::now() + duration);
    self.snooze_index += 1;
    self.stage = AlertStage::Snoozed;
    // bar variant: tray_controller handles via StopPulse + red color (variant=0, color stays red)
}
```

### Updated tick() — Snoozed branch

```rust
Snoozed:
  progress < config.stage1_threshold          →  Idle, emit []         ← problem resolved
  snoozed_until expired                        →  Stage1, emit [PulseBar]
  otherwise                                    →  nothing
```

### Updated on_standing() — reset snooze_index

```rust
pub fn on_standing(&mut self) -> Vec<AlertAction> {
    self.stage = AlertStage::Idle;
    self.snooze_index = 0;
    self.snoozed_until = None;
    vec![AlertAction::StopPulse, AlertAction::DismissPopup]
}
```

### Message selection helper

```rust
fn popup_message(&self) -> &str {
    let messages = if self.snooze_index >= self.config.tone_shift_threshold {
        &self.config.positive_messages
    } else {
        &self.config.neutral_messages
    };
    let idx = self.snooze_index % messages.len().max(1);
    &messages[idx]
}
```

Called in tick() when transitioning to Stage2.

### Bar variant behavior during snooze

| State | Bar variant | Color |
|-------|-------------|-------|
| Stage1 | 2 (pulsing) | red |
| Stage2 | 2 (pulsing) | red |
| Snoozed | 0 (solid) | red |
| Snooze expired → Stage1 | 2 (pulsing) | red |
| Standing → Idle | hidden | — |

tray_controller handles: on Snoozed entry → emit `StopPulse` (variant=0), color stays red.
Note: `StopPulse` already emitted by `dismiss()` return value — add `StopPulse` to dismiss() actions.

Actually: `dismiss()` currently returns void. Either:
- Add return `Vec<AlertAction>` to dismiss(), OR
- Have tick() detect transition from Stage2 → Snoozed and emit StopPulse

**Decision:** Add `StopPulse` to `dismiss()` by making it return `Vec<AlertAction>`.

```rust
pub fn dismiss(&mut self) -> Vec<AlertAction> {
    let idx = self.snooze_index.min(self.config.snooze_durations.len() - 1);
    self.snoozed_until = Some(Instant::now() + self.config.snooze_durations[idx]);
    self.snooze_index += 1;
    self.stage = AlertStage::Snoozed;
    vec![AlertAction::StopPulse, AlertAction::DismissPopup]
}
```

(T013 stub also returns `vec![AlertAction::StopPulse, AlertAction::DismissPopup]` without setting snoozed_until.)

### State diagram

```
snooze_index=0
     │
     ▼
IDLE ──(≥1.0)──→ STAGE1 ──(+2min)──→ STAGE2 ──dismiss──→ SNOOZED (5min)
                                                              │
              snooze_index=1          ◄──────(expired)────────┘
STAGE2 (neutral msg) ──dismiss──→ SNOOZED (15min)
              snooze_index=2          ◄──────(expired)────────┘
STAGE2 (neutral msg) ──dismiss──→ SNOOZED (30min)
              snooze_index=3          ◄──────(expired)────────┘
STAGE2 (POSITIVE msg) ──dismiss──→ SNOOZED (60min, capped)
              snooze_index=4+         ◄──────(expired)────────┘

ANY ──standing──→ IDLE (snooze_index=0)
ANY ──(progress<1.0)──→ IDLE (snooze cancelled — problem resolved)
```

### Tests (~8)

1. Dismiss #1 → snoozed_until = now+5min, snooze_index=1, neutral message next popup
2. Dismiss #2 → snoozed_until = now+15min, snooze_index=2, neutral message
3. Dismiss #3 → snoozed_until = now+30min, snooze_index=3, **positive** message next popup
4. Dismiss #4 → snoozed_until = now+60min (capped, snooze_index=4)
5. Standing while Snoozed → Idle, snooze_index=0, snoozed_until=None
6. `progress < 1.0` while Snoozed → Idle (cancel snooze), no StopPulse needed (bar hidden on Idle)
7. Bar variant: dismiss() returns [StopPulse, DismissPopup]; snooze expiry → Stage1 emits PulseBar
8. Snooze expiry → Stage1 (not Stage2 — user sees pulse first, then popup after +2min)

**Estimated commits:** 2 (snooze fields + dismiss update, tests)

---

## Summary

| Phase | Task | New files | Modified files | Tests |
|-------|------|-----------|----------------|-------|
| 1 | T-OVR-010 | overlay_variants.rs, overlay_opaque.rs, overlay_layered.rs, overlay_tests.rs | overlay_renderer.rs | 101 (unchanged) |
| 2 | T013+T014 | alert_manager.rs, alert_popup.rs | overlay_renderer.rs, tray_controller.rs, lib.rs, commands.rs | +15 |
| 3 | T015 | — | alert_manager.rs | +8 |
