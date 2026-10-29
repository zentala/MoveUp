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

## Accepted core reliability continuation — 2026-10-21

The owner requested implementation of the complete core-logic review with proportional architectural refactoring. Retain the existing pure-core decision and migrate in tested slices; do not replace the application with a new event framework or shared package. The owning continuation is the existing E027 plan.

### Responsibility boundaries

`observation adapter -> engine/metrics -> communication policy -> channel executors`

`engine -> versioned persistence/history -> wire adapter -> shared UI model -> rendering`

- **Observation adapter (stable):** owns hardware/input activity and captures time/connection quality. Absence of a reading is not proof of a posture or an Away interval. No UI or third-party types enter engine rules.
- **Engine (stable):** owns state transitions, one operation to close an observed interval at an explicit boundary, raw totals, and a separate exactly-once credit calculation. State transition, long gap and local midnight use the same boundary semantics. A gap must not erase the last observed interval or grant the same seconds twice.
- **Metrics (stable):** owns definitions/units and local calendar boundaries. Hour coverage counts overlap with each hour rather than carrying a cumulative threshold into an unrelated bucket. Stabilization measures the declared elapsed time as well as coherent samples.
- **Communication policy (stable):** owns whether/when/what to communicate, including device warnings, periodic notices, dismiss/snooze, profile reload and repeat schedule. One resolved intent is dispatched to the selected channels, log and optional webhook; adapters do not independently re-detect the same limit. Use an injected clock for deterministic policy tests.
- **Executors (stable):** own side effects and transport errors, not business thresholds. Preserve native tray main-thread handle lifetime and existing appearance contract. Profile Silent suppresses notification intents across producers; diagnostic state may remain visible without an unsolicited notification.
- **Persistence (stable):** owns versioned restart state and acknowledged history writes. Open observations, notification state and last successful checkpoint have explicit restore semantics; persistence errors stay errors and pending history is not silently discarded. SQLite raw history and Store credited/manager state retain their existing precedence.
- **Wire adapter (stable):** translates domain outputs without inventing values. Preserve raw versus credited sitting, recorded credit, signed score, local date/timezone provenance, connection/quality, exact-versus-legacy history and producer event counter. Unknown and null are not zero, Sitting or Away.
- **UI (stable):** renders one normalized chronology/day model in compact/detail forms and full local-day compositions. It owns interaction/loading/error presentation, not reconstructing a missing business counter or extending observations through arbitrary gaps.

No neutral shared package is justified: these are MoveUp concepts with existing in-product consumers. Keep the helpers in their current domain modules; extract a small local primitive only when it removes duplicated responsibility, not for speculative reuse.

### Behavioral decisions for uncertain cases

1. Preserve the observed prefix up to the last valid sample before any gap. The gap itself remains Unknown. Preserve ADR008 proportional credit for a known suspend/resume or confirmed qualifying break; an unexplained USB/observation outage alone is not sufficient evidence. If an existing runtime supplies no trustworthy suspend signal, do not invent one: retain Unknown/no automatic gap credit and document that compatibility change with tests.
2. Make the existing ADR010 maximum-four configurable reminder contract executable with an injected clock; preserve visual escalation and escalating silence rather than creating another competing notification type. Do not automatically increase reminders beyond the configured schedule. Missing/invalid channel values fail profile validation; `None` means no notification.
3. Persist an active snooze and exhausted/repeat state using bounded, versioned restart-compatible data. Live policy uses a monotonic/injected clock; persisted wall-clock deadline is validated on restore so clock rollback or stale dates cannot create indefinite silence or uncontrolled replays. Keep compatibility with records without these optional fields.
4. Bound abrupt-loss exposure using an ordinary periodic checkpoint at the existing minute cadence, plus current event/graceful-stop saves. Preserve observed/acknowledged history without filling stopped time. Define and test at-most-one-cadence loss under a functioning store; do not promise durability during IO failure or forced termination before a checkpoint. No live historical backfill or destructive migration.

### Compatibility and recovery

Additive fields default to unknown when absent. Keep legacy mixed rows visibly uncertain; only the new producer marks exact individual intervals. A nullable additive discriminator was already approved; do not relabel or UPDATE old rows. Reuse the existing generated bindings/exporter and current adapters. Validate older profiles/Store snapshots, nullable fields and malformed inputs with disposable fixtures. Preserve backups before installation and restore only through the owning recovery procedure; no manual live-row cleanup. Keep the reviewed previous executable/config for rollback. Unknown future data versions must not be rewritten by an older reader.

Each slice starts with a regression that discriminates the observed defect, then required checks and independent review. UI acceptance additionally requires actual-host images under DESIGN.md. Ordinary forceful Windows supervisor stop remains outside a guarantee of cooperative flush; it is not fixed by changing metric/policy semantics. Existing access refusals and machine-setting holds remain binding.

## Accepted ongoing observation display projection — 2026-10-25

E028 extends the existing pure-engine/wire/UI boundary, not the history or credit model. The engine provides a coherent read-only projection of the currently confirmed observation span, actual last sample, snapshot instant and source-qualified quality/reason, using history_span_started and known_projection_end(now). A freshness threshold can classify an old reading but cannot create covered duration after it. Preserve confirmed prefixes while current input is stale; gap/restart confirmation never joins observations across an outage. Snapshotting mutates no history/counters/policy.

Existing application adapters carry the additive nullable metadata through the actual Rust-generated DTO; absent older data stays unavailable. Transport receipt/error/order is separate from sensor sample freshness and physical connection. The shared local chronology clips/deduplicates recorded and current confirmed spans for popup/Analyst, preserving exact Unknown/Future and source-qualified local-day bounds. UI owns explanations/status, never deriving coverage from counters, browser time, cached height or an open row alone. No neutral package/new transport/schema/retrofill is needed. Existing ADR014 persistence precedence and ADR017 exporter ownership remain. Actual nullable names are recorded before consumer edits in the owning epic JOURNAL; compatibility and malformed/out-of-order input tests are required.
