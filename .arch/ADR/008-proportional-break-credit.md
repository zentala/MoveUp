# ADR 008: Proportional Break Credit

- **Status**: accepted
- **Date**: 2026-03-30
- **Epic**: E000 (maintenance)

## Context

The original break credit system used 3 fixed tiers:
- Break < 5 min → no credit
- Break 5-10 min → subtract 20 min from sitting timer
- Break ≥ 10 min → full reset (sitting timer = 0)

Problems:
1. Cliff effects: 4:59 break = zero credit, 5:00 = 20 min credit
2. Not configurable: thresholds hardcoded as constants
3. Sleep gap bug: when computer slept for hours, the gap was detected but break credit was NOT applied — sitting timer preserved its full value after waking up

## Decision

Replace with proportional break credit:
- Each second of break cancels `break_credit_multiplier` seconds of sitting (default: 2.0)
- Breaks under `break_min_secs` (default: 60s) get no credit
- Both parameters configurable in the ergonomic profile JSON

Formula: `new_sitting = max(0, old_sitting - break_secs × multiplier)`

Sleep gaps (machine suspend/resume) now apply break credit using the gap duration.

## Alternatives

1. **Keep 3-tier system, fix sleep gap only**: simpler, but cliff effects remain
2. **Exponential credit (longer breaks worth more per minute)**: more realistic physiologically, but harder to explain to users and configure
3. **Fixed rate without minimum**: even 1 second of standing counts — too generous, jitter from state machine would accumulate false credit

## Consequences

- Smoother credit curve: short breaks are worth something, long breaks are worth proportionally more
- Configurable per profile: users can set multiplier 1.0 (slow reset) to 3.0 (fast reset)
- Sleep gap now properly resets sitting timer (2h sleep × 2.0 = 4h credit → full reset)
- Break credit is no longer "all or nothing" — users see gradual improvement
