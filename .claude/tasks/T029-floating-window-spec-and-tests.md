# T029 — Floating Window: Spec, Expected Behavior & Tests

**Status:** open
**Priority:** P1 (spec + test coverage before any UI bug fixes)
**Branch:** feat/T029-floating-window-spec
**Note:** This task is SPEC + TESTS only. Do not fix bugs here — document what's broken,
write tests that fail, then fix in a follow-up task.

---

## Why This Task Exists

Two confirmed bugs + suspected broad undertesting:
1. **Sitting timer shows wrong value** — user sat 5 min ago, UI shows 50. Likely `sitting_seconds`
   conflates "current session" with "carry-over from before break" or "total today".
2. **Last standing duration never shown** — after Standing→Sitting transition, the popup
   shows "1 position change" but user cannot see how long they just stood.

Before fixing, we need a ground-truth spec and failing tests.

---

## Current Data Flow (read this first)

```
serial.rs ──sensor reading──► SessionManager::on_reading()
                                    │ (on state transition)
                                    ├──► emit desk:state-changed
                                    │         │
                                    │         ├──► tray_controller.rs (tray + overlay)
                                    │         └──► useDesk.ts listener → re-render
                                    └──► db.rs::save_session (persist completed session)

useDesk.ts also:
  - polls get_session_state()  every 1s  (keep in sync)
  - polls get_today_summary()  every 10s (totals + history)
  - runs useTimer() at 1 Hz   while Sitting (smooth countdown)
```

### Key fields in `SessionStateDto` (Rust → frontend)

```typescript
state: DeskState           // "Sitting" | "Standing" | "Walking" | "Away"
sitting_seconds: i64       // ⚠ currently: committed seconds in current session
                           //   (carries over if break < 5 min — BY DESIGN, but confusing)
standing_seconds: i64      // total standing seconds accumulated today (not current session!)
break_seconds: i64         // seconds in current break
session_limit_secs: i64    // configured limit (default 2700 = 45 min)
stand_limit_secs: i64      // standing limit (0 = disabled)
desk_height_cm: f32
position_changes: u32      // Sitting↔Standing transitions (in-memory, resets on restart)
```

### Break credit rules (session.rs)

These affect what `sitting_seconds` shows after the user returns to sitting:
```
Standing < 5 min  → NO credit:   sitting_seconds unchanged (session continues as if they never stood)
Standing 5-9 min  → PARTIAL:     sitting_seconds -= 20 min (can go below 0 → clamped to 0)
Standing ≥ 10 min → FULL RESET:  sitting_seconds = 0 (full break earned)
```

---

## Spec: What the Floating Window SHOULD Show

### State: Sitting

```
┌─────────────────────────────────┐
│  Sitting  •  72.3 cm            │  ← state + height
│                                 │
│       23:14                     │  ← current session timer (MM:SS)
│  ▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░░░  52%    │  ← progress bar toward 40-min limit
│  17 minutes until limit         │  ← human-readable countdown
│                                 │
│  Today:  Sit 1h23m  Stand 32m  │  ← daily totals
│  3 position changes             │  ← transition count today
└─────────────────────────────────┘
```

**Session timer semantics:**
- Shows time in the **current sitting session**, not total today
- If standing break was < 5 min: session CONTINUES from where it left off (expected, should be labeled)
- If standing break was 5-9 min: session reduced by 20 min (may jump backward — label this)
- If standing break was ≥ 10 min: resets to 0:00 (fresh start)

**Must NOT:**
- Show total sitting today in the session timer
- Show the DB-loaded today total as the starting value on first session
- Confuse `sitting_seconds` (current session carry-over) with `standing_seconds` (total today)

### State: Standing

```
┌─────────────────────────────────┐
│  Standing  •  114.0 cm          │  ← state + height
│                                 │
│       08:32                     │  ← current standing session timer (MM:SS)
│  ▓▓▓▓▓▓▓░░░░░░░░░░░░░  57%    │  ← gold bar: progress toward 15-min target
│  6 min 28 sec to complete lap  │  ← time remaining to standing target
│                                 │
│  Today:  Sit 1h23m  Stand 32m  │  ← daily totals
│  3 position changes             │
└─────────────────────────────────┘
```

**Standing session timer semantics:**
- Counts up from 0 when user stands up
- Resets to 0 on each new standing session (independent of today's total)
- Shows time since they stood up THIS TIME — not cumulative standing today

**Must NOT:**
- Show `standing_seconds` (total today) as the standing timer
- Hide this timer while user is standing (current bug: timer not shown at all)

### State: Transition moment (0–30 seconds after state change)

**Standing → Sitting transition:**
```
┌─────────────────────────────────┐
│  Sitting  •  72.3 cm            │
│                                 │
│  Stood for 12 min               │  ← last standing session duration (30s timeout)
│  +17 pts earned  🏆             │  ← if T028 is implemented
│                                 │
│  Session continues              │  ← if standing < 5 min (no credit)
│  -or-                           │
│  Session reduced by 20 min      │  ← if standing 5-9 min
│  -or-                           │
│  Fresh start!                   │  ← if standing ≥ 10 min
│                                 │
│  Today:  Sit 1h23m  Stand 44m  │
│  4 position changes             │
└─────────────────────────────────┘
```

**Sitting → Standing transition:**
```
┌─────────────────────────────────┐
│  Standing  •  114.0 cm          │
│                                 │
│  Sat for 38 min                 │  ← last sitting session duration (30s timeout)
│                                 │
│       00:05  (counting up)      │  ← standing session starts immediately
│  ░░░░░░░░░░░░░░░░░░░░   3%     │  ← gold bar starts filling
└─────────────────────────────────┘
```

### State: Walking / Away

```
┌─────────────────────────────────┐
│  Away  •  last: 114.0 cm        │
│                                 │
│       03:22                     │  ← time in Away/Walking state
│  (not counting as standing)     │  ← clarify these states don't earn points
│                                 │
│  Today:  Sit 1h23m  Stand 44m  │
│  4 position changes             │
└─────────────────────────────────┘
```

---

## Expected Behaviors — Test Scenarios

### Scenario A: Fresh start (no previous session today)

1. App starts, no sensor data yet → state = unknown/Away
2. User sits → `sitting_seconds` starts at 0
3. Timer shows 0:00 and counts up ✓

**Test assertion:** On first Sitting transition with no prior history, `sitting_seconds = 0`

### Scenario B: Short break (no credit, < 5 min)

1. User sits for 30 min → `sitting_seconds = 1800`
2. User stands for 3 min (too short, no credit)
3. User sits again
4. Timer should show **30:00** (continuing, not 0 or 33 min)
5. UI should show a label "Session continues from before break"

**Test assertion:** After standing < 5 min, returned `sitting_seconds` = pre-break value

**Bug to document:** Is this currently working or does it show wrong value?

### Scenario C: Medium break (partial credit, 5-9 min)

1. User sits for 30 min → `sitting_seconds = 1800`
2. User stands for 7 min
3. User sits again
4. Timer should show **10:00** (1800 - 1200 = 600 = 10 min)
5. UI should show "Session reduced — well done!"

**Test assertion:** `sitting_seconds` after medium break = max(0, pre_break - 1200)

### Scenario D: Full break (≥ 10 min standing)

1. User sits for 35 min → `sitting_seconds = 2100`
2. User stands for 12 min
3. User sits again
4. Timer should show **0:00** (full reset)
5. UI should show "Fresh start!"

**Test assertion:** `sitting_seconds = 0` after standing ≥ 10 min

### Scenario E: App restart mid-session

1. User sits for 20 min
2. App restarts
3. `load_today_totals()` is called — loads DB data
4. **What should happen?**
   - Option A: Session timer starts at 0 (conservative, simpler)
   - Option B: Session timer picks up from last known state (requires timestamp in DB)
5. **Current behavior unknown — needs investigation**

**Spec decision needed:** Which option? Recommend Option A (restart = fresh session, totals still accumulate).

**Test assertion:** After restart, `sitting_seconds = 0` (not DB total)

### Scenario F: Sitting timer shows total today (KNOWN BUG)

1. User sits 30 min, stands 10 min (full reset), sits 5 min
2. Expected timer: **5:00**
3. Reported behavior: shows ~50 min

**Hypothesis:** `load_today_totals()` seeds `sitting_seconds` from DB with total sitting today,
then session logic adds on top of it — so user who sat 45 min yesterday, restarts, sees 45 min
immediately on sitting.

Or: `sitting_seconds` in `SessionState` accumulates total today (not current session), and the
"current session" is computed as `total - prev_sessions_total` somewhere that's broken.

**Test assertion:** Must write a test that does sit→stand(10min)→sit and checks `sitting_seconds = 0`

### Scenario G: Last standing duration shown after transition (MISSING FEATURE)

1. User stands for 12 min
2. User sits down
3. For 30 seconds, UI should display "Stood for 12 min"
4. After 30s, transitions to normal sitting view

**Test assertion:** `StateChangedPayload` includes `last_break_seconds` or equivalent,
and UI renders it for 30s after transition.

**Current code gap:** `StateChangedPayload` does not include last standing duration.

### Scenario H: Position changes count

1. App starts: `position_changes = 0`
2. Sit → position_changes stays 0
3. Sit → Stand: position_changes = 1
4. Stand → Sit: position_changes = 2
5. Sit → Walk: position_changes stays 2 (Walk is NOT a position change)
6. Walk → Stand: position_changes stays 2

**Test assertion:** Only Sitting↔Standing transitions increment the counter.

### Scenario I: Daily reset at midnight

1. User working near midnight, sitting_seconds = 2100, position_changes = 5
2. Midnight occurs → daily reset fires
3. `sitting_seconds = 0`, `position_changes = 0`, totals reset

**Test assertion:** After daily reset, all counters are 0.

---

## Fields Missing from StateChangedPayload (bugs/gaps)

| Field | Needed for | Current status |
|-------|-----------|----------------|
| `last_break_secs: i64` | "Stood for X min" after transition | MISSING |
| `last_sitting_secs: i64` | "Sat for X min" after transition | MISSING |
| `break_credit_applied: BreakCredit` | Explain why timer jumped | MISSING |
| `standing_session_secs: i64` | Standing timer in popup | MISSING (added by T028) |

`BreakCredit` enum:
```rust
pub enum BreakCredit {
    None,           // < 5 min — session continues unchanged
    Partial(i64),   // 5-9 min — reduced by this many seconds
    Full,           // ≥ 10 min — session reset to 0
}
```

---

## Implementation Plan (SPEC + TESTS only — no fixes)

### Step 1 — Read and document current behavior

For each scenario A-I above, run the scenario manually (or via integration test) and document:
- What `sitting_seconds` returns at each step
- What the UI shows
- Whether it matches the spec

Write findings as comments in the test file: `// EXPECTED: X, ACTUAL: Y (BUG)`

### Step 2 — Write failing unit tests (Rust)

In `session_tests.rs`, add tests for:
- `test_sitting_timer_resets_after_full_break` — scenario D
- `test_sitting_timer_continues_after_no_credit_break` — scenario B
- `test_sitting_timer_reduced_after_partial_break` — scenario C
- `test_position_changes_only_on_sit_stand_transitions` — scenario H
- `test_load_today_totals_does_not_seed_sitting_seconds` — scenario E/F

### Step 3 — Write failing unit tests (TypeScript)

In `SessionProgress.test.tsx`, add tests for:
- `renders correct timer when sitting_seconds=0` — fresh start
- `renders last_break_secs after state change to Sitting` — scenario G (will fail: field missing)
- `shows break credit label when partial credit applied` — scenario C

### Step 4 — Extend StateChangedPayload (Rust)

In `session_types.rs`, add to `StateChangedPayload`:
```rust
pub last_break_secs: i64,                    // how long last break was
pub last_sitting_secs: i64,                  // how long last sitting session was
pub break_credit: String,                    // "none" | "partial" | "full"
```

This is needed so the frontend can show the transition message.

### Step 5 — Document what remains for a follow-up fix task

After tests are written and all failures are documented, create T030 with the fix list.

---

## Files to Read / Modify

| File | Action |
|------|--------|
| `src-tauri/src/session.rs` (→ `session_manager.rs` after T027) | Read: understand sitting_seconds accumulation and load_today_totals |
| `src-tauri/src/session_tests.rs` (after T027) | Write: new failing tests |
| `src/components/SessionProgress.tsx` | Read + write tests |
| `src/components/TodayStats.tsx` | Read + write tests |
| `src/hooks/useDesk.ts` | Read: understand how events update state |
| `src-tauri/src/session_types.rs` (after T027) | Modify: add fields to StateChangedPayload |

---

## Acceptance Criteria

- [ ] Document actual vs expected for all 9 scenarios (comment in test file)
- [ ] Failing Rust tests written for scenarios B, C, D, F (they may pass or fail — report which)
- [ ] Failing TypeScript test written for scenario G (must fail — field missing)
- [ ] `StateChangedPayload` extended with `last_break_secs`, `last_sitting_secs`, `break_credit`
- [ ] No behavior changes (this task = spec + tests only)
- [ ] `cargo test` runs (even if new tests fail — that's the point)
- [ ] Create follow-up task T030 listing all confirmed bugs with test references
