# Overlay Bar Demo Mode — Iteration Report

**Status:** BROKEN — Progress not cycling, blinking present
**Last Updated:** 2026-03-19
**Task ID:** overlay-testing-framework

---

## Summary of Attempts

### Iteration 1: Add demo_mode flag + WM_TIMER cycling
**Date:** 2026-03-19 (Commit: 46e9fe2)

**Approach:**
- Added `demo_mode: bool` field to OverlayState
- Read from `OVERLAY_DEMO_MODE=true` env var
- In WM_TIMER: calculate progress from frame_count every 60 frames (1 sec per stage)

**Code:**
```rust
if s.demo_mode {
    let stage = ((s.frame_count / 60) % 5) as u32;
    s.progress = stage as f32 / 4.0;
}
```

**Expected:** Bar cycles 0% -> 25% -> 50% -> 75% -> 100% every 1 second

**Observed:** No change. Bar always 100% width.

**Why Failed:** tray_controller keeps calling `overlay.update(progress, ...)` with real progress (1.0), overwriting demo progress before rendering.

**Lessons:**
- WM_TIMER happens before drawing, not at render time
- State updates from other threads override demo progress
- Need to override at LAST moment (in draw function)

---

### Iteration 2: Slow down 5x (300 frames per stage)
**Date:** 2026-03-19 (Commit: 4ec7e62)

**Approach:**
- Changed cycle from 60 frames (1s) to 300 frames (5s)
- Added `s.visible = true` in WM_TIMER

**Observed:** Same as before. No cycling. Always 100%.

**Why Failed:** Same root cause as iteration 1. Progress overwritten before rendering.

---

### Iteration 3: Move demo calculation to draw_layered_frame()
**Date:** 2026-03-19 (Commit: 0ff291f)

**Approach:**
- Removed WM_TIMER progress cycling
- Added demo override IN draw_layered_frame() (last moment before bar width calc)

**Observed:** Still not working. User reports: "nic sie nie zmienilo, miga caly czas"

**Why Failed:** Unknown. Agent tested LAYERED mode while user was seeing OPAQUE mode.

---

## CRITICAL DISCOVERY

### What Actually Works
**TEST CODE in OPAQUE mode:**
- Cycles through colors every 3 frames (fast, visible)
- Fills entire screen with colors
- **User can see it**

### What Doesn't Work
**Progress bar (real code):**
- Supposed to show: 0% -> 25% -> 50% -> 75% -> 100%
- Actually shows: NOTHING (invisible)
- Rendering is broken or bar_width=0

### Root Cause
- Agent tested LAYERED mode while user saw OPAQUE mode — two completely different code paths
- Progress bar invisible because `visible=false` without desk sensor connected
- Test code (color cycling) was the only visible element because it renders unconditionally

---

## Commits So Far

- `f5b885f` - fix(overlay): remove debug logging, restore original state logic
- `46e9fe2` - feat(overlay): add demo mode for progress bar animation testing
- `4ec7e62` - fix(overlay): 5x slower demo mode cycling and always visible
- `0ff291f` - fix(overlay): demo mode progress override at render time

All have: No visible effect reported by user
