---
id: E001-T09
epic: E001
status: completed
completed: 2026-03-23
original_id: "0007"
title: T007 — Add `position_changes` to DB Schema and TodaySummaryDto
---
# T007 — Add `position_changes` to DB Schema and TodaySummaryDto

**Priority**: P3
**Status**: open
**Depends on**: T005, T006

## Goal
Persist position change count in SQLite so it survives app restarts and
is included in the full daily summary.

## Scope

### `db.rs`
- Add `position_changes INTEGER NOT NULL DEFAULT 0` column to `sessions` table
  (migration: `ALTER TABLE sessions ADD COLUMN ...` guarded by `IF NOT EXISTS` check)
- Update `TodaySummary` struct: add `position_changes: u32`
- Add query: `SELECT SUM(position_changes) FROM sessions WHERE date(started_at) = date('now')`

### `commands.rs`
- `get_today_summary()`: run the new aggregation query via `tauri-plugin-sql`
  (currently this is a placeholder — this task replaces it with a real DB query)

### `types.ts`
- `TodaySummaryDto`: add `position_changes: number`

### `TodayStats.tsx`
- Read `summary.position_changes` (T005 already wires the live counter; this adds
  persistence across restarts)

## Tests
- Integration: insert session rows with known `position_changes`, verify summary sum
- Unit: migration guard doesn't fail on fresh schema

## Acceptance criteria
- [ ] After app restart, position change count for today is preserved
- [ ] Schema migration runs without errors on existing DB
