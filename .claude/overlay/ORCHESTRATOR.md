# Overlay Progress Bar — Orchestration Plan

**For:** Orchestrator agent coordinating parallel work via worktrees
**Date:** 2026-03-19
**Status:** Ready to execute

---

## Pre-Flight Checklist

Before dispatching agents:
1. Ensure `main` branch is clean: `git status`
2. Ensure code compiles: `cd src-tauri && cargo check`
3. Ensure tests pass: `cd src-tauri && cargo test --lib`

---

## Wave Structure

```
WAVE 1 (parallel — 2 agents on worktrees)
├── T-OVR-001: Dev mode visible bar     [overlay_renderer.rs]
└── T-OVR-002: Device notification      [serial.rs, lib.rs]
         │
         ▼ merge both to main
WAVE 2 (parallel — 3 agents on worktrees)
├── T-OVR-003: Compare modes visually   [manual + docs]
├── T-OVR-005: Adjustable bar height    [overlay_renderer.rs]
└── T-OVR-006: Fix auto-test            [auto-test.sh]
         │
         ▼ merge all to main
WAVE 3 (parallel — 2 agents on worktrees)
├── T-OVR-004: Bar variants             [overlay_renderer.rs]
└── T-OVR-007: Rust visual regression   [overlay_renderer.rs tests]
         │
         ▼ merge to main
WAVE 4 (sequential — 1 agent)
├── T-OVR-008: Wire to session data     [tray_controller.rs]
└── T-OVR-009: Choose production mode   [decision + docs]
```

---

## Worktree Setup Commands

Run these from project root before dispatching Wave 1:

```bash
# Wave 1
git branch feat/T-OVR-001-dev-mode main
git worktree add .claude/worktrees/T-OVR-001-dev-mode feat/T-OVR-001-dev-mode

git branch feat/T-OVR-002-device-notif main
git worktree add .claude/worktrees/T-OVR-002-device-notif feat/T-OVR-002-device-notif
```

After Wave 1 merge:
```bash
# Wave 2
git branch feat/T-OVR-003-compare-modes main
git worktree add .claude/worktrees/T-OVR-003-compare-modes feat/T-OVR-003-compare-modes

git branch feat/T-OVR-005-bar-height main
git worktree add .claude/worktrees/T-OVR-005-bar-height feat/T-OVR-005-bar-height

git branch feat/T-OVR-006-fix-autotest main
git worktree add .claude/worktrees/T-OVR-006-fix-autotest feat/T-OVR-006-fix-autotest
```

### Merge Protocol

After each wave completes:
```bash
git checkout main
git merge feat/T-OVR-001-dev-mode --no-ff -m "feat(overlay): dev mode — always-visible progress bar"
git merge feat/T-OVR-002-device-notif --no-ff -m "feat(desk): device disconnected notification"
# Resolve conflicts if any (unlikely — different files)

# Cleanup
git worktree remove .claude/worktrees/T-OVR-001-dev-mode
git branch -d feat/T-OVR-001-dev-mode
git worktree remove .claude/worktrees/T-OVR-002-device-notif
git branch -d feat/T-OVR-002-device-notif
```

---

## WAVE 1 — Task Dispatches

### Agent 1: T-OVR-001 — Dev Mode Visible Bar

**Worktree:** `.claude/worktrees/T-OVR-001-dev-mode`
**Branch:** `feat/T-OVR-001-dev-mode`
**Files to modify:** `src-tauri/src/overlay_renderer.rs`
**Estimated commits:** 3

**Full task spec follows — copy everything below to agent prompt:**

---

#### T-OVR-001: Dev Mode — Always-Visible Progress Bar

**CONTEXT — READ FIRST:**
You are modifying an overlay progress bar that renders a 4px-tall native Windows window at the top of the screen. The file `src-tauri/src/overlay_renderer.rs` contains TWO render backends:

1. **OPAQUE mode** (default): Uses GDI `BeginPaint`/`FillRect`. Black background. Renders via `WM_PAINT`.
2. **LAYERED mode** (experimental): Uses `UpdateLayeredWindow` with 32-bit ARGB DIBSection. Transparent background. Renders via `draw_layered_frame()`.

The mode is selected by env var `OVERLAY_MODE` (default: `opaque`).

**THE PROBLEM:**
The progress bar is currently **invisible** during development because:
- Bar rendering is gated by `if s.visible { ... }` (line ~321)
- `visible` is set to `true` ONLY when desk sensor (XIAO ESP32-C3 on COM3) sends `DeskState::Sitting` events via `tray_controller.rs`
- Without sensor → no events → `visible` stays `false` → bar never renders
- The ONLY visible element is test code (lines ~303-319) that fills entire window with cycling colors unconditionally

**WHAT YOU MUST DO:**
Implement "dev mode" that makes the bar visible without requiring a desk sensor.

**⚠️ CRITICAL RULES (from previous failed attempts):**
1. **DO NOT remove the test code (lines 303-319) until your new code is proven visible.** A previous agent removed it and the bar became completely invisible. Modify in-place instead.
2. **Test OPAQUE mode** (run `pnpm tauri:dev` without env vars). Do NOT test only LAYERED mode — they are separate code paths. A previous agent tested LAYERED and claimed success while OPAQUE was broken.
3. **One change at a time.** Compile and verify after each change. Do not batch 5 changes.

**IMPLEMENTATION (3 commits):**

**Commit 1: `refactor(overlay): rename demo_mode to dev_mode, auto-detect debug builds`**

In `OverlayState` struct (~line 14):
```rust
// BEFORE:
pub demo_mode: bool,

// AFTER:
pub dev_mode: bool,  // Auto-enabled in debug builds
```

In `Default` impl (~line 23):
```rust
// BEFORE:
let demo_mode = std::env::var("OVERLAY_DEMO_MODE")
    .map(|v| v.to_lowercase() == "true")
    .unwrap_or(false);

// AFTER:
let dev_mode = if cfg!(debug_assertions) {
    // Auto-enable in debug builds unless explicitly disabled
    std::env::var("OVERLAY_DEV_MODE")
        .map(|v| v.to_lowercase() != "false")
        .unwrap_or(true)  // true by default in debug
} else {
    std::env::var("OVERLAY_DEV_MODE")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false)  // false by default in release
};
```

In `update()`, `show()`, `hide()` methods (~lines 60-80):
```rust
pub fn update(&self, progress: f32, color_rgb: (u8, u8, u8)) {
    if let Ok(mut s) = self.state.lock() {
        if s.dev_mode { return; }  // Dev mode ignores production updates
        s.progress = progress.clamp(0.0, 1.0);
        s.color_rgb = color_rgb;
        s.needs_redraw = true;
    }
}

pub fn show(&self) {
    if let Ok(mut s) = self.state.lock() {
        if s.dev_mode { return; }
        s.visible = true;
        s.needs_redraw = true;
    }
}

pub fn hide(&self) {
    if let Ok(mut s) = self.state.lock() {
        if s.dev_mode { return; }
        s.visible = false;
        s.needs_redraw = true;
    }
}
```

Replace all occurrences of `demo_mode` → `dev_mode` throughout the file.
Replace all occurrences of `OVERLAY_DEMO_MODE` → `OVERLAY_DEV_MODE` throughout the file.

Add helper function (after `impl OverlayRenderer` block):
```rust
/// Calculates dev mode progress and color from frame count.
/// Cycles: 0% → 25% → 50% → 75% → 100% every 5 seconds (300 frames @ 60fps).
fn dev_mode_progress(frame_count: u32) -> (f32, (u8, u8, u8)) {
    use crate::colors::color_for_progress;
    let stage = ((frame_count / 300) % 5) as u32;
    let progress = stage as f32 / 4.0;
    let (r, g, b, _) = color_for_progress(progress);
    (progress, (r, g, b))
}
```

Verify: `cd src-tauri && cargo check && cargo test overlay_renderer --lib`

**Commit 2: `feat(overlay): implement dev mode rendering in OPAQUE mode`**

In OPAQUE `wnd_proc()` → `WM_TIMER` handler (~line 253):
```rust
// Replace the existing demo_mode block with:
if s.dev_mode {
    let (progress, color) = dev_mode_progress(s.frame_count);
    let prev_stage = ((s.frame_count.wrapping_sub(1) / 300) % 5) as u32;
    let curr_stage = ((s.frame_count / 300) % 5) as u32;
    if prev_stage != curr_stage {
        log::info!("[DEV] Stage {}: progress={:.0}%", curr_stage, progress * 100.0);
    }
    s.progress = progress;
    s.color_rgb = color;
    s.visible = true;
}
```

In OPAQUE `wnd_proc()` → `WM_PAINT` handler (~lines 296-349):

**Modify the test code IN PLACE.** Do NOT remove lines 303-319. Instead, replace the entire rendering block (lines 296-349) with:

```rust
// Fill background with black
let black_brush = CreateSolidBrush(COLORREF(0));
if !black_brush.is_invalid() {
    let _ = FillRect(hdc, &rect, black_brush);
    let _ = DeleteObject(black_brush.into());
}

// Draw progress bar
if s.dev_mode || s.visible {
    // Calculate bar width (progress × window_width, minimum 1px in dev mode)
    let bar_width = ((window_width as f32) * s.progress.clamp(0.0, 1.0)) as i32;
    let bar_width = if s.dev_mode { bar_width.max(1) } else { bar_width };

    let color = COLORREF(
        (s.color_rgb.0 as u32)
            | ((s.color_rgb.1 as u32) << 8)
            | ((s.color_rgb.2 as u32) << 16),
    );
    let brush = CreateSolidBrush(color);
    if bar_width > 0 && !brush.is_invalid() {
        let bar_rect = RECT {
            left: 0,
            top: 0,
            right: bar_width,
            bottom: window_height,
        };
        let _ = FillRect(hdc, &bar_rect, brush);
    }
    if !brush.is_invalid() {
        let _ = DeleteObject(brush.into());
    }
}
```

Also fix OPAQUE positioning — in `run_event_loop_opaque()` (~line 120-128):
```rust
// BEFORE:
let screen_width = if GetMonitorInfoW(hmonitor, &mut monitor_info).as_bool() {
    monitor_info.rcMonitor.right - monitor_info.rcMonitor.left
} else {
    1920 // fallback
};
let screen_height = 4i32;

// AFTER:
let (screen_width, screen_height, screen_x, screen_y) = if GetMonitorInfoW(hmonitor, &mut monitor_info).as_bool() {
    (
        monitor_info.rcMonitor.right - monitor_info.rcMonitor.left,
        4i32,
        monitor_info.rcMonitor.left,
        monitor_info.rcMonitor.top,
    )
} else {
    (1920, 4, 0, 0)
};
```

And update `CreateWindowExA` call to use `screen_x, screen_y` instead of `0, 0`.

Remove excessive `log::info!("WM_PAINT: ...")` lines (keep only stage-change logging).

Verify: `cd src-tauri && cargo check && cargo test overlay_renderer --lib`
Verify VISUALLY: Run `pnpm tauri:dev`. You MUST see a colored bar at top of screen that changes width every 5 seconds. If you see nothing, DO NOT commit — debug first.

**Commit 3: `feat(overlay): implement dev mode rendering in LAYERED mode`**

In LAYERED `wnd_proc_layered()` → `WM_TIMER` handler (~line 700):
Apply same dev_mode logic as OPAQUE WM_TIMER (use `dev_mode_progress()`).

In `draw_layered_frame()` (~line 590):
Replace the demo_mode override with dev_mode, using `dev_mode_progress()`.
Keep the minimum 1px bar width.
Log on stage change only (not every frame).
Remove all `log::warn!("🟢 [DRAW]...")` and `log::warn!("🔴 [DRAW]...")` spam.

Verify: `cd src-tauri && cargo check && cargo test overlay_renderer --lib`
Verify VISUALLY: Run `OVERLAY_MODE=layered pnpm tauri:dev`. Semi-transparent bar visible, changes width.

**ACCEPTANCE CRITERIA:**
- [ ] `pnpm tauri:dev` → colored bar visible at top of screen (OPAQUE)
- [ ] Bar width changes every 5 seconds: thin → 25% → 50% → 75% → full
- [ ] Bar color changes: green → green → yellow → yellow → red (matching progress)
- [ ] No rapid flickering (changes every 5 seconds, not faster)
- [ ] `OVERLAY_MODE=layered pnpm tauri:dev` → semi-transparent bar visible
- [ ] `cargo test overlay_renderer --lib` → all tests pass
- [ ] `OVERLAY_DEV_MODE=false pnpm tauri:dev` → bar NOT visible (dev mode disabled)

**WHAT TO DO IF BAR IS NOT VISIBLE:**
1. Check logs: are there `[DEV] Stage N:` messages? If no → dev_mode not enabled
2. Check if you accidentally removed the rendering code instead of modifying it
3. Add `log::warn!` inside the `if s.dev_mode || s.visible` block to confirm it executes
4. DO NOT commit invisible code. Document the failure and ask for help.

---

### Agent 2: T-OVR-002 — Device Disconnected Notification

**Worktree:** `.claude/worktrees/T-OVR-002-device-notif`
**Branch:** `feat/T-OVR-002-device-notif`
**Files to modify:** `src-tauri/src/serial.rs`, `src-tauri/src/lib.rs`
**Estimated commits:** 2

**Full task spec follows — copy everything below to agent prompt:**

---

#### T-OVR-002: Device Disconnected Notification

**CONTEXT:**
The desk app tracks a sit/stand desk via VL53L1X laser sensor on XIAO ESP32-C3 (COM3). The sensor auto-detects via `DEVICE: zntl-desk-sensor v1` string. See `src-tauri/src/serial.rs` for the serial reader.

The app currently fails silently when no sensor is connected. The user has no idea the sensor is missing.

**WHAT YOU MUST DO:**
Show a native Windows notification when the sensor is not connected.

**FILES:**
- `src-tauri/src/serial.rs` — serial port reader (find `scan_ports` or `connect` logic)
- `src-tauri/src/lib.rs` — Tauri app setup (where plugins are registered)
- `src-tauri/Cargo.toml` — verify `tauri-plugin-notification` is in dependencies

**IMPLEMENTATION:**

**Commit 1: `feat(desk): detect sensor absence and emit event`**

In `serial.rs`, find where the serial port scanning/connection happens.
When no sensor found after scanning all ports:
```rust
// Emit event for missing sensor
if let Some(app) = app_handle {
    let _ = app.emit("desk:device-missing", ());
}
```

When sensor disconnects during session (connection lost):
```rust
let _ = app.emit("desk:device-lost", ());
```

**Commit 2: `feat(desk): show native notification for missing sensor`**

In `lib.rs` setup, listen for these events:
```rust
use tauri::Listener;
use tauri_plugin_notification::NotificationExt;

app.listen("desk:device-missing", move |_| {
    let _ = app_handle.notification()
        .builder()
        .title("zntlDesk")
        .body("Sensor not connected. Plug in desk sensor to COM3.")
        .show();
});
```

Add throttling: don't spam notifications. Show once, then remind every 5 minutes max.

**ACCEPTANCE CRITERIA:**
- [ ] Start app without sensor → notification appears within 10 seconds
- [ ] Unplug sensor during session → notification appears
- [ ] Notification is native Windows toast
- [ ] No spam: max 1 notification per 5 minutes
- [ ] `cargo test --lib` passes

---

## WAVE 2 — Task Dispatches

### Agent 3: T-OVR-005 — Adjustable Bar Height

**Worktree:** `.claude/worktrees/T-OVR-005-bar-height`
**Branch:** `feat/T-OVR-005-bar-height`
**Depends on:** Wave 1 merged (T-OVR-001)

**Commit:** `feat(overlay): configurable bar height via OVERLAY_HEIGHT env var`

Read `OVERLAY_HEIGHT` env var (default 4, range 1-20). Use in both OPAQUE and LAYERED window creation where `screen_height = 4i32` is set. Clamp to range.

### Agent 4: T-OVR-006 — Fix Auto-Test

**Worktree:** `.claude/worktrees/T-OVR-006-fix-autotest`
**Branch:** `feat/T-OVR-006-fix-autotest`
**Depends on:** Wave 1 merged (T-OVR-001)

**Commit:** `fix(testing): auto-test.sh tests OPAQUE mode by default`

Modify `.claude/overlay/test-infrastructure/auto-test.sh`:
- Remove `OVERLAY_MODE=layered` (test default OPAQUE mode)
- Add parameter: `bash auto-test.sh [opaque|layered]`
- Parse `WM_PAINT` log lines (not just `[DRAW]`)
- Fix `pkill` → use Windows-compatible process kill
- Verify `bar_width` changes over time in OPAQUE logs

### Agent 5: T-OVR-003 — Compare Modes

**Worktree:** `.claude/worktrees/T-OVR-003-compare-modes`
**Branch:** `feat/T-OVR-003-compare-modes`
**Depends on:** Wave 1 merged (T-OVR-001)

**Commit:** `docs(overlay): visual comparison of OPAQUE vs LAYERED modes`

Run both modes, document differences in `.claude/overlay/MODE-COMPARISON.md`:
- OPAQUE: what it looks like, pros/cons
- LAYERED: what it looks like, pros/cons
- Recommendation for production

---

## WAVE 3 — Task Dispatches

### Agent 6: T-OVR-007 — Rust Visual Regression

**Worktree:** `.claude/worktrees/T-OVR-007-visual-test`
**Branch:** `feat/T-OVR-007-visual-test`
**Depends on:** Wave 2 merged

**Commit:** `feat(testing): Rust-native screenshot test for overlay bar`

Add `#[test]` in `overlay_renderer.rs` that:
1. Creates overlay window in dev mode
2. Advances frame_count through stages (0, 300, 600, 900, 1200)
3. Captures screen region via WinAPI `BitBlt` / `PrintWindow`
4. Asserts pixel colors in captured region match expected bar width
5. Saves debug PNGs to `.claude/overlay/test-runs/`

**NOTE:** Playwright screenshot tests (`tests/overlay-screenshots.test.ts`) do NOT work for this — they capture the web view, not the native WinAPI overlay. Delete that file and replace with this Rust test.

### Agent 7: T-OVR-004 — Bar Variants

**Worktree:** `.claude/worktrees/T-OVR-004-variants`
**Branch:** `feat/T-OVR-004-variants`
**Depends on:** Wave 2 merged

**Commits:**
1. `feat(overlay): add OVERLAY_VARIANT env var for bar styles`
2. `feat(overlay): implement gradient and pulsing bar variants`

Add `overlay_variant: u8` to OverlayState. Read from `OVERLAY_VARIANT` env var.
In rendering: match variant → different fill logic.

---

## WAVE 4 — Sequential

### T-OVR-008 + T-OVR-009

These are decisions, not code. After Waves 1-3:
1. Developer tests all variants visually
2. Chooses production mode (OPAQUE vs LAYERED) and variant
3. Wires overlay to real session data in tray_controller.rs
4. Removes dev mode test code if no longer needed

---

## Merge Conflict Risk Matrix

| Wave | Agent A | Agent B | Conflict Risk | Notes |
|------|---------|---------|---------------|-------|
| 1 | T-OVR-001 (overlay_renderer.rs) | T-OVR-002 (serial.rs, lib.rs) | **NONE** | Different files |
| 2 | T-OVR-005 (overlay_renderer.rs) | T-OVR-006 (auto-test.sh) | **NONE** | Different files |
| 2 | T-OVR-005 (overlay_renderer.rs) | T-OVR-003 (docs only) | **NONE** | Different files |
| 3 | T-OVR-007 (overlay_renderer.rs tests) | T-OVR-004 (overlay_renderer.rs) | **LOW** | Both touch same file but different sections |

---

## Quick Reference for Orchestrator

```bash
# WAVE 1: Create worktrees
git branch feat/T-OVR-001-dev-mode main
git worktree add .claude/worktrees/T-OVR-001-dev-mode feat/T-OVR-001-dev-mode
git branch feat/T-OVR-002-device-notif main
git worktree add .claude/worktrees/T-OVR-002-device-notif feat/T-OVR-002-device-notif

# Dispatch agents to worktrees (copy task specs above)

# WAVE 1: Merge
git checkout main
git merge feat/T-OVR-001-dev-mode --no-ff
git merge feat/T-OVR-002-device-notif --no-ff
git worktree remove .claude/worktrees/T-OVR-001-dev-mode
git worktree remove .claude/worktrees/T-OVR-002-device-notif
git branch -d feat/T-OVR-001-dev-mode feat/T-OVR-002-device-notif

# WAVE 2: Repeat pattern...
```

---

## File Index (for agents)

| File | Purpose | Who touches |
|------|---------|-------------|
| `src-tauri/src/overlay_renderer.rs` | Main overlay code | T-OVR-001, 004, 005, 007 |
| `src-tauri/src/tray_controller.rs` | Wires events → overlay | T-OVR-008 |
| `src-tauri/src/serial.rs` | Serial port reader | T-OVR-002 |
| `src-tauri/src/lib.rs` | Tauri setup | T-OVR-002 |
| `src-tauri/src/colors.rs` | Progress → color mapping | Read-only reference |
| `.claude/overlay/test-infrastructure/auto-test.sh` | Test harness | T-OVR-006 |
| `tests/overlay-screenshots.test.ts` | DELETE (doesn't work) | T-OVR-007 |
