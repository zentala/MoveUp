# Overlay Testing Guide

## Step 1: Capture Logs

Run app with demo mode and redirect logs:

```bash
cd C:\code\zntl-tray\apps\desk
OVERLAY_MODE=layered OVERLAY_DEMO_MODE=true RUST_LOG=warn pnpm tauri:dev 2>&1 | tee overlay-live.log
```

Or capture to file:
```bash
OVERLAY_MODE=layered OVERLAY_DEMO_MODE=true RUST_LOG=warn pnpm tauri:dev > overlay-debug.log 2>&1 &
```

## Step 2: Watch for These Log Lines

**Every ~1 second, should see:**
```
[TIMER] frame_count=60, demo_mode=true, visible=true
[TIMER] frame_count=120, demo_mode=true, visible=true
[TIMER] frame_count=180, demo_mode=true, visible=true
```

**Every frame (very frequent), should see:**
```
[DEMO-OVERRIDE] frame_count=1, stage=0, bar_progress=0.00% (before: calc from state)
[DEMO-OVERRIDE] frame_count=2, stage=0, bar_progress=0.00%
...
[DEMO-OVERRIDE] frame_count=300, stage=1, bar_progress=25.00%
[DEMO-OVERRIDE] frame_count=301, stage=1, bar_progress=25.00%
...
[DEMO-OVERRIDE] frame_count=600, stage=2, bar_progress=50.00%
```

**Draw calls should show progression:**
```
[DRAW] frame=1 | visible=true | progress=0.00% | bar_width=1/1920 | demo=true
[DRAW] frame=60 | visible=true | progress=0.00% | bar_width=1/1920 | demo=true
[DRAW] frame=300 | visible=true | progress=25.00% | bar_width=480/1920 | demo=true
[DRAW] frame=600 | visible=true | progress=50.00% | bar_width=960/1920 | demo=true
```

## Step 3: Questions Log Should Answer

1. **Is demo_mode=true?** (check TIMER line)
2. **Is frame_count incrementing?** (60, 120, 180, 240...)
3. **Is stage cycling?** (0, 0, 0... then 1, 1, 1... then 2, 2, 2...)
4. **Is bar_progress changing?** (0.00% -> 25.00% -> 50.00% -> 75.00% -> 100.00%)
5. **Is bar_width calculated correctly?** (0% = 1px, 25% = 480px, 50% = 960px, 100% = 1920px)
6. **Is visible=true?**

## Step 4: Send Logs + Visual Description

After running for 10-15 seconds, stop (Ctrl+C) and send me:

1. **First 5 seconds of log output** (show TIMER, DEMO-OVERRIDE, and DRAW lines)
2. **Describe what you see on screen:**
   - Is bar visible?
   - Does it blink? How fast?
   - Does it grow wider? Or stay same width?
   - Any color changes?
   - Any patterns?

## Current Status (Before Testing)

Expected with `OVERLAY_DEMO_MODE=true`:
- demo_mode should be true
- frame_count should count: 1, 2, 3, 4...
- stage should cycle: 0->0->0...->1->1->1...->2... (every 300 frames)
- bar_progress should cycle: 0%->25%->50%->75%->100%
- bar_width should grow: 1px->480px->960px->1440px->1920px
- bar should be visible
- blinking should stop (or not, need to see)

If any of these don't match -> that's the bug.
