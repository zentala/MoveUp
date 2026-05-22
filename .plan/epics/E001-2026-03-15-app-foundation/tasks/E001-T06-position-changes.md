---
id: E001-T06
epic: E001
status: completed
original_id: "0005"
title: T005 — Track `position_changes` Counter in SessionState
---
# T005 — Track `position_changes` Counter in SessionState

**Priority**: P2
**Status**: open

## Goal
Count how many times the user has transitioned between Sitting and Standing/Walking
during the current session. Shown as "Ilosc zmian: N" in ErgoTrackDesktop.

## Scope

### `session.rs`
- Add `position_changes: u32` to `SessionState` (persists across the day)
- Increment in `apply_height()` on every confirmed state transition
  (after debounce, not on every raw reading)
- Reset daily — add `reset_daily()` called at midnight or on app start for a new day

### `SessionStateDto`
- Add `position_changes: u32` field

### `StateChangedPayload` (types.ts)
- Add `position_changes: number` field

### `useDesk` hook
- Expose `positionChanges` from the event payload

### `TodayStats.tsx`
- Add "changes: N" stat next to sitting/standing times

## Tests
- Unit: counter increments on Sitting->Standing, Standing->Sitting
- Unit: counter does NOT increment on Standing->Walking (same break type)
- Unit: counter resets at midnight

## Acceptance criteria
- [ ] `TodayStats` shows position change count
- [ ] Count survives HMR reload (backed by in-memory state from Rust)
- [ ] Correct count shown in `get_today_summary` response (T007)
