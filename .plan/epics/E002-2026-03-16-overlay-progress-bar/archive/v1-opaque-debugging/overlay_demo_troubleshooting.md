# Overlay Demo Mode — Troubleshooting History

**Date:** 2026-03-19
**Status:** FIX APPLIED — Commit 0ff291f

## Problem Summary

Demo mode should cycle progress bar width: 0% -> 25% -> 50% -> 75% -> 100% every 5 seconds.

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

**Result:** FAILED
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

**Result:** FAILED
- No visible change in bar width
- Blinking too fast

---

### Approach 3: Slow down 5x (300 frames per stage = 5 seconds)
**What we did:**
```rust
let stage = ((s.frame_count / 300) % 5) as u32; // 300 frames @ 60fps
s.progress = stage as f32 / 4.0;
s.visible = true; // Added to ensure bar is visible
```

**Result:** FAILED
- Same behavior: bar always 100% width
- Slowdown doesn't appear to have worked

---

## Possible Root Causes

### 1. Env Var Not Being Read
- `OVERLAY_DEMO_MODE=true` set in shell, but not reaching Rust code

### 2. Demo Mode Not Triggering in Correct Overlay Instance
- Code added to BOTH OPAQUE and LAYERED modes
- But maybe only one is running, and the other has the demo logic

### 3. Progress Always 1.0 from tray_controller
- Even in demo mode, `tray_controller.rs` calls `overlay.update(progress, ...)`
- This overwrites the demo-calculated progress!
- **Most likely issue**

### 4. Visible Flag Not Persisting
- Set `s.visible = true` in WM_TIMER demo mode
- But `tray_controller` might set it to false for non-Sitting states

---

## Summary

**We have:**
- Demo flag in state
- Cycling logic in WM_TIMER
- Visible=true set
- Progress not actually cycling (stays 1.0)
- Bar stays 100% width
- Demo mode activation unclear

**Most likely cause:** Real `tray_controller` progress updates override demo progress.
