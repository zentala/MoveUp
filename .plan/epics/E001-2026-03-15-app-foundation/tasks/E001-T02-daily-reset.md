---
id: E001-T02
epic: E001
status: completed
original_id: "0009"
title: T009 — Daily Reset Logic
---
# T009 — Daily Reset Logic

**Priority**: P1
**Status**: open
**Depends on**: T002 (Rust owns truth, SQLite seed on startup)

## Problem
`sitting_seconds` and `standing_seconds` accumulate forever in `SessionManager`.
If the app runs overnight, the next day starts with yesterday's totals. `TodayStats`
shows the correct SQL aggregate (filtered to today), but `SessionProgress` shows an
incorrect inflated counter from in-memory state.

## Scope

### `session.rs`
Add `last_reset_date: chrono::NaiveDate` to `SessionManager`.

In the serial reader tick (or a dedicated 60s timer), call `check_daily_reset()`:
```rust
pub fn check_daily_reset(&mut self) {
    let today = Utc::now().date_naive();
    if self.last_reset_date < today {
        log::info!("daily reset: new day detected, resetting in-memory counters");
        self.state.sitting_seconds  = 0;
        self.state.standing_seconds = 0;
        self.state.position_changes = 0; // (once T005 exists)
        self.alert_fired            = false;
        self.stand_alert_fired      = false;
        // Notification debounce flags from T003:
        self.notify_inactivity_fired        = false;
        self.notify_posture_balance_fired   = false;
        self.praise_halfway_fired_today     = false;
        self.last_reset_date = today;
        // Re-seed from SQLite (today is a new day, so 0s is correct — no seed needed)
    }
}
```

### `lib.rs` / startup
```rust
// Init last_reset_date to today so we don't immediately reset on first launch
session_manager.last_reset_date = Utc::now().date_naive();
```

### `commands.rs`
`check_daily_reset()` is called in the serial reader background thread loop (every reading),
or hoisted into a separate 60s Tokio interval task if the sensor disconnects overnight.

## Edge cases
- App started on day N, runs to day N+1: midnight check triggers reset → counters go to 0 → correct
- App restarted on day N+1: `load_today_totals()` from SQLite seeds 0 (new day has no rows) → correct
- App started at 23:59, reset fires at 00:00: brief 1-min session before reset → persisted correctly if session row written on transition

## Tests
- Unit: `check_daily_reset()` resets counters when `last_reset_date < today`
- Unit: `check_daily_reset()` does NOT reset when called multiple times same day
- Unit: all notification flags reset on daily reset
- Unit: `last_reset_date` advances to new day after reset

## Acceptance criteria
- [ ] Running app overnight: next morning shows 0s sitting (not yesterday's total)
- [ ] `TodayStats` and `SessionProgress` agree on session time after midnight
- [ ] All notification debounce flags cleared at midnight
