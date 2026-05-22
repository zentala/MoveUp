---
id: E005-T09
epic: E005
status: completed
created: 2026-03-21
completed: 2026-03-22
original_id: T030
title: T030 — Floating Window: Fix Bugs + Show Transition Info
---
# T030 — Floating Window: Fix Bugs + Show Transition Info

**Status:** open
**Priority:** P1
**Depends on:** T029 (spec + tests + StateChangedPayload extended), T027 (session.rs split)
**Branch:** feat/T030-floating-window-fix

---

## Bugs to Fix

1. Sitting timer shows wrong value on session start (sitting_seconds conflates current session with daily total)
2. No "Stood for X min" shown after Standing->Sitting transition
3. Standing session timer not shown while standing

## Key Fix

Add `current_session_secs: i64` to `SessionState` that resets to 0 on every Sitting state entry.
Keep `sitting_seconds` as the daily accumulator for TodayStats.

---

## Acceptance Criteria

- [ ] Sitting timer shows 0:00 after a >= 10 min standing break
- [ ] "Stood for X min" shown for 30s after transition
- [ ] Standing session timer counts up while standing
- [ ] All T029 tests now pass
- [ ] `cargo test` + `pnpm test:unit` pass
