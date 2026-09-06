# ADR 015: The session engine is a pure core; adapters own the clock and the UI

- **Status**: accepted
- **Date**: 2026-09-06
- **Epic**: E020 (engine: pure core) — tasks T01–T06
- **Related**: [ADR 008](008-proportional-break-credit.md),
  [ADR 009](009-day-break-credit.md),
  [ADR 011](011-unified-sit-stand-walk-cycle.md),
  [ADR 014](014-persistence-precedence.md),
  [E015](../../.plan/epics/E015-2026-09-06-engine-single-truth/PLAN.md),
  [E020](../../.plan/epics/E020-2026-09-06-engine-pure-core/PLAN.md)

## Context

E015 made one credited counter the only number a user-facing surface reads.
That fixed the counter, not the seams around it. Four of them were still
open, and each had already produced a real bug:

- **The engine read the wall clock itself.** `on_reading()` called
  `Utc::now()` inside; so did the daily-reset and notification checks, while
  the sibling `handle_state_exit()` already took `now` as a parameter. Every
  duration test backdated `Utc::now() - Duration::seconds(N)` and hoped the
  real clock did not tick between setup and assertion, so a midnight or DST
  edge could not be constructed at all.
- **The daily reset compared UTC calendar days** while `session_persistence.rs`
  and `db_sessions.rs` keyed off the local day. East or west of UTC the
  in-memory reset fired hours away from the persisted "today". No test caught
  it, because no test could pick the instant.
- **Configuration lived inside state.** Six `ErgonomicProfile` fields were
  copied field-by-field into `SessionState` at construction and read back off
  `self.state.*`. `profile_reload.rs` updated `CommunicationPolicy`'s copy and
  never called back into `SessionManager`, so hot-reloading an ergonomic
  profile silently stopped affecting break credit, PostureBalance and the
  computer-time reset until the app restarted.
- **`tray_controller.rs` re-derived what the engine already knew.**
  `compute_standing_lap()` and an `elapsed_secs` match recomputed
  session-derived numbers from a fresh snapshot on every ~1 s tick, so a
  second consumer could disagree with the first without anything failing.

Underneath all four is one rule that was never written down: **who owns
time, configuration, and derived values.**

## Decision

The session engine (`SessionManager` and the `session_*.rs` modules) is a
pure core. Four rules, each enforced by the shape of the code rather than by
convention:

1. **The caller owns the clock.** Every stepping function has an `_at`
   variant that takes `now: DateTime<Utc>` and reads no clock:
   `on_reading_at`, `needs_daily_reset_at`, `check_daily_reset_at`,
   `check_notification_conditions_at`, `accumulate_ongoing(now)`,
   `handle_state_exit(.., now)`, `policy_input(now, ..)`. The zero-argument
   names that remain (`on_reading`, `check_daily_reset`, …) are one-line
   convenience wrappers that call `Utc::now()` once and delegate — they are
   the documented impure boundary, alongside the real adapters
   (`serial_periodic.rs`, `commands.rs::inject_reading`), which capture `now`
   once per tick and thread it down. Tests call the `_at` form and never race
   the wall clock.

2. **Calendar days are local days.** A day boundary is derived from the
   injected instant's *local* calendar date, matching
   `session_persistence.rs` and `db_sessions.rs`. `needs_daily_reset_at`
   and `check_daily_reset_at` take that date as an explicit second argument,
   so the timezone conversion happens once, at the adapter, and is visible in
   the signature rather than hidden in the body. (Decision D3.)

3. **Configuration is held once, as configuration.** `SessionState` holds no
   profile fields. `SessionManager.limits: Limits` is the single copy of the
   ergonomic limits in force, refreshed from the profile the serial loop
   already reloads each tick (`set_limits` / `set_ergo_profile`). A profile
   edit takes effect on the next tick, with no restart. State is what
   happened; limits are what the user configured; the two no longer share a
   struct.

4. **The engine computes every policy-facing derived value.**
   `SessionManager::policy_input(now, sensor_connected)` builds the whole
   `PolicyInput` — `elapsed_secs`, `standing_lap`, `standing_lap_progress`,
   `standing_lap_flash` included. Adapters pass in only what the engine
   cannot know (sensor connectivity) and never recompute a value from a
   snapshot. `PolicyInput`'s shape did not change; only who fills it in did.

Two supporting consequences of the same rule:

- **Persistence is one versioned snapshot derived from the state**, not a
  hand-mirrored twin. `PersistedEngineState { schema_version, .. }` wraps
  `SessionState` plus the manager-level flags and `HourlyBreakTracker`, and
  a v0 (unversioned) file is migrated on load. Adding a persisted field is
  one edit, not two kept in sync by hand — see
  [ADR 014](014-persistence-precedence.md) for which store owns what.
- **"Break" names three separate things**, and the module doc comment in
  `session_breaks.rs` says so: Session Break Credit (`[break:credit]`,
  ADR 008), Day Break Credit (`[break:day]`, ADR 009), and hourly break
  coverage (`[break:hourly]`, a KPI). They are not merged; they are
  labelled, so a log line tells you which one acted.

## Alternatives

1. **Patch only the two live bugs** (hot reload, UTC-vs-local day) and leave
   the seams. Cheapest, and it leaves every mechanism that produced them
   intact — the next feature reopens one, exactly as the nine break-credit
   rewrites in March did.
2. **Full rewrite to `ErgoEngine::step(event, cfg, now) -> Vec<Signal>`.**
   Cleanest on paper, and it discards the 700-test safety net mid-migration
   while inventing a second "event" vocabulary — the very bug class this ADR
   exists to close. `communication_types::Signals` already provides most of
   what the new enum would add.
3. **Chosen: strangler steps.** Each seam closed as its own task, with
   `cargo test --lib` green at every step.

## Consequences

- Duration, midnight-rollover and DST behaviour are testable by construction;
  `session_tests_clock.rs` and `session_tests_scenario_table.rs` assert
  against the DTO the user's screen renders, not against internal helpers.
- Hot-reloading an ergonomic profile now changes engine behaviour on the next
  tick. Anything that previously relied on limits being frozen at
  construction no longer holds.
- `tray_controller.rs` contains no derivation logic. `CLAUDE.md`'s
  description of the tray path changes accordingly: the engine computes the
  policy inputs, `TrayController` routes, `tray_signal_exec.rs` executes.
- The zero-argument wrappers are a deliberate, documented exception to
  "the engine never reads the clock". They keep call sites that genuinely
  have no `now` in hand short; adding logic to one of them, rather than to
  its `_at` twin, reintroduces the untestable path.
- Adding a new derived value that a UI needs means extending `PolicyInput`
  and `policy_input()`, never computing it in an adapter.
