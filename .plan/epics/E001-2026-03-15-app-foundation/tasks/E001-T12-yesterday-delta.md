---
id: E001-T12
epic: E001
status: completed
completed: 2026-03-21
original_id: "0012"
title: T012 — Yesterday Delta Arrow in TodayStats
---
# T012 — Yesterday Delta Arrow in TodayStats

**Priority**: P3
**Status**: open
**Depends on**: T002 (Rust owns truth, `get_today_summary` is a real command)

## Goal
Show a up or down indicator next to today's sitting time, comparing to yesterday's total.
Motivating context: "am I doing better or worse than usual?"

## Scope

### `commands.rs` — extend `get_today_summary`
Add `yesterday_sitting_secs: i64` and `yesterday_standing_secs: i64` to `TodaySummary`.
Query: `SELECT SUM(duration_seconds) FROM sessions WHERE state='Sitting' AND date(started_at) = date('now', '-1 day')`

### `TodaySummary` struct (`db.rs`)
```rust
pub struct TodaySummary {
    pub sitting_secs: i64,
    pub standing_secs: i64,
    pub yesterday_sitting_secs: i64,
    pub yesterday_standing_secs: i64,
    pub sessions: Vec<SessionRow>,
}
```

### `TodayStats.tsx`
```tsx
const sitDelta = summary.sitting_secs - summary.yesterday_sitting_secs;
// Show down-arrow (green) if sitting less, up-arrow (amber) if sitting more
const deltaSymbol = sitDelta < -300 ? "down" : sitDelta > 300 ? "up" : "";
const deltaClass  = sitDelta < -300 ? "delta--better" : "delta--worse";
```

Threshold: +/-5 minutes (300s) before showing arrow — avoids noise for near-equal days.

### `globals.css`
```css
.delta--better { color: var(--signal-ok);   font-size: 11px; }
.delta--worse  { color: var(--signal-warn);  font-size: 11px; }
```

## Tests
- Unit (Rust): `get_today_summary` returns 0 for yesterday if no sessions
- Vitest: down-arrow shown when sitting_secs < yesterday_sitting_secs - 300
- Vitest: up-arrow shown when sitting_secs > yesterday_sitting_secs + 300
- Vitest: no arrow shown when difference < 300s

## Acceptance criteria
- [ ] Down-arrow (green) shown when sitting less than yesterday
- [ ] Up-arrow (amber) shown when sitting more than yesterday
- [ ] No arrow on first day of use (yesterday = 0, delta is misleading)
- [ ] No arrow when difference < 5 minutes
