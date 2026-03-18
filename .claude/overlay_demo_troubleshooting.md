# Overlay Demo Mode — Troubleshooting History

**Date:** 2026-03-19
**Status:** 🔴 NOT WORKING AS EXPECTED

## Problem Summary

Demo mode should cycle progress bar width: 0% → 25% → 50% → 75% → 100% every 5 seconds.

**What's happening instead:**
- Progress bar is ALWAYS at 100% width (full screen)
- No visible cycling/changes
- Blinking/flickering happens regardless of demo mode on/off
- Demo mode env var may not be activating correctly

---

## Approaches Tried & Results

### Approach 1: Add `demo_mode` flag to OverlayState
**What we did:**
- Added `pub demo_mode: bool` field to OverlayState
- Set from env var `OVERLAY_DEMO_MODE=true` in Default impl
- Logic: If demo_mode, override progress with calculated stage every WM_TIMER

**Code:**
```rust
pub demo_mode: bool,

// In Default impl:
let demo_mode = std::env::var("OVERLAY_DEMO_MODE")
    .map(|v| v.to_lowercase() == "true")
    .unwrap_or(false);
```

**Result:** ❌ FAILED
- Progress still shows 100%
- No cycling observed
- Env var may not be read correctly

---

### Approach 2: Cycle progress every 60 frames (1 second per stage)
**What we did:**
```rust
if s.demo_mode {
    let stage = ((s.frame_count / 60) % 5) as u32;
    s.progress = stage as f32 / 4.0; // 0/4, 1/4, 2/4, 3/4, 4/4
}
```

**Result:** ❌ FAILED
- No visible change in bar width
- User reported: "cały czas jest na całą szerokość" (always at full width)
- Blinking too fast

---

### Approach 3: Slow down 5x (300 frames per stage = 5 seconds)
**What we did:**
```rust
let stage = ((s.frame_count / 300) % 5) as u32; // 300 frames @ 60fps
s.progress = stage as f32 / 4.0;
s.visible = true; // Added to ensure bar is visible
```

**Result:** ❌ FAILED
- Same behavior: bar always 100% width
- Slowdown doesn't appear to have worked
- User: "te zwolnienie nie podzialało" (slowdown didn't work)

---

## Possible Root Causes

### 1. **Env Var Not Being Read**
- `OVERLAY_DEMO_MODE=true` set in shell, but not reaching Rust code
- Possible: env var is read BEFORE app starts, not at runtime
- When `OverlayRenderer::new()` is called, demo_mode might not reflect the env var

### 2. **Demo Mode Not Triggering in Correct Overlay Instance**
- Code added to BOTH OPAQUE and LAYERED modes
- But maybe only one is running, and the other has the demo logic?
- User is testing with `OVERLAY_MODE=layered OVERLAY_DEMO_MODE=true`
- But maybe OPAQUE mode is running instead?

### 3. **Progress Always 1.0 from tray_controller**
- Even in demo mode, `tray_controller.rs` calls `overlay.update(progress, ...)`
- This overwrites the demo-calculated progress!
- **Most likely issue:** Demo progress gets overwritten by real progress every time state changes

```rust
// In tray_controller.rs on_state_changed():
overlay.update(progress, (r, g, b));  // ← This overwrites demo progress!
```

### 4. **Visible Flag Not Persisting**
- Set `s.visible = true` in WM_TIMER demo mode
- But `tray_controller` might set it to false for non-Sitting states
- Race condition: demo sets visible, tray immediately unsets it

### 5. **Frame Count Wrapping**
- `frame_count` is `u32`, wraps at ~4 billion
- Math: `(frame_count / 300) % 5` should work, but maybe there's integer precision issue?

---

## Observations

1. **Regular (non-demo) mode also blinking** — suggests rendering is updating too frequently
2. **Bar always 100% width** — progress stuck at 1.0, not reading demo calculation
3. **Demo flag might not be set** — env var read, but `demo_mode` stays false
4. **Real progress overwrites demo** — most likely culprit

---

## Next Steps to Try

### Option A: Disable tray_controller updates in demo mode
Modify `tray_controller.rs` to skip `overlay.update()` when demo mode is active:
```rust
if !overlay_state.demo_mode {
    overlay.update(progress, (r, g, b));
    overlay.show();
}
```

### Option B: Make demo mode override at rendering time
Move demo calculation to `draw_layered_frame()` instead of WM_TIMER, so it always overwrites progress just before rendering:
```rust
let bar_progress = if state.demo_mode {
    let stage = ((frame_count / 300) % 5) as u32;
    stage as f32 / 4.0
} else {
    s.progress
};
```

### Option C: Debug env var reading
Add logging to confirm `OVERLAY_DEMO_MODE` env var is actually being read:
```rust
let demo_mode = std::env::var("OVERLAY_DEMO_MODE")
    .map(|v| {
        log::info!("🔍 OVERLAY_DEMO_MODE env var = {:?}", v);
        v.to_lowercase() == "true"
    })
    .unwrap_or_else(|_| {
        log::info!("🔍 OVERLAY_DEMO_MODE not set, default=false");
        false
    });
```

### Option D: Reduce rendering frequency
- Currently renders every 16ms (60fps)
- Maybe reduce to 30fps (32ms) to make changes more visible?

---

## Files Modified

- `src-tauri/src/overlay_renderer.rs` — Added demo_mode flag, cycling logic, visible=true
- Commits:
  - `46e9fe2` - feat: add demo mode
  - `4ec7e62` - fix: 5x slower cycling, visible=true

## Test Commands

```bash
# Should activate demo mode (not working)
OVERLAY_MODE=layered OVERLAY_DEMO_MODE=true pnpm tauri:dev

# Without demo (normal mode, also shows problem)
pnpm tauri:dev
```

Both show same blinking, bar always 100% width.

---

## Summary

**We have:**
- ✅ Demo flag in state
- ✅ Cycling logic in WM_TIMER
- ✅ Visible=true set
- ❌ Progress not actually cycling (stays 1.0)
- ❌ Bar stays 100% width
- ❓ Demo mode activation unclear

**Most likely cause:** Real `tray_controller` progress updates override demo progress.
