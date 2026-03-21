# T030 — Floating Window: Fix Bugs + Show Transition Info

**Status:** open
**Priority:** P1
**Depends on:** T029 (spec + tests + StateChangedPayload extended), T027 (session.rs split)
**Branch:** feat/T030-floating-window-fix

---

## Bugs to Fix

### Bug 1 — Sitting timer shows wrong value on session start

**Symptom:** User sat down 5 min ago, timer shows 50 min.

**Root cause hypothesis (confirm via T029 tests):**
`load_today_totals()` in `SessionManager` seeds `sitting_seconds` from the DB total sitting
today. When the user next sits, the session timer starts at the total (e.g. 45 min) instead of 0.

`sitting_seconds` in `SessionState` appears to serve two roles:
- "current session timer" (used by SessionProgress in the popup)
- "committed sitting time" that accumulates across sessions

These need to be separated. The session timer must show time in the CURRENT sitting session,
not total today.

**Fix:**
Add `current_session_secs: i64` to `SessionState` that resets to 0 on every Sitting state entry
(after applying break credit). Keep `sitting_seconds` as the daily accumulator for TodayStats.

Update `SessionStateDto` to expose both:
```rust
pub sitting_seconds: i64,         // total today (for TodayStats)
pub current_session_secs: i64,    // current session only (for SessionProgress timer)
```

Update `SessionProgress.tsx` to use `current_session_secs` instead of `sitting_seconds`.

**⚠ Wait for T029:** If T029 tests disprove this hypothesis, update the fix accordingly.
The fix approach may differ — do not implement until T029 confirms root cause.

---

### Bug 2 — No "Stood for X min" shown after Standing→Sitting transition

**Symptom:** After standing 12 min and sitting down, popup shows "1 position change" but
nothing about how long the standing session was.

**Root cause:** `StateChangedPayload` does not include `last_break_secs` or `last_sitting_secs`.
T029 adds these fields to the payload. This task wires them into the frontend.

**Fix — Frontend (SessionProgress.tsx or new TransitionBanner.tsx):**

Show a transition banner for 30 seconds after a state change:

```
Standing → Sitting:
  "Stood for 12 min  ✓"         (if ≥ 10 min = full credit)
  "Stood for 7 min  (partial)"  (if 5-9 min = partial credit)
  "Stood for 3 min  (too short)" (if < 5 min = no credit)

Sitting → Standing:
  "Sat for 38 min"
```

Implementation:
- `useDesk.ts`: on `desk:state-changed` event, save `lastBreakSecs` and `breakCredit` to local state
- Set a 30-second timeout to clear the banner
- `SessionProgress.tsx` (or new `TransitionBanner.tsx`): render banner when `lastBreakSecs > 0`

**Break credit labels:**
```typescript
function breakCreditLabel(credit: string, secs: number): string {
  const min = Math.round(secs / 60);
  switch (credit) {
    case 'full':    return `Stood ${min} min — full reset ✓`;
    case 'partial': return `Stood ${min} min — session reduced`;
    case 'none':    return `Stood ${min} min — session continues`;
  }
}
```

---

### Bug 3 — Standing session timer not shown while standing

**Symptom:** While user is standing, popup does not show how long they have been standing.
"Break duration" field exists but may not be wired or rendered correctly.

**Root cause:** `break_seconds` in `SessionStateDto` should count up while Standing, but
the UI may not render it in the standing state, or `useTimer()` may only tick while Sitting.

**Fix:**
- `useDesk.ts`: run `useTimer()` for both Sitting AND Standing states
- `App.tsx`: ensure the standing duration section renders `break_seconds` while state === "Standing"
- Add label "Standing for: 08:32" (distinct from "Break:" which implies inactivity)

**Note:** After T028, this timer will be replaced by `standing_session_secs` from SessionStateDto.
Coordinate with T028 agent — use `standing_session_secs` if available, fall back to `break_seconds`.

---

## New Features (while touching this code)

### Feature: Break credit explanation label in SessionProgress

When the user returns to sitting, show which break credit rule applied:

```
┌──────────────────────────────────┐
│  Sitting  •  72.3 cm             │
│                                  │
│  Session continues from 30:00    │  ← if no credit (< 5 min break)
│  -or-                            │
│  Session reduced to 10:00        │  ← if partial credit (5-9 min break)
│  -or-                            │
│  Fresh start!  0:00              │  ← if full credit (≥ 10 min break)
│                                  │
│       10:23  (counting up)       │
└──────────────────────────────────┘
```

Label auto-clears after 30 seconds (same timeout as transition banner).

### Feature: Walking / Away state shows neutral message

Currently unknown what's shown. Should display:
- State name: "Away" / "Walking"
- Time in state (counting up)
- Clarification: "Not counting as standing"

---

## Implementation Plan

### Step 1 — Rust: split `sitting_seconds` into two fields

In `session_types.rs` (after T027 split), add to `SessionState`:
```rust
/// Seconds in the current sitting session only (resets on Sitting entry, after break credit).
/// Use this for the session progress timer in the UI.
pub current_session_secs: i64,
```

Keep `sitting_seconds` as the daily accumulator (unchanged semantics).

In `session_manager.rs`, on transition to Sitting:
```rust
// Apply break credit to get the new session start value
let after_credit = apply_break_credit(self.state.sitting_seconds, standing_duration);
self.state.sitting_seconds = after_credit;      // daily accumulator (carry-over)
self.state.current_session_secs = after_credit; // session timer (what user sees)
```

Wait — on full credit (≥10 min), both reset to 0. On no credit, both carry the same value.
On partial credit, both carry the reduced value. So `current_session_secs = sitting_seconds`
after credit application. But at startup (`load_today_totals`), `sitting_seconds` is loaded
from DB (total today), while `current_session_secs` should start at 0 (fresh session).

```rust
// In load_today_totals():
self.state.sitting_seconds = db_total;      // correct: restore daily accumulator
self.state.current_session_secs = 0;        // correct: no active session yet
```

Add `current_session_secs` to `SessionStateDto`:
```rust
pub current_session_secs: i64,
```

### Step 2 — Rust: use StateChangedPayload fields added by T029

T029 adds `last_break_secs`, `last_sitting_secs`, `break_credit` to `StateChangedPayload`.
Verify they are populated correctly in `session_manager.rs` on state transitions.

### Step 3 — Frontend: fix SessionProgress timer source

In `useDesk.ts`, change the field used for the session timer:
```typescript
// Before:
setSittingSeconds(payload.sitting_seconds);   // ← was using daily accumulator

// After:
setSittingSeconds(payload.current_session_secs);  // ← current session only
```

Also in the polling path (`get_session_state` response), use `current_session_secs`.

### Step 4 — Frontend: add TransitionBanner component

New `src/components/TransitionBanner.tsx`:
```tsx
interface Props {
  lastBreakSecs: number;
  lastSittingSecs: number;
  breakCredit: 'none' | 'partial' | 'full' | null;
  transitionTo: DeskState | null;
}
```

- Auto-clears after 30s via `useEffect` + `setTimeout`
- Shows different messages for Sitting→Standing vs Standing→Sitting
- Shows break credit explanation

Wire into `App.tsx`, render above SessionProgress.

### Step 5 — Frontend: fix standing timer

Ensure `break_seconds` (or `standing_session_secs` after T028) counts up while Standing.
In `useDesk.ts`, extend `useTimer()` to tick for Standing state, not just Sitting.

### Step 6 — Frontend: Walking/Away state display

Add a section in `App.tsx` for Walking/Away states showing elapsed time + neutral message.

---

## Files to Modify

| File | Change |
|------|--------|
| `src-tauri/src/session_types.rs` | Add `current_session_secs` to `SessionState` + `SessionStateDto` |
| `src-tauri/src/session_manager.rs` | Populate `current_session_secs`, fix `load_today_totals` |
| `src-tauri/src/session_tests.rs` | Fix/pass tests written by T029 |
| `src/hooks/useDesk.ts` | Use `current_session_secs`, save `lastBreakSecs` from events, extend timer |
| `src/components/SessionProgress.tsx` | Use correct field for timer |
| `src/components/TransitionBanner.tsx` | NEW — 30s transition message |
| `src/App.tsx` | Wire TransitionBanner, add Walking/Away state section |

---

## Tests

- `test_current_session_secs_zero_after_full_break` — standing ≥10 min → current_session_secs=0
- `test_current_session_secs_continues_after_no_credit` — standing <5 min → carries over
- `test_current_session_secs_reduced_after_partial` — standing 5-9 min → reduced by 20 min
- `test_load_today_totals_does_not_set_current_session` — sitting_seconds loaded, current_session=0
- `TransitionBanner renders after Standing→Sitting` — shows "Stood for X min"
- `TransitionBanner auto-clears after 30s` — banner gone after timeout
- `SessionProgress uses current_session_secs not sitting_seconds` — correct field wired

---

## Acceptance Criteria

- [ ] Sitting timer shows 0:00 after a ≥10 min standing break
- [ ] Sitting timer continues from carry-over after <5 min break (with label)
- [ ] "Stood for X min" shown for 30s after Standing→Sitting transition
- [ ] "Sat for X min" shown for 30s after Sitting→Standing transition
- [ ] Break credit label shown ("fresh start", "session reduced", "session continues")
- [ ] Standing session timer counts up while standing (not frozen)
- [ ] Walking/Away state shows time + "not counting as standing"
- [ ] All T029 tests now pass
- [ ] `cargo test` + `pnpm test:unit` pass
