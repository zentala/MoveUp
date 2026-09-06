---
formatVersion: 1
type: handoff
status: todo
---

# E020 Handoff — engine: pure core (steps 4-8)

## TLDR

Implementing session reads only this file plus [`PLAN.md`](PLAN.md). Do not
start this epic until E015 (`../E015-2026-09-06-engine-single-truth/`) has
merged — T02 of this epic removes fields E015 does not touch, but T01
rewrites `on_reading`'s signature, which E015-T01 also edits; rebasing
this epic onto E015's merged result avoids a painful conflict. Seven
sequential/paired waves, 59 points total. Work in a worktree via
`wt-add`, branch `feat/E020-engine-pure-core`. Bump version per
`.claude/rules/versioning.md` if this epic starts a new minor (check
`STATE.md` for the version E017 shipped).

## Decisions already made (apply unless Paweł overrides)

- D3: daily reset compares the injected clock's LOCAL calendar day, not
  UTC — matches every other date-keyed surface in this repo
  (`session_persistence.rs`, `db_sessions.rs`). No alternative considered;
  see PLAN.md Decisions.
- ADR number for the new pure-core ADR is 015. If the sibling epic E019
  (backend hardening, not written yet) has already claimed 014 or 015 by
  the time T07 runs, renumber to the next free integer and fix the
  cross-reference — do not block on this, do not fight over the number.
- No full `ErgoEvent`/`step()` rewrite — Alternative B in PLAN.md,
  strangler steps only.
- Approach: five sequential-by-necessity Rust tasks (T01-T05) because each
  rewrites shared files (`session_types.rs`, `session_breaks.rs`,
  `session_reading.rs`), then two tasks that can run in parallel (T05/T06
  — wait, T05 and T06 are the parallel pair; T01-T04 are strictly
  sequential), then docs (T07), then a full-suite verify (T08).

## Mental model

- **Where the clock currently leaks in** (T01 fixes all of these):
  `session_reading.rs:12` (`let now = Utc::now();` inside `on_reading`),
  `session_daily.rs:13,23` (`needs_daily_reset`/`check_daily_reset`),
  `session_breaks.rs:170` (`check_notification_conditions`). Contrast with
  `handle_state_exit` (`session_breaks.rs:14`), which already takes `now`
  as a parameter — that is the pattern to extend everywhere else.
- **The UTC/local day bug**: `session_daily.rs:14,24` do
  `now.date_naive()` where `now = Utc::now()`. Fix: derive the local date
  via `now.with_timezone(&chrono::Local).date_naive()` (or accept the
  local date as a second, explicit argument — implementer's call, but
  document which in the doc comment). `chrono-tz` is already a dependency
  (`Cargo.toml:29`) for the DST test.
- **The six config-in-state fields** (T02 deletes them from
  `SessionState`, defined at `session_types.rs:139-150`): read at
  `session_breaks.rs:127-128` (`apply_break_credit`), `:145`
  (day-break threshold), `:183` (PostureBalance guard),
  `session_reading.rs:197` (`computer_break_reset_secs`). Written at
  construction only, `session_manager.rs:71-76` (`new()`) and `:136-141`
  (`new_from_config()`) — **never re-written after that**, which is why
  `profile_reload.rs:96-105` calling `comm_policy.set_ergo_profile(...)`
  does nothing for the session engine. Fix means these functions borrow
  `&ErgonomicProfile` (or `&Limits`) instead; `serial_periodic.rs:32`
  (`check_periodic`) and `:115` (`handle_reading`) already load
  `policy.ergo_profile().clone()` every tick — pass a reference through
  instead of relying on stored state.
  Files that build a full `SessionState` or `SessionStateDto` struct
  literal and must be updated once the fields are gone: `session_manager.rs`
  (both constructors), `metrics/tests.rs:9-49`,
  `communication_policy_cooldown_tests.rs`,
  `communication_policy_nudge_tests.rs`, `communication_policy_tests.rs`,
  `session_tests_serde.rs`, `ws_broadcaster.rs:136`. Tests that mutate
  these fields directly on `m.state.*` and must switch to passing a
  custom profile instead: `session_tests_sleep.rs:142`,
  `session_tests_day_break.rs:61`, `session_tests_away.rs:185,210`.
- **Break means three things** (T03): `BreakCredit` in
  `session_breaks.rs:126-158` (`session_tests_break_credit.rs` covers it),
  `HourlyBreakTracker` in `hourly_break_tracker.rs` (its own
  `#[cfg(test)] mod tests` at the bottom of that file), and the Day Break
  Credit special case nested inside `apply_break_credit`,
  `session_breaks.rs:142-157` (covered by `session_tests_day_break.rs`).
  Do not merge them into one struct — they measure different things
  (a countdown, a per-hour boolean, a notification-flag reset). Extract
  the nested special case into its own named function
  (`apply_day_break_reset` or similar) called from `apply_break_credit`,
  and add doc comments cross-referencing all three.
- **The hand-mirrored persistence type** (T04): `PersistedSessionState`
  at `session_persistence.rs:78-107` vs. `SessionState` at
  `session_types.rs:78-153` (no `Serialize`/`Deserialize` derive today).
  Manager-level flags that belong in the new unified snapshot but
  currently live outside `SessionState` entirely: `alert_fired`,
  `stand_alert_fired` (`session_manager.rs:16-17`),
  `notify_inactivity_fired`, `notify_posture_balance_fired`,
  `praise_halfway_fired_today`, `standing_target_reached_fired`
  (`session_manager.rs:20-23`), and `hourly_break_tracker`
  (`session_manager.rs:33`, its own struct in `hourly_break_tracker.rs`).
  Public functions to keep stable: `save_via_app`, `load_persisted_state`,
  `clear`, `load_reset_after` (all in `session_persistence.rs`) —
  `serial_periodic.rs` and `commands.rs` call these by name; changing
  their signatures ripples further than this task's scope.
- **Where tray re-derives instead of consuming** (T05):
  `tray_controller.rs:121-128` (`elapsed_secs` match) and `:154-164`
  (`compute_standing_lap`). These read `snapshot()`'s existing fields
  (`snapshot.sitting_seconds`, `.break_seconds`, `.stand_limit_secs`,
  `.standing_seconds`) to recompute values the engine could hand over
  directly. `communication_policy.rs`'s `PolicyInput` struct
  (`communication_policy.rs:12-20`) is the target shape both
  `elapsed_secs` and the three lap fields already feed — do not change
  `PolicyInput` itself, only who fills it in.
- **Full list of `session_tests_*.rs`** (18 files, from `ls
  src-tauri/src/session_tests_*.rs`): `alerts`, `away`,
  `away_transitions`, `break_credit`, `break_tracker`, `daily`,
  `day_break`, `floating`, `flush`, `kpi`, `persistence`, `props`,
  `scenarios`, `scenarios_adv`, `score`, `serde`, `sleep`, `timers`,
  `timers_live`. Not every file needs editing in every task — see each
  task's own file list below; this list exists so nobody has to re-glob
  it while implementing.

## Tasks

- [ ] **T01** (13, ts-dev Rust) — inject the clock. Change
  `SessionManager::on_reading`, `needs_daily_reset`, `check_daily_reset`,
  `check_notification_conditions` to take `now: DateTime<Utc>`; remove
  every internal `Utc::now()` call from these; update
  `serial_periodic.rs::check_periodic`/`handle_reading` and
  `commands.rs::inject_reading` to capture `now` once per invocation and
  pass it down; fix the UTC-vs-local day comparison (D3); rewrite
  backdating patterns in `session_tests_break_credit.rs`,
  `session_tests_sleep.rs`, `session_tests_day_break.rs`,
  `session_tests_daily.rs`, `session_tests_timers.rs`,
  `session_tests_timers_live.rs`, `session_tests_scenarios.rs`,
  `session_tests_scenarios_adv.rs`, `session_tests_away.rs`,
  `session_tests_away_transitions.rs` to inject `now` directly; add
  `session_tests_clock.rs` with a midnight-rollover test and a
  DST-transition-day test.
  Verify: `cargo test --manifest-path src-tauri/Cargo.toml --lib --
  session_tests_clock session_tests_break_credit session_tests_sleep
  session_tests_day_break session_tests_daily`.
- [ ] **T02** (13, ts-dev Rust) — extract config from state. Delete
  `break_min_secs`, `break_credit_multiplier`, `day_break_min_secs`,
  `posture_balance_min_sitting_secs`, `max_continuous_computer_secs`,
  `computer_break_reset_secs` from `SessionState`; thread
  `&ErgonomicProfile` (or a narrower `&Limits`) into `apply_break_credit`,
  `check_notification_conditions`, `accumulate_ongoing`; update both
  `SessionManager` constructors and every struct literal listed in
  Mental model above. Verify:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib -- session_tests_away
  session_tests_serde communication_policy`.
- [ ] **T03** (5, ts-dev Rust) — unify the break model. Extract the Day
  Break Credit special case out of `apply_break_credit` into its own
  named function; add cross-referencing doc comments to
  `session_breaks.rs` and `hourly_break_tracker.rs`; give the three
  concepts distinct, consistent log-line prefixes if they are not already
  distinct. Verify: `cargo test --manifest-path src-tauri/Cargo.toml
  --lib -- session_tests_day_break hourly_break_tracker`.
- [ ] **T04** (13, ts-dev Rust) — one persistence snapshot. Derive
  `Serialize`/`Deserialize` on `SessionState`; introduce
  `PersistedEngineState { schema_version: u32, .. }` wrapping it plus the
  manager-level flags and `HourlyBreakTracker`; write a migration from the
  old unversioned JSON shape; keep `save_via_app`/`load_persisted_state`/
  `clear`/`load_reset_after` signatures. Verify:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib --
  session_tests_persistence`.
- [ ] **T05** (13, ts-dev Rust) — engine owns policy-facing derived
  fields. Add `elapsed_secs`, `standing_lap_progress`, `standing_lap`,
  `standing_lap_flash` to what `SessionManager::snapshot()` (or a new
  `policy_input(now)` method) computes; delete
  `tray_controller.rs::compute_standing_lap` and the `elapsed_secs` match
  in `update_from_policy`; `PolicyInput`'s shape in
  `communication_policy.rs` does not change. Verify:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib --
  tray_controller communication_policy`.
- [ ] **T06** (8, ts-dev Rust) — scenario-table regression test. New file,
  one table of `(events, expected DTO)` cases: sit→stand(2min)→sit credit
  on the DTO, sleep-gap credit, midnight rollover, DST-transition day.
  Each row's comment names the pre-existing test it supersedes and
  whether that test was deleted or inverted. Verify:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib -- scenario_table`.
- [ ] **T07** (3, main) — docs. `.arch/ADR/015-pure-ergo-engine.md`
  (numbering caveat in PLAN.md Decisions); cross-references added to ADR
  008/009; `.arch/ARCHITECTURE.md` session-engine section rewritten;
  `CLAUDE.md` Communication Architecture section updated;
  `.plan/decisions.jsonl` gets D3 appended (create the file if E015-T04
  has not already created it — check first, do not overwrite). Verify:
  `node scripts/check-e020-t07-docs.mjs` (new script, asserts ADR-015
  exists, is non-empty, and is referenced by both `.arch/ARCHITECTURE.md`
  and `CLAUDE.md`; exits non-zero on any missing reference).
- [ ] **T08** (1, verify) — full-suite verify. Run
  `cargo test --manifest-path src-tauri/Cargo.toml --lib` unfiltered;
  confirm 0 failures; write all eight evidence records as `current`.

## Done means

All eight acceptance criteria in PLAN.md hold, evidence records are
`current`, `.plan/HISTORY.md` entry written, `STATE.md` updated, and the
epic's `IMPRO.md` (if the Stop hook created one) has been triaged before
closing via `/done`.

## AO

```yaml
project: MoveUp
epic: E020
base_ref: main
tasks:
  - id: E020-T01
    repo: MoveUp
    executor: ts-dev
    depends_on: []
    write_set: ["src-tauri/src/session_reading.rs", "src-tauri/src/session_daily.rs", "src-tauri/src/session_breaks.rs", "src-tauri/src/session_manager.rs", "src-tauri/src/serial_periodic.rs", "src-tauri/src/commands.rs", "src-tauri/src/session_tests_break_credit.rs", "src-tauri/src/session_tests_sleep.rs", "src-tauri/src/session_tests_day_break.rs", "src-tauri/src/session_tests_daily.rs", "src-tauri/src/session_tests_timers.rs", "src-tauri/src/session_tests_timers_live.rs", "src-tauri/src/session_tests_scenarios.rs", "src-tauri/src/session_tests_scenarios_adv.rs", "src-tauri/src/session_tests_away.rs", "src-tauri/src/session_tests_away_transitions.rs", "src-tauri/src/session_tests_clock.rs", "src-tauri/src/lib.rs"]
    claims: ["src-tauri/src/session_reading.rs", "src-tauri/src/session_daily.rs", "src-tauri/src/session_breaks.rs", "src-tauri/src/session_manager.rs", "src-tauri/src/serial_periodic.rs", "src-tauri/src/commands.rs", "src-tauri/src/session_tests_break_credit.rs", "src-tauri/src/session_tests_sleep.rs", "src-tauri/src/session_tests_day_break.rs", "src-tauri/src/session_tests_daily.rs", "src-tauri/src/session_tests_timers.rs", "src-tauri/src/session_tests_timers_live.rs", "src-tauri/src/session_tests_scenarios.rs", "src-tauri/src/session_tests_scenarios_adv.rs", "src-tauri/src/session_tests_away.rs", "src-tauri/src/session_tests_away_transitions.rs", "src-tauri/src/session_tests_clock.rs", "src-tauri/src/lib.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- session_tests_clock session_tests_break_credit session_tests_sleep session_tests_day_break session_tests_daily"
    budget_minutes: 90
  - id: E020-T02
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E020-T01"]
    write_set: ["src-tauri/src/session_types.rs", "src-tauri/src/session_manager.rs", "src-tauri/src/session_breaks.rs", "src-tauri/src/session_reading.rs", "src-tauri/src/profile_reload.rs", "src-tauri/src/serial_periodic.rs", "src-tauri/src/metrics/tests.rs", "src-tauri/src/communication_policy_cooldown_tests.rs", "src-tauri/src/communication_policy_nudge_tests.rs", "src-tauri/src/communication_policy_tests.rs", "src-tauri/src/session_tests_serde.rs", "src-tauri/src/session_tests_sleep.rs", "src-tauri/src/session_tests_day_break.rs", "src-tauri/src/session_tests_away.rs", "src-tauri/src/session_tests_e015_credited.rs", "src-tauri/src/session_tests_limits.rs", "src-tauri/src/ws_broadcaster.rs", "src-tauri/src/lib.rs"]
    claims: ["src-tauri/src/session_types.rs", "src-tauri/src/session_manager.rs", "src-tauri/src/session_breaks.rs", "src-tauri/src/session_reading.rs", "src-tauri/src/profile_reload.rs", "src-tauri/src/serial_periodic.rs", "src-tauri/src/metrics/tests.rs", "src-tauri/src/communication_policy_cooldown_tests.rs", "src-tauri/src/communication_policy_nudge_tests.rs", "src-tauri/src/communication_policy_tests.rs", "src-tauri/src/session_tests_serde.rs", "src-tauri/src/session_tests_sleep.rs", "src-tauri/src/session_tests_day_break.rs", "src-tauri/src/session_tests_away.rs", "src-tauri/src/session_tests_e015_credited.rs", "src-tauri/src/session_tests_limits.rs", "src-tauri/src/ws_broadcaster.rs", "src-tauri/src/lib.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- session_tests_away session_tests_serde communication_policy"
    budget_minutes: 90
  - id: E020-T03
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E020-T02"]
    write_set: ["src-tauri/src/session_breaks.rs", "src-tauri/src/hourly_break_tracker.rs", "src-tauri/src/session_tests_day_break.rs"]
    claims: ["src-tauri/src/session_breaks.rs", "src-tauri/src/hourly_break_tracker.rs", "src-tauri/src/session_tests_day_break.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- session_tests_day_break hourly_break_tracker"
    budget_minutes: 60
  - id: E020-T04
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E020-T03"]
    write_set: ["src-tauri/src/session_types.rs", "src-tauri/src/session_persistence.rs", "src-tauri/src/session_manager.rs", "src-tauri/src/session_tests_persistence.rs"]
    claims: ["src-tauri/src/session_types.rs", "src-tauri/src/session_persistence.rs", "src-tauri/src/session_manager.rs", "src-tauri/src/session_tests_persistence.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- session_tests_persistence"
    budget_minutes: 90
  - id: E020-T05
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E020-T04"]
    write_set: ["src-tauri/src/session_manager.rs", "src-tauri/src/session_types.rs", "src-tauri/src/tray_controller.rs", "src-tauri/src/tray_controller_tests.rs", "src-tauri/src/communication_policy.rs"]
    claims: ["src-tauri/src/session_manager.rs", "src-tauri/src/session_types.rs", "src-tauri/src/tray_controller.rs", "src-tauri/src/tray_controller_tests.rs", "src-tauri/src/communication_policy.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- tray_controller communication_policy"
    budget_minutes: 90
  - id: E020-T06
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E020-T04"]
    write_set: ["src-tauri/src/session_tests_scenario_table.rs", "src-tauri/src/lib.rs"]
    claims: ["src-tauri/src/session_tests_scenario_table.rs", "src-tauri/src/lib.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- scenario_table"
    budget_minutes: 60
  - id: E020-T07
    repo: MoveUp
    executor: main
    depends_on: ["E020-T05", "E020-T06"]
    write_set: [".arch/ADR/015-pure-ergo-engine.md", ".arch/ADR/008-proportional-break-credit.md", ".arch/ADR/009-day-break-credit.md", ".arch/ARCHITECTURE.md", "CLAUDE.md", ".plan/decisions.jsonl", "scripts/check-e020-t07-docs.mjs"]
    claims: [".arch/ADR/015-pure-ergo-engine.md", ".arch/ADR/008-proportional-break-credit.md", ".arch/ADR/009-day-break-credit.md", ".arch/ARCHITECTURE.md", "CLAUDE.md", ".plan/decisions.jsonl", "scripts/check-e020-t07-docs.mjs"]
    verification: "node scripts/check-e020-t07-docs.mjs"
    budget_minutes: 45
  - id: E020-T08
    repo: MoveUp
    executor: verify
    depends_on: ["E020-T07"]
    write_set: [".plan/epics/E020-2026-09-06-engine-pure-core/evidence/records/T08-full-suite.json"]
    claims: [".plan/epics/E020-2026-09-06-engine-pure-core/evidence/records/T08-full-suite.json"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib"
    budget_minutes: 30
```
