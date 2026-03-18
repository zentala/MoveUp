# Overlay Testing System

## Overview

**Problem:** We can't verify if changes work. Need automated testing that:
1. Captures logs with clean state (no old data)
2. Takes timestamped screenshots synchronized with logs
3. Generates analysis comparing expectations vs reality
4. Creates evidence for debugging

**Solution:** Three-part system

---

## Part 1: Test Harness (Logging)

**File:** `.claude/test-harness.sh`

**What it does:**
1. Creates clean test directory with timestamp: `.claude/test-runs/2026-03-19_14-30-45/`
2. Cleans up any old app processes
3. Starts app with `OVERLAY_MODE=layered OVERLAY_DEMO_MODE=true`
4. Runs for 15 seconds (or specify different duration)
5. Captures logs to `.log` file (no old data mixed in)
6. Stops app cleanly
7. Analyzes logs: counts TIMER, DEMO-OVERRIDE, DRAW entries
8. Generates `analysis.md` with:
   - Log summary
   - Sample entries showing progression
   - Checklist of questions

**Usage:**
```bash
cd C:\code\zntl-tray\apps\desk
bash .claude/test-harness.sh 15    # Run for 15 seconds
```

**Output:**
```
.claude/test-runs/2026-03-19_14-30-45/
├── overlay.log          (raw logs)
├── analysis.md          (questions + checklist)
└── screenshots/         (filled by Part 2)
```

---

## Part 2: Screenshot Capture (Visual Sync)

**File:** `tests/overlay-screenshots.test.ts`

**What it does:**
1. Runs alongside the harness
2. Takes screenshot of top 10px every 500ms
3. Saves as `frame-0.png`, `frame-1.png`, etc.
4. Creates `manifest.json` mapping:
   - Frame number → Time (ms) → File path
5. Generates `ANALYSIS.md` with:
   - Table of expected bar width at each time
   - Frame-by-frame reference
   - Checklist of visual observations

**Usage (while harness is running):**
```bash
# Terminal 1: Start harness
bash .claude/test-harness.sh 30

# Terminal 2: Start playwright capture (after app is running)
pnpm test:overlay-screenshots
```

**Output:**
```
.claude/test-runs/2026-03-19_14-30-45/screenshots/
├── frame-0.png                     (at 0.0s)
├── frame-10.png                    (at 5.0s)
├── frame-20.png                    (at 10.0s)
├── frame-40.png                    (at 20.0s) ← Should be 100% width
├── manifest.json                   (frame → time mapping)
└── ANALYSIS.md                     (expectations + checklist)
```

---

## Part 3: Log-Screenshot Synchronization

**How they work together:**

1. **Frame Number = Time Key**
   - Log entry at frame 300 = screenshot at frame-number (300/60 ≈ 5s)
   - Can correlate: "At frame-10.png (5s), logs show stage=1, progress=25%"

2. **Manifest Links Everything**
   - `manifest.json` in screenshots dir shows exact times
   - `overlay.log` has timestamps
   - Can overlay: log values on screenshot analysis

3. **Evidence Trail**
   ```
   Frame 10 (5.0s):
   - Screenshot: frame-10.png (should show 25% width bar)
   - Log entry: [DEMO-OVERRIDE] frame_count=300, stage=1, bar_progress=25.00%
   - Expected: 25% of 1920px = 480px wide
   - If screenshot shows full 1920px → BUG FOUND: progress not overriding
   ```

---

## Step-by-Step Testing

### 1. Clean Test Run

```bash
cd C:\code\zntl-tray\apps\desk

# Terminal 1: Run harness
bash .claude/test-harness.sh 30
# (waits 30 seconds, then stops)

# Output: .claude/test-runs/2026-03-19_14-30-45/overlay.log
```

### 2. Check Logs

```bash
cat .claude/test-runs/2026-03-19_14-30-45/analysis.md

# Look for:
# - demo_mode=true?
# - frame_count incrementing?
# - stage cycling 0→1→2→3→4?
# - progress changing?
```

### 3. Take Screenshots

```bash
# Terminal 2 (while harness still running OR use captured video)
pnpm test:overlay-screenshots
# Captures frame-0.png through frame-60.png every 500ms
```

### 4. Verify Visually

```bash
# Open in image viewer:
.claude/test-runs/2026-03-19_14-30-45/screenshots/

# Check:
# - frame-10.png (5s): bar should be 25% width
# - frame-20.png (10s): bar should be 50% width
# - frame-40.png (20s): bar should be 100% width

# Compare against ANALYSIS.md expectations
```

### 5. Generate Report

Both tools create analysis:
- Logs: `analysis.md` with checklist
- Screenshots: `ANALYSIS.md` with expectations
- You verify each item: ✅ or ❌

---

## What We'll Learn

**From logs:**
```
✅ demo_mode=true means env var was read
✅ frame_count=60,120,180... means timer is working
✅ stage=0,1,2,3,4 means cycle logic works
✅ progress=0.00%,25.00%,50.00%... means math is right
✅ bar_width=1,480,960,1440,1920 means calculation is right
```

**From screenshots:**
```
✅ Bar visible means rendering happens
✅ Bar grows means width calculation applied
✅ No flicker means refresh rate OK
✅ Correct width at each frame = everything works
```

**If mismatch:**
```
❌ Logs say progress=25% but screenshot shows 100% width
   → Progress override not working
   → Bug: tray_controller overwriting demo progress

❌ Logs say frame_count=600 but screenshot stuck at 0%
   → Rendering not called or not updating
   → Bug: draw_layered_frame not invoked
```

---

## File Structure

```
.claude/
├── test-harness.sh                 ← Run this
├── TESTING-OVERLAY.md              ← Manual testing (deprecated)
├── TESTING-SYSTEM.md               ← This file
└── test-runs/
    └── 2026-03-19_14-30-45/        ← Timestamped test run
        ├── overlay.log             ← Raw logs
        ├── analysis.md             ← Log analysis
        └── screenshots/
            ├── frame-0.png         ← Captured images
            ├── frame-10.png
            ├── manifest.json       ← Frame ↔ time mapping
            └── ANALYSIS.md         ← Screenshot expectations
```

---

## Next: Run the Test

1. Run harness: `bash .claude/test-harness.sh 30`
2. Watch app run (check if bar visible, blinking, etc.)
3. After 30s, check logs: `cat .claude/test-runs/[timestamp]/analysis.md`
4. If logs show demo_mode=true + progress cycling:
   - Take screenshots with Playwright
   - Verify visual matches log values
5. Report findings in `.claude/tasks/OVERLAY-REPORT.md`
