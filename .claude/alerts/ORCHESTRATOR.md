# Session Alerts — Orchestration Plan

**Date:** 2026-03-20
**Status:** Ready to execute
**Scope:** T-OVR-010 (split) → T013 (AlertManager) → T014 (popup)

---

## Execution Order

```
PHASE 1: T-OVR-010 — Split overlay_renderer.rs (P0 blocker)
│   Single agent, main branch. Pure refactor.
│   Must complete before T013 — AlertManager needs clean module structure.
│
▼ verify: cargo test --lib (101 tests), all files < 250 lines
│
PHASE 2: T013+T014 BUNDLED — AlertManager + Stage 1 + Stage 2 popup
│   ├── Agent A: alert_manager.rs + tray_controller wiring + bar pulse
│   └── Agent B (optional parallel): T016 tray icon color dot (independent)
│   CEO decision: bundle T013+T014 — bar pulse alone too subtle.
│
▼ verify: cargo test --lib, bar pulses at 100%, popup at +2min, auto-dismiss on stand
│
PHASE 4: T015 — Snooze logic (depends on T013)
│   Single agent. Extends AlertManager.
│
▼ verify: dismiss → snooze → re-escalate
```

---

## Phase 1: T-OVR-010 — Split overlay_renderer.rs

**Why first:** 1132 lines, 4.5x over limit. Can't add alert_manager integration to a 1000+ line file.

**Agent setup:** Single agent, direct on main (no worktree needed — single task).

**Steps:**
1. Create `overlay_variants.rs` — extract variant match blocks
2. Create `overlay_opaque.rs` — move OPAQUE backend
3. Create `overlay_layered.rs` — move LAYERED backend
4. Create `overlay_tests.rs` — move test module
5. Slim `overlay_renderer.rs` to API + dispatcher
6. Add `mod` declarations
7. Verify: `cargo test --lib` (101), `cargo check` (0 warnings), all files < 250 lines

**Key decisions (from eng review):**
- Variants: shared module with `render_variant_gdi()` + `render_variant_pixels()`
- State: passed as `Arc<Mutex<OverlayState>>` parameter
- Tests: separate `overlay_tests.rs`
- Visibility: `pub(crate)` on moved functions

**Estimated commits:** 3 (variants, backends, tests+cleanup)

---

## Phase 2: T013 — AlertManager + Stage 1

**Why second:** Core alert logic. T014 popup and T015 snooze build on this.

**Agent setup:** Single agent on main (or worktree if T016 runs in parallel).

**Files to create/modify:**
- CREATE `src-tauri/src/alert_manager.rs` (~150 lines)
- MODIFY `src-tauri/src/tray_controller.rs` — wire AlertManager into `desk:distance` listener
- MODIFY `src-tauri/src/lib.rs` — add `mod alert_manager`, inject AlertManager into AppState
- MODIFY `src-tauri/src/commands.rs` — add AlertManager to AppState struct
- MODIFY `src-tauri/src/overlay_renderer.rs` — add `set_pulsing()` method or use existing variant=2

**AlertManager design:**
```rust
pub enum AlertStage { Idle, Stage1, Stage2, Stage3, Stage4, Stage5, Snoozed }

pub enum AlertAction {
    PulseBar,           // Stage 1: set overlay variant to pulsing
    StopPulse,          // Reset overlay variant
    ShowPopup(String),  // Stage 2+: show WinAPI popup with message
    DismissPopup,       // Auto-dismiss on standing
    ExpandOverlay,      // Stage 4: increase bar height
    FullScreenNudge,    // Stage 5: full overlay
}

pub struct AlertManager {
    stage: AlertStage,
    stage_entered_at: Instant,
    snoozed_until: Option<Instant>,
    config: AlertConfig,
}

impl AlertManager {
    pub fn tick(&mut self, progress: f32) -> Vec<AlertAction> { ... }
    pub fn on_standing(&mut self) -> Vec<AlertAction> { ... }
    pub fn dismiss(&mut self) { ... }
}
```

**Tests (~10):**
- Idle when progress < 1.0
- Stage 1 when progress >= 1.0
- Stage 1 → Stage 2 after 2 min elapsed
- on_standing() → Idle + StopPulse
- dismiss() → Snoozed
- Snoozed → Stage 1 after cooldown
- Progress oscillation (0.99 → 1.01 → 0.99) — no flapping
- Rapid sit/stand/sit — stable state
- Disabled stages skipped

**Estimated commits:** 3 (alert_manager.rs, wiring, tests)

---

## Phase 2b (optional parallel): T016 — Tray Icon Color Dot

**Independent of T013.** Can run in parallel worktree.

**Files:** `src-tauri/src/tray_icon.rs`, `src-tauri/src/tray.rs`
**Concept:** Overlay small colored circle on tray icon bitmap, synced with `color_for_progress()`.

---

## Phase 3: T014 — Stage 2 Popup

**Depends on:** T013 (AlertManager returns `ShowPopup` action)

**Files to create/modify:**
- CREATE `src-tauri/src/alert_popup.rs` — WinAPI popup window (clone overlay_renderer pattern)
- MODIFY `src-tauri/src/tray_controller.rs` — execute `ShowPopup` / `DismissPopup` actions

**Popup spec:**
- WinAPI window: ~300x100px, centered top of screen, always-on-top
- Text: "You've been sitting {min} min. Take a 5-min break!"
- Dismiss button (custom drawn or Windows button control)
- Auto-dismiss on `DeskState::Standing`
- Arrow cursor, non-modal

---

## Phase 4: T015 — Snooze Logic

**Depends on:** T013 (AlertManager.dismiss() exists)

**Files:** `src-tauri/src/alert_manager.rs` only (extend existing)
- Add `snoozed_until: Option<Instant>`
- Config: `snooze_duration_secs` in AppConfig
- tick() checks snooze expiry → restart from Stage 1

---

## Conflict Risk Matrix

| Phase | Agent A | Agent B | Conflict Risk |
|-------|---------|---------|---------------|
| 1 | T-OVR-010 (overlay split) | — | **NONE** (single agent) |
| 2 | T013 (alert_manager.rs) | T016 (tray_icon.rs) | **NONE** (different files) |
| 3 | T014 (alert_popup.rs) | — | **NONE** (new file) |
| 4 | T015 (alert_manager.rs) | — | **NONE** (extends existing) |

---

## Pre-Flight Checklist

Before starting Phase 1:
```bash
git status          # must be clean
cd src-tauri && cargo check    # must compile
cd src-tauri && cargo test --lib   # must pass 101 tests
wc -l src-tauri/src/overlay_renderer.rs   # confirm >250 lines
```
