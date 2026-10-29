# ADR 008: Proportional Break Credit

- **Status**: accepted (revised 2026-09-06, E015)
- **Date**: 2026-03-30
- **Epic**: E000 (maintenance); revised by [E015](../../.plan/epics/E015-2026-09-06-engine-single-truth/PLAN.md)
- **Related**: [ADR 009](009-day-break-credit.md),
  [ADR 015](015-pure-ergo-engine.md)

> The formula below is unchanged by E020. What changed is where its inputs
> come from: `break_credit_multiplier` and `break_min_secs` are read from
> `SessionManager.limits` (refreshed from the ergonomic profile each tick),
> not from a copy inside `SessionState`, so a profile edit applies without a
> restart. This is one of three unrelated things called a "break" — see the
> module doc comment in `session_breaks.rs` and
> [ADR 015](015-pure-ergo-engine.md). Log prefix: `[break:credit]`.

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

An unexplained observation gap does not earn break credit. Preserve the known
observed prefix; the unobserved interval remains Unknown. As clarified by
[ADR 015](015-pure-ergo-engine.md), a trustworthy, qualified suspend/resume
signal could support separate credit, but the current engine has no such input.
The native UI diagnostics power adapter records diagnostic events only; it does
not authorize physiological credit or turn missing observations into a break.

## Alternatives

1. **Keep 3-tier system, fix sleep gap only**: simpler, but cliff effects remain
2. **Exponential credit (longer breaks worth more per minute)**: more realistic physiologically, but harder to explain to users and configure
3. **Fixed rate without minimum**: even 1 second of standing counts — too generous, jitter from state machine would accumulate false credit

## Consequences

- Smoother credit curve: short breaks are worth something, long breaks are worth proportionally more
- Configurable per profile: users can set multiplier 1.0 (slow reset) to 3.0 (fast reset)
- A known qualifying break reduces credited load; an unknown outage neither adds sitting time nor automatically subtracts load.
- Break credit is no longer "all or nothing" — users see gradual improvement

## Revision 2026-09-06 (E015)

Two changes. The proportional-credit decision stands; the historical sleep-gap
claim is superseded by the observed-time clarification below.

### One credited counter — no second "session" timer

The credited counter `SessionState.sitting_seconds`, exposed to every UI as
`SessionStateDto.limit_used_secs` (`limitUsedSecs` in TypeScript), is **the
only number a timer, progress bar, colour band or notification may read.**
It is reduced by the formula above and is never reset by a return to sitting.

Until E015 the popup timer and the overlay read `current_session_secs`, a
second counter zeroed on every entry into Sitting. The user saw the timer
drop to zero after a short break while the engine had credited only part of
it, and the same widget took its number from one field and its colour from
the other. That field is deleted from the state, the DTO and the payload
(E015 decision D2). The Debug tab — and nothing else — keeps
`secs_since_last_break`, named for what it actually measures.

Raw daily counters (`sitting_seconds_total`, `standing_seconds`) are for KPI
and reporting only. Compare a raw counter with another raw counter; never mix
a credited value with a raw one. The PostureBalance notification broke that
rule and could not fire for anyone who took breaks (E015-T03).

### Default multiplier per profile

`break_credit_multiplier` is per ergonomic profile, not one global default:

| Profile | Multiplier | Meaning |
|---------|-----------|---------|
| `standard` | **3.0** | 15 min break cancels 45 min of sitting |
| `default` | 2.0 | 10 min break cancels 20 min of sitting |
| `relaxed` | 2.0 | as `default` |
| `strict` | 2.0 | unchanged by E015 |
| `demo` | 2.0 | unchanged by E015 |

The struct fallback for a profile that omits the key stays 2.0
(`ergonomic_profile.rs`). Decision D1, Paweł, 2026-09-06 — recorded in
[`.plan/decisions.jsonl`](../../.plan/decisions.jsonl).

## Documentation reconciliation — 2026-10-27

The earlier decision and consequence claimed that every detected sleep gap
earned credit from its entire duration (for example, two hours at multiplier
2.0 clearing four hours of load). That claim is retained here as superseded
history, not current runtime behavior. A long reading gap alone cannot establish
machine sleep or human absence.

Current `session_reading.rs` preserves the observed prefix across gaps over
300 seconds without crediting the gap; `session_tests_sleep.rs` explicitly
covers unknown-gap exclusion and credit for the known break prefix only.
`session_live.rs` bounds projections to actual observations, and restart
restoration clears live coverage anchors. The conditional trusted-suspend
contract in ADR 015 remains unchanged. `ui_diagnostics_power.rs` forwards real
OS power notifications to UI diagnostics, not to the ergonomic engine.
This reconciles documentation with existing behavior; it changes no policy,
formula, profile, or runtime implementation.
