# Autonomous Testing & Iteration Workflow

**Goal:** Claude iterates independently without asking user to run tests.

---

## How It Works

### Step 1: Auto Test
```bash
bash .claude/auto-test.sh
```

Claude runs this. Script:
1. Builds code (stops if broken)
2. Starts app with demo mode
3. Waits 30 seconds
4. Stops app
5. Parses logs automatically
6. Generates `.claude/test-runs/LATEST-RESULTS.md`

### Step 2: Claude Analyzes Results

Script outputs answer to each question:
- demo_mode=true?
- frame_count incrementing?
- progress cycling?
- bar_width changing?

**If all pass:** Ask user for visual feedback only
```
User says: "Widac bar, zmienia sie szerokosc, nie miga"
-> Works! Done!
```

**If any fail:** Claude identifies specific bug and fixes

### Step 3: Claude Fixes & Re-tests

**Example flow:**

```
Test 1: demo_mode=true pass, frame_count pass, progress fail (stuck 100%)
->
Diagnosis: Progress overwrite not working
->
Fix: Remove WM_TIMER demo logic, debug draw_layered_frame override
->
Test 2: Run auto-test.sh again
->
If fixed: Proceed. If not: Try different fix.
```

### Step 4: Loop Until Working

Claude keeps testing & fixing until either:
- All metrics pass -> Ask user visual question
- Can't make progress -> Report blocker + evidence to user

---

## Decision Tree

```
Auto Test Results:

    demo_mode=true?
    |-- NO -> FIX: env var reading
    |        -> Re-test
    |
    -> YES: frame_count incrementing?
        |-- NO -> FIX: WM_TIMER handler / timer not set
        |        -> Re-test
        |
        -> YES: progress cycling?
            |-- NO (stuck 100%) -> FIX: draw_layered_frame override
            |                    -> Re-test
            |
            |-- YES: bar_width changing?
            |   |-- NO -> FIX: bar_width calculation
            |   |        -> Re-test
            |   |
            |   -> YES: ALL WORKING
            |        -> Ask user: "See bar growing? Blinking?"
            |
            -> PARTIAL (some progress, but not 0->25->50->75->100)
                -> FIX: stage calculation / progress mapping
                -> Re-test
```

---

## Rules for Iteration

1. **Every test run is clean** (new timestamp, no old logs mixed)
2. **Parse first, change second** (analyze results before fixing)
3. **One fix per iteration** (don't change 5 things at once)
4. **Document findings** (update OVERLAY-REPORT.md each time)
5. **Only ask user visual questions** (never operational ones)
6. **Stop if stuck 3+ iterations on same issue** -> Ask user for evidence

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
