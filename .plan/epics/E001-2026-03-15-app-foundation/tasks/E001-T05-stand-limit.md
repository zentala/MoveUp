---
id: E001-T05
epic: E001
status: completed
original_id: "0004"
title: T004 — Stand Reminder: Configurable "Sit Down After X Min" Limit
---
# T004 — Stand Reminder: Configurable "Sit Down After X Min" Limit

**Priority**: P2
**Status**: open
**Depends on**: T002

## Goal
Mirror the "Remind me to sit down after X minutes" slider from the ErgoTrackDesktop
settings. Currently we only track sitting session limit; standing has no upper bound.

## Behaviour spec
- When `state == Standing` and `break_seconds >= stand_limit_mins * 60`:
  - Fire notification: "You've been standing for {N} minutes. Ready to sit?"
  - Fire at most once per standing session (debounce: reset flag on state change)

## Scope

### `session.rs`
- Add `stand_limit_secs: i64` to `SessionState`
- Add `stand_alert_fired: bool` — reset to `false` on every state transition
- In `apply_break_tick()`: check `break_seconds >= stand_limit_secs && !stand_alert_fired`
  -> emit notification + set flag

### `commands.rs`
- `set_stand_limit(minutes: u32)` command (or fold into `save_settings` from T002)

### `SessionStateDto`
- Expose `stand_limit_secs` so the frontend progress bar could optionally show
  standing progress too (out of scope for now — just expose the field)

## Tests
- Unit: alert fires exactly once per standing session
- Unit: alert does not fire when `stand_limit_secs == 0` (disabled)
- Unit: resumes correctly after sit -> stand -> sit -> stand

## Acceptance criteria
- [ ] Notification fires when standing exceeds configured limit
- [ ] Fires only once per standing session
- [ ] Setting `stand_limit_mins = 0` disables the reminder entirely
