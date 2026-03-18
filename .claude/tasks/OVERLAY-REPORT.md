# Overlay Bar Demo Mode — Iteration Report

**Status:** 🔴 BROKEN — Progress not cycling, blinking present
**Last Updated:** 2026-03-19
**Task ID:** overlay-testing-framework

---

## Summary of Attempts

### ❌ Iteration 1: Add demo_mode flag + WM_TIMER cycling
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

**Expected:** Bar cycles 0% → 25% → 50% → 75% → 100% every 1 second

**Observed:** ❌ No change. Bar always 100% width.

**Why Failed:** tray_controller keeps calling `overlay.update(progress, ...)` with real progress (1.0), overwriting demo progress before rendering.

**Lessons:**
- WM_TIMER happens before drawing, not at render time
- State updates from other threads override demo progress
- Need to override at LAST moment (in draw function)

---

### ❌ Iteration 2: Slow down 5x (300 frames per stage)
**Date:** 2026-03-19 (Commit: 4ec7e62)

**Approach:**
- Changed cycle from 60 frames (1s) to 300 frames (5s)
- Added `s.visible = true` in WM_TIMER

**Code:**
```rust
let stage = ((s.frame_count / 300) % 5) as u32;
s.progress = stage as f32 / 4.0;
s.visible = true;
```

**Expected:** Bar changes every 5 seconds, always visible

**Observed:** ❌ Same as before. No cycling. Always 100%.

**Why Failed:** Same root cause as iteration 1. Progress overwritten before rendering.

**Lessons:**
- Visibility fix worked (bar shows)
- Progress calculation in WM_TIMER never reaches render due to overwrites
- Setting visible in WM_TIMER is too early — better to set in demo override

---

### ❌ Iteration 3: Move demo calculation to draw_layered_frame()
**Date:** 2026-03-19 (Commit: 0ff291f)

**Approach:**
- Removed WM_TIMER progress cycling
- Added demo override IN draw_layered_frame() (last moment before bar width calc)
- If `demo_mode=true`, calculate progress from frame_count right before rendering

**Code:**
```rust
if demo_mode {
    let stage = ((frame_count / 300) % 5) as u32;
    bar_progress = stage as f32 / 4.0;
    log::info!("[DEMO] stage={}, progress={:.0}%", stage, bar_progress * 100.0);
}
```

**Expected:** Progress overridden at last moment, no overwrites possible

**Observed:** ❌ Still not working. User reports: "nic się nie zmieniło, miga cały czas"

**Why Failed:** Unknown. Possibilities:
1. demo_mode flag not set to true (env var not read?)
2. draw_layered_frame not being called
3. Logging shows but changes not visible
4. Issue is not progress — issue is rendering/blinking itself

**Lessons:**
- Need to verify demo_mode is actually true via logs
- Need visual confirmation (screenshot testing)
- Blinking issue may be separate from progress issue

---

## Known Problems

### Problem A: Progress Always 100%
- Bar always fills entire width
- Demo cycling (0%, 25%, 50%, etc.) never happens
- Hypothesis: demo_mode flag not activating OR frame_count not incrementing OR bar_width calculation wrong

### Problem B: Blinking/Flickering
- Bar flashes or blinks continuously
- Happens even without demo mode
- Frequency: unclear (user says "zbyt szybko" but can't measure)
- Hypothesis: rendering every frame causes flicker? 16ms too fast?

### Problem C: Can't Verify Anything
- No way to see what's actually happening
- Only user feedback ("nie działa")
- No measurements, no frame-by-frame analysis
- Need: automated visual testing

---

## Next Steps (New Approach)

### Phase 1: Add Comprehensive Logging
```rust
log::info!("[OVERLAY] Frame {}: demo_mode={}, visible={}, progress={:.2}%, bar_width={}",
    frame_count, demo_mode, visible, bar_progress * 100.0, bar_width);
```
Write to file: `.claude/overlay-debug.log` every frame
User analyzes log to see actual values

### Phase 2: Screenshot Testing
Playwright screenshots every 500ms of top 10 pixels
Visual proof of: bar width, color, position
Detect blinking pattern (frame-by-frame analysis)

### Phase 3: Rust Self-Tests
Unit test for demo cycling (mocked timer, no rendering)
Assert progress values match expected stages

### Phase 4: Root Cause Analysis
Based on logging + screenshots, identify:
- Is demo_mode truly on?
- Is frame_count incrementing?
- Is bar_width being calculated correctly?
- What causes blinking?

---

## File Locations

- **This report:** `.claude/tasks/OVERLAY-REPORT.md`
- **Debug log:** `.claude/overlay-debug.log` (to be created)
- **Screenshots:** `.claude/overlay-screenshots/` (to be created)
- **Test results:** `.claude/overlay-test-results.md` (to be created)
- **Code:** `src-tauri/src/overlay_renderer.rs`

---

## Rules for Future Iterations

1. **Before proposing a fix:** Ask "How will we verify this works?"
2. **After coding:** Run logging test (check debug log)
3. **If same mistake 3+ times:** Document in this report + pause before next attempt
4. **No "should work" claims** — must have test proof

---

## Commits So Far

- `f5b885f` - fix(overlay): remove debug logging, restore original state logic
- `46e9fe2` - feat(overlay): add demo mode for progress bar animation testing
- `4ec7e62` - fix(overlay): 5x slower demo mode cycling and always visible
- `0ff291f` - fix(overlay): demo mode progress override at render time

All have: ❌ No visible effect reported by user

---

## What Actually Works ✅

- Bar appears on screen (semi-transparent white)
- Background is transparent
- Window is 4px tall, full screen width
- No crashes

## What Doesn't Work ❌

- Progress cycling (always 100%)
- Speed control
- Blinking/flickering
- Cannot verify any changes with current setup
