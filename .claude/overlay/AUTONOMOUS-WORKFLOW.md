# Autonomous Testing & Iteration Workflow

**Goal:** Claude iterates independently without asking user to run tests.

---

## How It Works

### Step 1: Auto Test
```bash
bash .claude/auto-test.sh
```

Claude runs this. Script:
1. ✅ Builds code (stops if broken)
2. ✅ Starts app with demo mode
3. ✅ Waits 30 seconds
4. ✅ Stops app
5. ✅ Parses logs automatically
6. ✅ Generates `.claude/test-runs/LATEST-RESULTS.md`

### Step 2: Claude Analyzes Results

Script outputs answer to each question:
- ✅ demo_mode=true?
- ✅ frame_count incrementing?
- ✅ progress cycling?
- ✅ bar_width changing?

**If all ✅:** Ask user for visual feedback only
```
User says: "Widać bar, zmienia się szerokość, nie miga"
→ Works! Done!
```

**If any ❌:** Claude identifies specific bug and fixes

### Step 3: Claude Fixes & Re-tests

**Example flow:**

```
Test 1: demo_mode=true ✅, frame_count ✅, progress ❌ (stuck 100%)
↓
Diagnosis: Progress overwrite not working
↓
Fix: Remove WM_TIMER demo logic, debug draw_layered_frame override
↓
Test 2: Run auto-test.sh again
↓
If fixed: Proceed. If not: Try different fix.
```

### Step 4: Loop Until Working

Claude keeps testing & fixing until either:
- ✅ All metrics pass → Ask user visual question
- ❌ Can't make progress → Report blocker + evidence to user

---

## Decision Tree

```
Auto Test Results:

    demo_mode=true?
    ├─ NO → FIX: env var reading
    │        └─ Re-test
    │
    └─ YES: frame_count incrementing?
        ├─ NO → FIX: WM_TIMER handler / timer not set
        │        └─ Re-test
        │
        └─ YES: progress cycling?
            ├─ NO (stuck 100%) → FIX: draw_layered_frame override
            │                    └─ Re-test
            │
            ├─ YES: bar_width changing?
            │   ├─ NO → FIX: bar_width calculation
            │   │        └─ Re-test
            │   │
            │   └─ YES: ALL WORKING ✅
            │        └─ Ask user: "See bar growing? Blinking?"
            │
            └─ PARTIAL (some progress, but not 0→25→50→75→100)
                → FIX: stage calculation / progress mapping
                └─ Re-test
```

---

## What Claude Does

### Per Iteration (Auto Loop)

1. **Run Test**
   ```bash
   bash .claude/auto-test.sh > /tmp/test-output.txt 2>&1
   ```

2. **Parse Results**
   - Read `.claude/test-runs/LATEST-RESULTS.md`
   - Check metrics: ✅ or ❌
   - Identify next fix

3. **Fix Code** (if needed)
   - Modify `src-tauri/src/overlay_renderer.rs`
   - Document fix in `.claude/tasks/OVERLAY-REPORT.md`
   - Commit

4. **Re-test**
   - Run `bash .claude/auto-test.sh` again
   - Compare with previous results
   - Did it improve?

5. **Report Status**
   - User sees: "Test 3: progress cycling now works! Testing bar_width..."
   - Only blockers need user input

### When to Ask User

**Visual Questions Only:**
- "Is bar visible at top of screen?" ✅/❌
- "Does bar blink/flicker?" ✅/❌
- "Does bar grow wider over time?" ✅/❌
- "How often does it change?" (1s / 5s / fast / slow)

**NOT:** "Can you run this command?"
**NOT:** "Send me logs?"
**NOT:** "Take a screenshot?"

---

## File Structure

```
.claude/
├── auto-test.sh                    ← Claude runs this
├── AUTONOMOUS-WORKFLOW.md          ← This file
├── tasks/
│   └── OVERLAY-REPORT.md           ← Updated with each iteration
└── test-runs/
    ├── LATEST-RESULTS.md           ← Newest test results
    ├── 2026-03-19_14-30-45/        ← Test run 1
    │   ├── overlay.log
    │   └── analysis.md
    ├── 2026-03-19_14-35-12/        ← Test run 2
    │   ├── overlay.log
    │   └── analysis.md
    └── 2026-03-19_14-40-30/        ← Test run 3
        ├── overlay.log
        └── analysis.md
```

---

## Example Iteration

### Iteration 1: Baseline
```
🧪 Auto Test 1
├─ demo_mode: ❌ NOT FOUND
├─ frame_count: ✅ 30 TIMER logs
├─ progress: ❌ NO DEMO-OVERRIDE logs
└─ bar_width: ❌ all 1/1920 (stuck)

📝 Diagnosis: demo_mode not set to true
🔧 Fix: Debug env var reading
```

### Iteration 2: Env Var Fix
```
🧪 Auto Test 2
├─ demo_mode: ✅ FOUND true
├─ frame_count: ✅ 30 TIMER logs
├─ progress: ✅ 900+ DEMO-OVERRIDE logs
│            First: 0.00%, Last: 100.00%
└─ bar_width: ✅ 1/1920 → 1920/1920

📝 Diagnosis: Progress cycling works! But stuck at 100%
🔧 Fix: Check draw_layered_frame override logic
```

### Iteration 3: Progress Override
```
🧪 Auto Test 3
├─ demo_mode: ✅ true
├─ frame_count: ✅ 30 TIMER
├─ progress: ✅ Cycles 0% → 25% → 50% → 75% → 100%
│            First: 0.00%, Last: 100.00%
└─ bar_width: ✅ 1/1920 → 480/1920 → 960/1920 → 1440/1920 → 1920/1920

📝 Diagnosis: ALL METRICS PASS ✅
🤔 Question for user: "Widzisz to? Bar rosnący? Jak szybko zmienia?"
```

---

## Rules for Iteration

1. **Every test run is clean** (new timestamp, no old logs mixed)
2. **Parse first, change second** (analyze results before fixing)
3. **One fix per iteration** (don't change 5 things at once)
4. **Document findings** (update OVERLAY-REPORT.md each time)
5. **Only ask user visual questions** (never operational ones)
6. **Stop if stuck 3+ iterations on same issue** → Ask user for evidence

---

## Implementation

Claude runs autonomously:

```
while not_solved:
    test_results = run_auto_test()
    findings = parse_results(test_results)

    if findings.all_working:
        ask_user_visual_question()
        break
    else:
        issue = identify_blocker(findings)
        fix_code(issue)
        document_fix(issue)
        # loop continues
```

No user intervention until all automated tests pass.
