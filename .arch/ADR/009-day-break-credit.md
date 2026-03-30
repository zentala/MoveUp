# ADR 009: Day Break Credit and PostureBalance Minimum

- **Status**: accepted
- **Date**: 2026-03-30
- **Epic**: E000 (maintenance)

## Context

ADR 008 introduced proportional break credit for session-level resets.
However, notification flags (`notify_posture_balance_fired`, etc.) only
reset at midnight. After an overnight sleep or long lunch, the PostureBalance
notification fires immediately because:

1. `sitting_seconds > standing_seconds * 2` triggers with any sitting when
   standing is 0 — even after a 2-hour break.
2. The flag was already fired earlier and won't re-fire, OR it hasn't fired
   yet and triggers after minimal sitting — but "most of today" is misleading
   after only 2 hours.
3. `daily_score` carries over from before the break, making the scoring
   feel disconnected from the user's current session.

## Decision

### Two levels of break credit

**Level 1 — Session Break Credit** (existing, ADR 008):
- Resets `sitting_seconds` proportionally: `credit = break_secs * multiplier`
- Default: 2.0 multiplier, 60s minimum
- Purpose: encourage short breaks (lunch ~1.5h resets session timer)

**Level 2 — Day Break Credit** (new):
- Threshold: `day_break_min_secs` (default: 21600 = 6 hours)
- When `apply_break_credit()` detects a break >= threshold, also reset:
  - `notify_inactivity_fired`
  - `notify_posture_balance_fired`
  - `praise_halfway_fired_today`
  - `standing_target_reached_fired`
  - `daily_score` to 0.0
- Does NOT reset daily KPI counters (`sitting_seconds_total`, `standing_seconds`,
  `position_changes`, `hourly_breaks_*`) — those stay for accurate daily reporting
- Rationale: a 6-hour break (sleep) means a fresh motivational start. You can't
  say "sitting most of today" after someone slept for 6 hours.

### PostureBalance minimum sitting guard

New field: `posture_balance_min_sitting_secs` (default: 21600 = 6 hours).
PostureBalance only fires when both conditions are met:
- `sitting_seconds_total >= posture_balance_min_sitting_secs`
- `sitting_seconds > standing_seconds * 2`

This prevents "sitting most of today" after only 2 hours of sitting.

### Notification message

Replaced vague "You've been sitting most of today" with concrete numbers:
`"Sitting Xh Ym vs standing Xh Ym"` — actionable, not accusatory.

## Alternatives

1. **Reset flags at fixed intervals (every 4h)**: simpler but arbitrary,
   doesn't correlate with actual breaks.
2. **Use sitting_seconds for the guard**: sitting_seconds is reduced by
   break credit so could be 0 even after hours of sitting.
   `sitting_seconds_total` is the correct raw metric.
3. **Separate "day reset" from break credit**: cleaner separation but
   `apply_break_credit` already runs on every gap detection, so
   piggy-backing avoids a second detection path.

## Consequences

- After overnight sleep, user gets a clean motivational slate
- PostureBalance won't nag after short sitting periods
- Notification messages provide actionable data
- Two new configurable fields in ergonomic profiles; serde defaults
  ensure backward compatibility with existing profile files
- Midnight daily reset still works as before (resets everything)
