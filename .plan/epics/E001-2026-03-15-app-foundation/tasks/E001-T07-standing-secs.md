---
id: E001-T07
epic: E001
status: done
original_id: "0006"
---
# T006 — Fix `standing_secs` in TodaySummary (remove placeholder 0)

**Priority**: P2
**Status**: open

## Problem
`commands.rs:get_today_summary()` returns `standing_secs: 0` — a known placeholder.
The `TodayStats` component shows "stood 0s" regardless of how long the user has stood.

## Root cause
`SessionManager::snapshot()` only tracks `sitting_seconds`. Standing/break time is
accumulated in `break_seconds` but the summary command ignores it.

## Scope

### `session.rs`
- Add `standing_seconds: i64` to `SessionState`
  - Incremented only when `state == Standing` (not Walking or Away)
- Add `walking_seconds: i64` similarly (optional but clean)
- Expose both in `SessionStateDto`

### `commands.rs`
- `get_today_summary()`: use `snap.standing_seconds` instead of `0`

### `types.ts`
- `SessionStateDto`: add `standing_seconds: number`
- `TodaySummaryDto`: `standing_secs` was already present, now actually populated

### `TodayStats.tsx` — no change needed (already reads `standing_secs`)

### `useDesk` hook — expose `standingSeconds` alongside `sittingSeconds`

## Tests
- Unit: `standing_seconds` increments only during `Standing` state
- Unit: `break_seconds` still includes Walking + Away (unchanged behaviour)

## Acceptance criteria
- [ ] `TodayStats` shows non-zero standing time after user has stood
- [ ] Sitting + Standing + Walking totals are plausible (<= uptime)
