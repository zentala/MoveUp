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

**Requirements:**
- When `OVERLAY_DEV_MODE=true`: bar renders unconditionally (no `visible` check)
- Bar shows even without desk sensor connected
- Progress cycles: 0% → 25% → 50% → 75% → 100% (every 5 seconds)
- Color cycles: green → yellow → red (every 5 seconds, synced with progress)
- Works in BOTH OPAQUE and LAYERED modes
- Default: `pnpm tauri:dev` should enable dev mode automatically

**Implementation approach:**
- Modify WM_PAINT (OPAQUE): remove `if s.visible` gate when dev_mode=true
- Modify draw_layered_frame (LAYERED): same
- Base on existing test code (proven visible) — modify shape, not mechanism
- Test code fills full rect → change to partial rect (bar_width)

**Acceptance criteria:**
- `pnpm tauri:dev` → bar visible at top of screen
- Bar width changes every 5 seconds (0%, 25%, 50%, 75%, 100%)
- Bar color changes with progress (green → yellow → red)
- No fast flickering/blinking

**Testing:**
- Auto-test: `bash .claude/overlay/test-infrastructure/auto-test.sh`
- Visual: developer sees bar on screen

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
