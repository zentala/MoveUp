# Overlay Progress Bar — Development Tasks

**Status:** In Development
**Goal:** Widoczny, konfigurowalny progress bar na górze ekranu

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│  Top of screen (4px tall × full width)                         │
│                                                                 │
│  ┌──────────────────┐                                          │
│  │ Progress Bar 25% │ ← transparent rest (LAYERED)             │
│  └──────────────────┘   or black rest (OPAQUE)                 │
│                                                                 │
│  Controlled by:                                                 │
│    - overlay.update(progress, color)  ← sets width + color     │
│    - overlay.show() / hide()          ← visibility              │
│                                                                 │
│  Two render backends:                                           │
│    OPAQUE  — GDI FillRect, black background, simple             │
│    LAYERED — UpdateLayeredWindow, transparent, per-pixel alpha   │
└─────────────────────────────────────────────────────────────────┘
```

---

## P0 — Blocker: Bar Must Be Visible in Dev Mode

### [ ] T-OVR-001: Dev mode — always-visible progress bar
**Priority:** P0
**Why:** Developer cannot test/style overlay without seeing it. Currently invisible without desk sensor.

**⚠️ BEFORE STARTING: Read `.claude/overlay/KNOWLEDGE-BASE.md` — contains critical context about architecture, root causes, and pitfalls to avoid.**

**Root cause of current invisibility:**
The progress bar code in OPAQUE mode is gated by `if s.visible { ... }` (overlay_renderer.rs, WM_PAINT handler). `visible` is set to `true` ONLY when `tray_controller.rs` receives `desk:state-changed` event with `DeskState::Sitting`. Without a desk sensor connected to COM3, no events fire, `visible` stays `false`, progress bar never renders.

**What IS currently visible:**
Test code (overlay_renderer.rs, WM_PAINT, ~lines 303-319) runs UNCONDITIONALLY — fills entire window with cycling colors (Red, Maroon, Purple, Cream, Green) every 3 frames. This proves the rendering pipeline works. The test code is the ONLY visible element.

**Files to modify:**
- `src-tauri/src/overlay_renderer.rs` — main overlay code
  - `OverlayState` struct (~line 14) — add `dev_mode: bool` field
  - `wnd_proc()` → WM_PAINT handler (~line 275) — OPAQUE rendering
  - `draw_layered_frame()` (~line 576) — LAYERED rendering
  - `wnd_proc()` → WM_TIMER handler (~line 253) — frame counter + demo cycling
  - `wnd_proc_layered()` → WM_TIMER handler (~line 690) — layered frame counter

**Requirements:**
- When `OVERLAY_DEV_MODE=true`: bar renders unconditionally (no `visible` check)
- Bar shows even without desk sensor connected
- Progress cycles: 0% → 25% → 50% → 75% → 100% (every 5 seconds = 300 frames @ 60fps)
- Color cycles: green → yellow → red (synced with progress via `color_for_progress()` in `colors.rs`)
- Works in BOTH OPAQUE and LAYERED modes
- Default: `pnpm tauri:dev` should enable dev mode automatically

**Implementation approach (step by step):**

1. **Add `dev_mode` to OverlayState** (rename existing `demo_mode`):
   ```rust
   pub dev_mode: bool,  // OVERLAY_DEV_MODE=true → renders unconditionally
   ```
   Read from env var in `Default` impl.

2. **OPAQUE WM_TIMER: Add dev mode cycling** (~line 253):
   ```rust
   if s.dev_mode {
       let stage = ((s.frame_count / 300) % 5) as u32;  // 5 sec per stage
       s.progress = stage as f32 / 4.0;  // 0.0, 0.25, 0.50, 0.75, 1.0
       s.color_rgb = color_for_progress(s.progress);  // green→yellow→red
       s.visible = true;  // force visible
   }
   ```

3. **OPAQUE WM_PAINT: Replace test code with dev bar** (~lines 303-349):
   Remove the test color cycling block entirely.
   Replace with dev-mode-aware progress bar:
   ```rust
   // Calculate bar_width from progress
   let bar_width = ((window_width as f32) * s.progress.clamp(0.0, 1.0)) as i32;
   let bar_width = if s.dev_mode { bar_width.max(1) } else { bar_width };

   // Draw bar — unconditionally in dev mode, gated by visible in prod
   if s.dev_mode || s.visible {
       let color = COLORREF(
           (s.color_rgb.0 as u32) | ((s.color_rgb.1 as u32) << 8) | ((s.color_rgb.2 as u32) << 16)
       );
       let brush = CreateSolidBrush(color);
       if bar_width > 0 && !brush.is_invalid() {
           let bar_rect = RECT { left: 0, top: 0, right: bar_width, bottom: window_height };
           FillRect(hdc, &bar_rect, brush);
       }
       DeleteObject(brush.into());
   }
   ```

4. **LAYERED draw_layered_frame: Same pattern** (~line 604):
   ```rust
   if dev_mode {
       let stage = ((frame_count / 300) % 5) as u32;
       bar_progress = stage as f32 / 4.0;
       // color override happens in pixel fill
   }
   let bar_width = if dev_mode || visible {
       ((buf.width as f32) * bar_progress.clamp(0.0, 1.0) as i32).max(1)
   } else { 0 };
   ```

5. **Verify:** Run `pnpm tauri:dev` and confirm bar visible + changing width.

**⚠️ PITFALLS (from previous debugging):**
- Do NOT test LAYERED mode and claim it works for OPAQUE — they are separate code paths
- Do NOT remove test code until you verify the new code is visible on screen
- The old test code fills ENTIRE rect (`FillRect(hdc, &rect, ...)`). Your progress bar must fill PARTIAL rect (`FillRect(hdc, &bar_rect, ...)` where `bar_rect.right = bar_width`)
- `bar_width` can be 0 if `progress = 0.0` — ensure minimum 1px in dev mode
- Run `pnpm tauri:dev` (not `OVERLAY_MODE=layered`) to test OPAQUE — that's what user sees by default
- After changing code: `cd src-tauri && cargo check` to verify compilation

**Acceptance criteria:**
- `pnpm tauri:dev` → bar visible at top of screen (OPAQUE mode)
- `OVERLAY_MODE=layered pnpm tauri:dev` → bar visible (LAYERED mode)
- Bar width changes every 5 seconds (0%, 25%, 50%, 75%, 100%)
- Bar color changes with progress (green → yellow → red)
- No fast flickering/blinking (color changes every 5 seconds, not every 48ms)
- Existing unit tests pass: `cd src-tauri && cargo test overlay_renderer --lib`

**Testing strategy:**
1. Compile: `cd src-tauri && cargo check`
2. Unit tests: `cd src-tauri && cargo test overlay_renderer --lib`
3. Auto-test: `bash .claude/overlay/test-infrastructure/auto-test.sh`
4. Visual: run `pnpm tauri:dev`, look at top of screen, confirm bar visible + changing
5. Document results in `.claude/overlay/TASKS.md` (mark done or document failure)

---

### [ ] T-OVR-002: Device disconnected notification
**Priority:** P0
**Why:** App's purpose is to track desk sensor. User must know if sensor is disconnected.

**Requirements:**
- When sensor not detected on COM3: show native notification
- Notification text: "zntlDesk: Sensor not connected"
- Show on app startup if sensor missing
- Show when sensor disconnects during session
- Use `tauri-plugin-notification` (already in project)
- Don't spam: show once, then remind every 5 minutes

**Acceptance criteria:**
- Start app without sensor → notification appears
- Unplug sensor during session → notification appears
- Notification is native Windows toast (not custom UI)

---

## P1 — Core Overlay Development

### [ ] T-OVR-003: Compare OPAQUE vs LAYERED visually
**Priority:** P1
**Why:** Developer needs to see both modes to decide which to use for production.

**Requirements:**
- Both modes must be testable with dev mode (T-OVR-001)
- Document visual differences:
  - OPAQUE: black background under bar
  - LAYERED: transparent background (desktop visible)
- Screenshot comparison in test report

**Commands:**
```bash
# Test OPAQUE
OVERLAY_DEV_MODE=true pnpm tauri:dev

# Test LAYERED
OVERLAY_MODE=layered OVERLAY_DEV_MODE=true pnpm tauri:dev
```

---

### [ ] T-OVR-004: Multiple bar variants for UX testing
**Priority:** P1
**Why:** Developer wants to test different bar styles before committing to design.

**Requirements:**
- Switchable via env var: `OVERLAY_VARIANT=1|2|3|4`
- Variants:
  1. **Solid color** — current (green/yellow/red, opaque)
  2. **Semi-transparent** — 50% alpha white (LAYERED only)
  3. **Gradient** — left=full opacity, right=fading
  4. **Pulsing** — opacity oscillates (breathing effect)
- Each variant uses same progress/color API
- Developer switches variants to compare visual quality

**Implementation:**
- Add `overlay_variant` to OverlayState
- In draw functions: match variant → different fill logic
- Same bar_width calculation for all variants

---

### [ ] T-OVR-005: Adjustable bar height
**Priority:** P1
**Why:** 4px may be too thin/thick. Developer needs to test different heights.

**Requirements:**
- Env var: `OVERLAY_HEIGHT=4` (default 4, range 1-20)
- Window recreated with new height
- Test: 2px, 4px, 8px, 12px — see which is most visible without being intrusive

---

## P2 — Testing Infrastructure

### [ ] T-OVR-006: Fix auto-test.sh for OPAQUE mode
**Priority:** P2
**Why:** Current auto-test only tests LAYERED mode. Must test what user sees.

**Requirements:**
- Default: test OPAQUE mode (no OVERLAY_MODE env var)
- Add parameter: `bash auto-test.sh opaque|layered`
- Parse WM_PAINT logs (not just DRAW logs)
- Verify bar_width values in WM_PAINT output
- Report both modes separately

---

### [ ] T-OVR-007: Screenshot-based visual regression
**Priority:** P2
**Why:** Log analysis alone is insufficient — need visual proof.

**Requirements:**
- Playwright captures top 10px of screen every 500ms
- Compares frame-by-frame: is bar width growing?
- Pixel analysis: count colored pixels vs black pixels
- Generate report: "frame 0: 0% colored, frame 10: 25% colored, ..."
- Fail if: bar never changes width over 30 seconds

---

## P3 — Production Integration

### [ ] T-OVR-008: Wire overlay to real session data
**Priority:** P3
**Depends on:** T-OVR-001 (dev mode working)
**Why:** After visual development, connect to real sitting session tracking.

**Requirements:**
- In production mode (no dev flags): overlay controlled by tray_controller
- progress = sitting_seconds / session_limit_secs
- color = green (< 60%), yellow (60-85%), red (> 85%)
- Show only when DeskState::Sitting
- Hide when Standing/Walking/Away

---

### [ ] T-OVR-009: Choose production render mode
**Priority:** P3
**Depends on:** T-OVR-003 (visual comparison)
**Why:** Decide between OPAQUE and LAYERED based on visual testing.

**Decision criteria:**
- Which looks better? (user preference)
- Which is more reliable? (crash/flicker history)
- Performance impact? (CPU usage)
- Default for production release

---

## Dependency Graph

```
T-OVR-001 (dev mode visible)
├─→ T-OVR-002 (device notification)      [independent]
├─→ T-OVR-003 (compare OPAQUE vs LAYERED)
│   └─→ T-OVR-009 (choose production mode)
├─→ T-OVR-004 (bar variants)
├─→ T-OVR-005 (bar height)
├─→ T-OVR-006 (fix auto-test)
│   └─→ T-OVR-007 (visual regression)
└─→ T-OVR-008 (wire to session data)
```

**Start with T-OVR-001** — everything else depends on being able to SEE the bar.

---

## Dev Mode Architecture

```
                    ┌──────────────┐
                    │  App Startup │
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │ Check env:   │
                    │ DEV_MODE?    │
                    └──────┬───────┘
                           │
              ┌────────────┴────────────┐
              │                         │
       ┌──────▼──────┐          ┌──────▼──────┐
       │  DEV MODE   │          │  PROD MODE  │
       │             │          │             │
       │ • visible   │          │ • visible   │
       │   =true     │          │   from      │
       │   always    │          │   sensor    │
       │             │          │             │
       │ • progress  │          │ • progress  │
       │   cycles    │          │   from      │
       │   demo      │          │   session   │
       │             │          │             │
       │ • color     │          │ • color     │
       │   cycles    │          │   from      │
       │   demo      │          │   progress  │
       │             │          │             │
       │ • bar       │          │ • show      │
       │   always    │          │   device    │
       │   shown     │          │   notif if  │
       │             │          │   missing   │
       └─────────────┘          └─────────────┘
```

---

## Environment Variables

| Variable | Values | Default | Effect |
|----------|--------|---------|--------|
| `OVERLAY_MODE` | `opaque`, `layered` | `opaque` | Render backend |
| `OVERLAY_DEV_MODE` | `true`, `false` | `false` (`true` in tauri:dev) | Always-visible, demo cycling |
| `OVERLAY_VARIANT` | `1`, `2`, `3`, `4` | `1` | Visual style (future) |
| `OVERLAY_HEIGHT` | `1`-`20` | `4` | Bar height in pixels (future) |
