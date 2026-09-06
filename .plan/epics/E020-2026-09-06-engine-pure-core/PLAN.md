---
formatVersion: 1
type: epic
status: todo
readiness: ready
points: 59
agent: ts-dev
wave: 4
parallel: []
depends-on: [E015, E017]
blocked-by: ""
---

# E020 — Engine: pure core (migration steps 4-8)

Source review: [`../../reports/_review-2026-09-06/engine.md`](../../reports/_review-2026-09-06/engine.md)
§Target architecture and migration steps. Synthesis:
[`../../reports/2026-09-06-pelny-przeglad-architektury-i-release.md`](../../reports/2026-09-06-pelny-przeglad-architektury-i-release.md)
§3 Silnik, §6 E020. Depends on
[`../E015-2026-09-06-engine-single-truth/PLAN.md`](../E015-2026-09-06-engine-single-truth/PLAN.md)
(steps 1-3, one credited counter) having merged first. Board deck (Polish):
[`PRES.md`](PRES.md). Handoff: [`HANDOFF.md`](HANDOFF.md).

## TLDR

E015 made the credited sitting counter the only number any UI reads. This
epic finishes the other five migration steps `engine.md` names: the clock
stops being read from inside the engine (`Utc::now()` calls move to the
adapters that already own a tick), the six ergonomic-profile fields
`SessionState` currently copies field-by-field stop existing as state (they
become a borrowed `&ErgonomicProfile`), the hand-mirrored
`PersistedSessionState` becomes a versioned, derived snapshot of the real
state, `tray_controller.rs` stops re-deriving standing-lap progress and
`elapsed_secs` from a fresh snapshot every tick, and "break" — three
unrelated things sharing one English word today — gets one naming
convention. A dedicated scenario-table test closes the gap the review
found: existing tests assert the helper field, never the path the user
sees. 8 sequential/parallel tasks, 59 points across 7 waves, each wave
green on `cargo test --lib`.

## Problem

Findings from `engine.md` §Model inconsistencies, all still present after
E015 lands (E015 only touches the counter identity, not these seams):

- **Engine reads the wall clock itself.** `on_reading()` calls `Utc::now()`
  internally (`session_reading.rs:12`); so do `check_daily_reset()` /
  `needs_daily_reset()` (`session_daily.rs:13,23`) and
  `check_notification_conditions()` (`session_breaks.rs:170`). Sibling
  method `handle_state_exit()` already takes `now` as a parameter
  (`session_breaks.rs:14`) — the module is internally inconsistent about
  who owns time. Every test that needs a specific elapsed duration
  backdates `Utc::now() - Duration::seconds(N)` and hopes the real clock
  doesn't tick between setup and assertion
  (`session_tests_break_credit.rs:33-34,60-61,86-87,113-114,131-132`;
  the same pattern repeats in `session_tests_sleep.rs`,
  `session_tests_day_break.rs`, `session_tests_daily.rs`,
  `session_tests_timers.rs`, `session_tests_timers_live.rs`,
  `session_tests_scenarios.rs`, `session_tests_scenarios_adv.rs`,
  `session_tests_away.rs`, `session_tests_away_transitions.rs`).
- **A new bug found while reading this code for the epic**: the daily
  reset compares `Utc::now().date_naive()` to `last_reset_date`
  (`session_daily.rs:14,24`) — a UTC calendar day — while every other
  date-keyed surface in the same app (`session_persistence.rs:34,66,114`,
  `db_sessions.rs:24`) keys off `chrono::Local::now()`. For a user east or
  west of UTC, the in-memory daily reset fires at UTC midnight, not local
  midnight — several hours off from the persisted/DB "today". No test
  catches this because every existing daily-reset test runs against
  whatever `Utc::now()` happens to be at CI time, never a constructed
  clock. This is folded into T01 because injecting the clock is exactly
  where the fix belongs (compare the injected `now`'s local calendar day,
  not its UTC one) and the task already asks for a midnight-rollover test.
- **Config lives inside state.** `break_min_secs`, `break_credit_multiplier`,
  `day_break_min_secs`, `posture_balance_min_sitting_secs`,
  `max_continuous_computer_secs`, `computer_break_reset_secs` are all
  copied field-by-field from `ErgonomicProfile` into `SessionState` at
  construction (`session_manager.rs:71-76,136-141`) and read back off
  `self.state.*` inside the engine (`session_breaks.rs:127-128,145,183`;
  `session_reading.rs:197`). **`profile_reload.rs` never re-syncs them**:
  `reload_profiles_if_changed()` calls `comm_policy.set_ergo_profile(...)`
  (`profile_reload.rs:96-105`), which updates
  `CommunicationPolicy`'s own copy, but nothing calls back into
  `SessionManager` — hot-reloading the ergonomic profile silently stops
  affecting break credit, PostureBalance, and the computer-time reset
  until the app restarts. This is a real, previously-undocumented bug,
  not a hypothetical one; it disappears once these six fields are read
  from a borrowed `&ErgonomicProfile` passed in per call, because the
  serial loop already reloads and holds the current profile
  (`serial_periodic.rs:32`) every tick.
- **Persistence hand-mirrors the state instead of deriving from it.**
  `PersistedSessionState` (`session_persistence.rs:78-107`) is a second,
  parallel struct with no `schema_version`; `SessionState` itself has no
  `Serialize`/`Deserialize` derive at all. Every new persisted field must
  be added in two places and kept in sync by hand — the exact failure
  mode that produced nine break-credit rewrites in March (per
  `plans-and-history.md` §2, cited in E015's `PLAN.md`).
- **`tray_controller.rs` re-derives, it doesn't consume.**
  `compute_standing_lap()` (`tray_controller.rs:154-164`) and the
  `elapsed_secs` match (`tray_controller.rs:124-128`) recompute
  session-derived numbers from a fresh `snapshot()` on every ~1s tick,
  duplicating knowledge the engine already has about which field means
  what per state. `CLAUDE.md`'s own Communication Architecture section
  says "TrayController only executes signals — no decision logic" — this
  is the place code does not yet match that stated architecture.
- **"Break" means three things.** `BreakCredit`
  (session-level proportional credit, `session_breaks.rs:126-158`),
  `HourlyBreakTracker` (per-clock-hour boolean coverage KPI,
  `hourly_break_tracker.rs`), and the Day Break Credit special case
  (notification-flag/score reset nested inside `apply_break_credit`,
  `session_breaks.rs:142-157`) share the English word "break" and nothing
  else — no shared type, no shared naming convention, no doc section that
  says there are three.
- **Test coverage verdict from `engine.md`**: "tests exist but test the
  wrong things" — coverage exercises the internal field, not the DTO path
  a user's screen renders. E015 fixes the *counter* half of that
  complaint; this epic's T06 is the general fix — one scenario-table test
  that asserts DTOs, not helpers, across the code paths this epic
  actually touches (clock, midnight, sleep gap, DST).

## Decisions and ADRs

- ADR 008 (proportional break credit) and ADR 009 (day break credit) —
  unaffected in substance; T07 adds a cross-reference from both to the new
  ADR, since T03 renames the types they describe without changing the
  formulas.
- ADR 011 (unified sit-stand-walk cycle) — unaffected; `max_continuous_computer_secs`
  and `computer_break_reset_secs` move from state to config-by-reference in
  T02, no behavior change.
- **New ADR 015** (`.arch/ADR/015-pure-ergo-engine.md`, written in T07):
  records the "engine owns policy-facing derived fields; adapters only map
  outputs to WinAPI/Tauri calls" rule that T05 enforces. ADR number 015
  (not 014) is deliberate — 014 is reserved for the persistence-precedence
  ADR that the sibling backend-hardening epic (E019, not yet written at
  the time of this plan) is expected to create; if E019 lands first and
  claims a different number, T07 renumbers this ADR to the next free slot
  and fixes the cross-reference, it does not fight over 014.
  `.plan/decisions.jsonl` does not exist in this repo yet — E015-T04
  creates it with D1/D2; T07 appends this epic's decisions (D3 below) to
  the same file rather than creating a second one.
  - **D3 — daily-reset calendar day.** The UTC-vs-local mismatch found
    above is fixed in T01 by comparing the injected `now`'s local calendar
    day, matching `session_persistence.rs` and `db_sessions.rs`. No
    alternative is proposed: every other date-keyed surface in this repo
    already uses local date, so UTC-day comparison is the outlier, not a
    considered choice.
- Alternatives to a full `ErgoEvent`/`step()` rewrite were considered and
  rejected for T05 — see Alternatives below.

## Alternatives

| | A. Minimum (patch the seams) | B. Strangler steps 4-8 (this plan) | C. Full engine rewrite (`ErgoEngine::step(event, cfg, now) -> Vec<Signal>`) |
|---|---|---|---|
| Summary | Fix only the hot-reload bug and the UTC/local mismatch found while reading the code; leave clock-reading, config-copying, hand-mirrored persistence, and tray re-derivation as-is | Do all five remaining `engine.md` migration steps as separate, sequenced tasks, each keeping `cargo test --lib` green | Replace `SessionManager`/`SessionState` with a single `ErgoEvent`-driven state machine and a `Signal` enum, as `engine.md`'s "Target shape" sketches it |
| Effort | S (5) | L (59, this plan) | XL (80+, unplanned) |
| Risk | L | M | H |
| Pros | Ships the two real bugs fast | Removes every seam `engine.md` names; each step ships independently and is individually testable; strangler keeps the 740-test net alive throughout | Cleanest possible model on paper |
| Cons | Leaves clock-reading, config-copying, hand-mirrored persistence and tray re-derivation exactly as found — the next feature reopens one of them, same pattern as the nine March rewrites | Touches nearly every file in the session engine, in sequence, so most tasks cannot run in parallel (they share files) | Re-creates the exact bug class this epic exists to close — a second "elapsed" or "event" concept invented mid-migration — while discarding the safety net that catches it; `engine.md` explicitly recommends against this, and E015's PLAN.md already rejected it for the counter fix at a smaller scale |
| Reuses | everything | `SessionState`, `SessionManager`, `CommunicationPolicy`, `Signals`/`NotifySignal` (already exist in `communication_types.rs` — see T05), the full test suite | little; `communication_types::Signals` already covers most of what a hand-rolled `Signal` enum would add, making the rewrite's main selling point already available cheaply |

**Recommendation: B.** `engine.md` names these exact five steps as the
sequel to E015's steps 1-3 and estimates them at 42 points combined; this
plan adds the explicit scenario-table test (T06) and the docs/verify
closing tasks the review didn't itemize, landing at 59. C's core claim —
"one struct owns everything, signals instead of ad-hoc derivation" — is
already achievable without inventing a new event enum, because
`communication_types::Signals` exists today; T05 wires the engine to
populate the fields `tray_controller.rs` currently recomputes, which is
the actual behavior C promises, at a fraction of the risk.

## Scope

**In**: `session_types.rs`, `session_reading.rs`, `session_breaks.rs`,
`session_daily.rs`, `session_manager.rs`, `session_persistence.rs`,
`hourly_break_tracker.rs`, `serial_periodic.rs`, `commands.rs`
(`inject_reading` call site only), `profile_reload.rs`,
`tray_controller.rs`, `communication_policy.rs`, `ergonomic_profile.rs`
(no field changes, read-only reference), `ws_broadcaster.rs` (DTO
construction site), `metrics/tests.rs`,
`communication_policy_cooldown_tests.rs`,
`communication_policy_nudge_tests.rs`, `communication_policy_tests.rs`,
every `session_tests_*.rs` file that backdates `Utc::now()` or constructs
a full `SessionState`/`SessionStateDto` literal, `.arch/ADR/015-pure-ergo-engine.md`
(new), `.arch/ADR/008`, `.arch/ADR/009` (cross-reference only),
`.arch/ARCHITECTURE.md`, `CLAUDE.md` Communication Architecture section,
`.plan/decisions.jsonl` (append).

**Out**: the credited-counter fix and `SessionRow.break_credit` (E015,
already merged when this epic starts); release docs, LICENSE, code
signing (E017); frontend reducer merge, TS/Rust type codegen, dead
component removal (E018); `commands.rs` mutex poisoning, event-name
constants, `EventLogger` panic-vs-warn parity, `google_fit*.rs` split
(E019 — sibling epic, not yet written at plan time); a full
`ErgoEvent`/`step()` rewrite (rejected above); anything about the popup's
visual rendering (E015-T05 already covers the browser check for the
credited counter; this epic does not touch `OneBarTimer.tsx` or any other
`.tsx` file).

## Tasks

| ID | Task | Points | Agent | Wave |
|---|---|---|---|---|
| T01 | Inject the clock: `on_reading`, `check_daily_reset`/`needs_daily_reset`, `check_notification_conditions` take `now: DateTime<Utc>`; adapters (`serial_periodic.rs`, `commands.rs::inject_reading`) call `Utc::now()` once per tick and pass it down; fix the UTC-vs-local daily-reset day comparison (D3); rewrite every backdating test in the 9 affected `session_tests_*.rs` files to inject `now` directly; add a midnight-rollover test and a DST-transition-day test | 13 | ts-dev (Rust) | 1 |
| T02 | Extract config from state: delete the 6 duplicated fields from `SessionState`; `apply_break_credit`, `check_notification_conditions`, `accumulate_ongoing` take `&ErgonomicProfile` (or `&Limits`) instead of reading `self.state.*`; callers thread the profile already loaded each tick; fixes the hot-reload bug (profile changes now apply without restart); update every `SessionState`/`SessionStateDto` struct literal across test files and `ws_broadcaster.rs` | 13 | ts-dev (Rust) | 2 |
| T03 | Unify the break model under one naming convention: `BreakCredit` → session-level credit stays as-is but documented as one of three named "break" concepts; extract the Day Break Credit special case out of `apply_break_credit` into its own named function; add a module-level doc comment in `session_breaks.rs` and `hourly_break_tracker.rs` cross-referencing the other two, so "break" never again means three unrelated things without a map between them | 5 | ts-dev (Rust) | 3 |
| T04 | One persistence snapshot: derive `Serialize`/`Deserialize` on `SessionState` (post-T02, six fields already gone); fold the manager-level flags (`alert_fired`, `stand_alert_fired`, the four `notify_*`/`praise_*`/`standing_target_reached_fired` flags) and `HourlyBreakTracker`'s maps into one versioned `PersistedEngineState { schema_version: 1, state: SessionState, .. }`; write a migration path that reads the old unversioned `PersistedSessionState` JSON shape (no `schema_version` field present) into the new one; `session_persistence.rs`'s public functions (`save_via_app`, `load_persisted_state`, `clear`) keep their signatures | 13 | ts-dev (Rust) | 4 |
| T05 | Engine owns policy-facing derived fields: `SessionManager::snapshot()` (or a new `policy_input(now)` method) computes and exposes `elapsed_secs`, `standing_lap_progress`, `standing_lap`, `standing_lap_flash` directly; delete `tray_controller.rs::compute_standing_lap()` and the `elapsed_secs` match in `update_from_policy` (`tray_controller.rs:121-128,154-164`), replacing both with reads off the engine-provided values; `CommunicationPolicy::evaluate`'s `PolicyInput` shape is unchanged, only who computes its fields changes | 13 | ts-dev (Rust) | 5 |
| T06 | Scenario-table regression test: one Rust test module, one table of `(events, expected DTO)` cases exercising the real `on_reading`/`snapshot` path with an injected clock — sit→stand→sit credit visible on the DTO, a sleep-gap credit, a midnight rollover, and a DST-transition day; each case documents, in a comment, which pre-existing test (if any) asserted the old wrong behavior for that scenario and whether it was deleted or inverted | 8 | ts-dev (Rust) | 5 |
| T07 | Docs: `.arch/ADR/015-pure-ergo-engine.md` (new), cross-references in ADR 008/009, `.arch/ARCHITECTURE.md` session-engine section rewritten to describe the config-by-reference + derived-snapshot + engine-owns-signals shape, `CLAUDE.md` Communication Architecture section updated to state the rule as fact instead of aspiration, `.plan/decisions.jsonl` gets D3 appended | 3 | main | 6 |
| T08 | Verify: full `cargo test --manifest-path src-tauri/Cargo.toml --lib` green (not a filtered subset — this is the epic-closing full-suite run); evidence records for every check below written as `current` | 1 | verify | 7 |

Wave points: W1=13, W2=13, W3=5, W4=13, W5=21 (T05 13 + T06 8, parallel —
disjoint files: T05 touches `tray_controller.rs`/`communication_policy.rs`/
`session_manager.rs`, T06 only adds a new test file), W6=3, W7=1. All
waves are ≤ 40. T01→T02→T03→T04 are strictly sequential: each rewrites
`SessionState`, `session_breaks.rs`, or both, and AO's "two tasks in one
wave never claim the same file" rule forbids overlapping them. T05/T06
both depend on T04 and do not touch each other's files, so they run in
the same wave.

## Test strategy

Four shadow paths (happy / nil / empty / error) per `rules/testing.md`,
named against the actual change:

- **T01 (clock injection)**
  - happy: sit 30 min (injected `now` advances by fixed increments) →
    stand 2 min → sit; DTO shows credited value computed purely from the
    injected timestamps, no wall-clock dependency. **Fails today**: the
    function signature has no `now` parameter to inject.
  - nil: `last_tick_ts` is `None` (fresh boot) — no sleep-gap credit is
    applied on the very first reading. Already covered by
    `session_tests_sleep.rs`; re-verify after the signature change.
  - empty: a break of exactly `break_min_secs` (60s) — boundary, credit
    fires (`break_secs < min_secs` is strict `<`). Already covered by
    `break_credit_none_for_very_short_break` at 30s; add a 60s-exact case.
  - error/edge: **midnight rollover** — construct `now` at
    `23:59:59 local` then `00:00:01 local` (same UTC day if the machine
    is UTC+, different UTC day if UTC-) and assert the reset fires on the
    LOCAL day boundary, not the UTC one. **Fails today**: `session_daily.rs:14,24`
    compares `now.date_naive()` (UTC). **DST day**: construct `now`
    spanning a spring-forward or fall-back transition (`chrono-tz` is
    already a dependency, see `Cargo.toml:29`) and assert the reset still
    fires exactly once for that local calendar day. **Fails today**: no
    such test exists; the whole suite depends on wall-clock `Utc::now()`
    so a DST edge cannot be constructed deterministically before this
    task.
  - New file: `session_tests_clock.rs`.
- **T02 (config extraction)**
  - happy: hot-reload the ergonomic profile mid-session (change
    `break_credit_multiplier` on disk, trigger `reload_profiles_if_changed`)
    and confirm the NEXT break credit computation uses the new multiplier
    without an app restart. **Fails today**: `SessionState` holds its own
    copy set once at construction; `profile_reload.rs` never touches
    `SessionManager`.
  - nil: an `ErgonomicProfile` with all `Limits` at their serde defaults
    produces identical behavior to today's hardcoded constants.
  - empty: `day_break_min_secs: 0` (disabled) — already covered by
    `session_tests_day_break.rs:61`; confirm the disabled path still
    works when the value is passed by reference instead of copied.
  - error: `break_credit_multiplier` outside `[0.0, 10.0]` is still
    clamped (existing behavior in `session_breaks.rs:128`); assert the
    clamp survives the signature change.
- **T03 (unify break model)**
  - happy: a 6h+ break triggers both session-level Full credit (already
    tested) and the Day Break Credit reset (already tested) — after this
    task, both are visible in the event log under the new naming, and a
    single new test asserts the log lines are recognizably distinct
    ("SESSION_CREDIT" vs "DAY_CREDIT" vs "HOURLY_COVERAGE", exact names
    decided by the implementer, documented in the module doc comment).
  - nil/empty/error: no new branch logic is introduced — this task is a
    naming/extraction task, not a behavior change; existing
    `session_tests_day_break.rs` and `hourly_break_tracker.rs`'s own
    `#[cfg(test)] mod tests` continue to pass unmodified in substance
    (only import paths may change).
- **T04 (persistence snapshot)**
  - happy: save a live session, reload it, confirm every field round-trips
    including the ones currently split across `PersistedSessionState` and
    `HourlyBreakTracker`. **Fails today**: no single type exists to
    round-trip in one call.
  - nil: load with no stored value at all (`store.get(STORE_KEY)` returns
    `None`) — falls through to fresh state, matches today's behavior.
  - empty: load a value that deserializes but has an empty
    `hours_with_break`/`hours_active` map — round-trips to empty maps, not
    a panic.
  - error: load the OLD (pre-migration, unversioned) JSON shape — the
    migration path produces a valid `PersistedEngineState` with
    `schema_version: 1`. **Fails today**: no migration function exists;
    the old and new shapes are the same struct today, so there is nothing
    to migrate FROM yet.
- **T05 (signals from engine)**
  - happy: a Standing session crossing a stand-limit lap boundary produces
    the same `standing_lap`, `standing_lap_progress`, `standing_lap_flash`
    values whether read from the engine snapshot or (before this task)
    recomputed in `tray_controller.rs` — a characterization test captures
    today's `compute_standing_lap()` output for a fixed scenario, then
    the same scenario against the engine-provided fields must match
    after the refactor.
  - nil: `stand_limit_secs == 0` (disabled) — lap fields are inert
    (`0.0, 0, false`), matching `tray_controller.rs:155-156`'s existing
    early return.
  - empty: `Sitting` state — lap fields are their zero value (only
    meaningful in `Standing`), matching today.
  - error: `standing_seconds` somehow exceeds a `u32`-scale target after a
    long uptime — assert the lap count computation does not overflow
    (`(snapshot.standing_seconds / target) as u32`, `tray_controller.rs:162`,
    already an `as` cast with no overflow check — add a bounds assertion
    if the cast is now moved into the engine).
- **T06 (scenario table)** — this task's entire deliverable IS the test:
  one `#[test] fn scenario_table()` iterating a `Vec<Scenario>`, each row
  a struct `{ name: &str, events: Vec<(i32 /* mm */, bool /* active */, DateTime<Utc>)>, expect: fn(&SessionStateDto) }`.
  Minimum rows: sit→stand(2min)→sit credit visible on DTO; sleep-gap
  credit; midnight rollover; DST day. Every row is annotated with a
  comment naming the pre-existing test (if any) that asserted the WRONG
  behavior for that scenario before E015/this epic, and whether that test
  was deleted or its assertion inverted — this is the direct answer to
  `engine.md`'s "tests exist but test the wrong things" verdict.

## Evidence contract

| check_id | class | procedure | expected | record |
|---|---|---|---|---|
| clock-injection | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- session_tests_clock session_tests_break_credit session_tests_sleep session_tests_day_break session_tests_daily` | pass | `evidence/records/T01-clock-injection.json` |
| config-extraction | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- session_tests_away session_tests_serde communication_policy` | pass | `evidence/records/T02-config-extraction.json` |
| unify-break-model | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- session_tests_day_break hourly_break_tracker` | pass | `evidence/records/T03-unify-break-model.json` |
| persistence-snapshot | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- session_tests_persistence` | pass | `evidence/records/T04-persistence-snapshot.json` |
| engine-signals | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- tray_controller communication_policy` | pass | `evidence/records/T05-engine-signals.json` |
| scenario-table | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- scenario_table` | pass | `evidence/records/T06-scenario-table.json` |
| docs-consistency | manual | `node scripts/check-e020-t07-docs.mjs` | exits 0, ADR-015 exists and is referenced from ARCHITECTURE.md and CLAUDE.md | `evidence/records/T07-docs-consistency.json` |
| full-suite | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib` | pass, 0 failures | `evidence/records/T08-full-suite.json` |

## Architecture impact

- `.arch/ARCHITECTURE.md`: session-engine section rewritten — config is
  borrowed, not copied; persistence is one versioned snapshot; tray/comm
  read engine-computed fields instead of re-deriving them.
- New ADR: `.arch/ADR/015-pure-ergo-engine.md` (see Decisions above for
  the numbering caveat).
- ADR 008, ADR 009: cross-reference added, status stays `accepted`, no
  content change to their formulas.
- `CLAUDE.md` Communication Architecture section: "TrayController only
  executes signals — no decision logic" changes from aspiration to a
  cited, true statement.

## Acceptance criteria

1. No function in the session engine calls `Utc::now()` internally except
   at a documented adapter boundary (`serial_periodic.rs`,
   `commands.rs::inject_reading`); every stepping function takes `now`.
2. Daily reset fires on the local calendar day boundary, matching
   `session_persistence.rs` and `db_sessions.rs`, not the UTC boundary.
3. `SessionState` no longer contains `break_min_secs`,
   `break_credit_multiplier`, `day_break_min_secs`,
   `posture_balance_min_sitting_secs`, `max_continuous_computer_secs`, or
   `computer_break_reset_secs`; hot-reloading the ergonomic profile
   changes break-credit behavior on the next tick, without an app
   restart.
4. `hourly_break_tracker.rs` and `session_breaks.rs` each carry a doc
   comment naming the other two "break" concepts and how they differ.
5. Saving and loading session state round-trips through one versioned
   type; an old, unversioned persisted-state file loads correctly via the
   migration path.
6. `tray_controller.rs` contains no function that recomputes
   `elapsed_secs` or standing-lap progress from a snapshot — it reads
   those fields off the snapshot the engine already produced.
7. A scenario-table test exists covering sit→stand→sit credit on the DTO,
   sleep gap, midnight rollover, and a DST-transition day, and each row
   documents what pre-existing test it supersedes.
8. `cargo test --manifest-path src-tauri/Cargo.toml --lib` is green with
   zero filtered-out modules.
