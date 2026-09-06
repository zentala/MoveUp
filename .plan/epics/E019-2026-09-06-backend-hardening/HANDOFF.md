# E019 Handoff — backend hardening

## TLDR

Implementing session reads only this file plus [`PLAN.md`](PLAN.md). Eight
tasks, mostly one per wave — `commands.rs` and `lib.rs` are each touched by
5+ tasks, so parallelizing two of them would double-claim a file. Work in a
worktree via `wt-add`, branch `feat/E019-backend-hardening`. Do not touch
`session_types.rs`, `session_manager.rs`, `session_breaks.rs`,
`session_live.rs`, `session_reading.rs`, `session_daily.rs`,
`session_persistence.rs`, or `db_sessions.rs` — those are E015's files.

## Decisions already made

- Approach B from PLAN.md (targeted hardening, no Tauri-free crate) —
  Paweł's instruction explicitly rules out a crate split; the reviewer's own
  recommendation agrees.
- ADR goes to `.arch/ADR/014-persistence-precedence.md`, not
  `.plan/ADR/` — this repo still uses `.arch/` (see
  `.arch/ARCHITECTURE.md`, `.plan/decisions.jsonl` does not exist here).
- T03's `src/events.ts` is a new, standalone file (constants only) — do
  **not** rewire any existing TS consumer (`useDesk.ts`, `types.ts`,
  `TodayStats.tsx`, etc.) to import from it. That rewiring is E018's job;
  wiring it here would create a merge conflict with E018's reducer work.
- Every new Rust test in this epic is named with the prefix `e019_t0N_`
  (N = task number) so each task's `verification` command matches only its
  own tests and nothing from a sibling task or from E015/E017/E018.

## Mental model

- **Why 7 of 8 waves are single-task**: every top-level Rust module in this
  crate is declared with a flat `mod x;` line in `src-tauri/src/lib.rs`
  (confirmed: 66 such lines, e.g. `lib.rs:1-90`) — there is no nested
  `mod` declared inside a parent file. Any task that adds a new file
  (`today_totals.rs`, `desk_events.rs`, `remote_display_state.rs`, the
  google_fit/serial_periodic split siblings, new `*_tests.rs` files) must
  add a line to `lib.rs`. T01, T02, T03, T04, T05, T06, T07 all add at least
  one file, so all seven would claim `lib.rs` if run in parallel. They run
  sequentially instead, in the numeric order above (T01 first because its
  edit to `commands.rs` is the smallest and most mechanical; T02 next
  because it also touches `commands.rs`, before T03 changes the same
  file's `emit` call site for a different reason).
- **Poison-safe pattern already exists — copy it exactly**:
  `src-tauri/src/tray_controller.rs:46,56,58,101,102,105,108,120,140,173,176`
  and `src-tauri/src/remote_server.rs:163,174,182` all use
  `.lock().unwrap_or_else(|e| e.into_inner())`. T01 makes
  `src-tauri/src/commands.rs:44,74,75,80,86,89,95,97,102,155,166,182,193,199,216`
  match that pattern exactly — same replacement, no new abstraction.
- **The three real "today's totals" call sites** (T02): `ensure_initialized`
  at `commands.rs:60-82` (seeds in-memory session from
  `db_sessions::load_today_totals` + `session_persistence::PersistedSessionState`
  at startup — do not touch, this is a read of E015-owned functions, not an
  edit to them); `commands::get_today_summary` (calls
  `db_queries::get_today_summary`, then patches `position_changes` from the
  live session — **this is the duplicate to consolidate**); Analyst's
  `commands_analyst.rs:129-159` `collect_snapshots` (reads raw per-minute
  JSON files for charting — this is NOT a duplicate implementation of the
  same scalar, it serves a different purpose; T02 adds a doc comment there
  pointing at the new ADR so it stops reading as a third source of truth).
- **8 event names to constant-ize** (T03), from the backend review's
  Event/IPC inventory: `desk:distance` (`serial.rs`), `desk:state-changed`
  (`commands.rs` inject_reading, `serial_periodic.rs`), `desk:device-connected`
  (`serial.rs`), `desk:device-lost` (`serial.rs`), `desk:device-missing`
  (`serial.rs`), `desk:sensor-error` (`serial.rs`), `desk:daily-reset`
  (`serial_periodic.rs`), and the three tray-only events `desk:show-widget`,
  `desk:show-settings`, `desk:popup-theme` (`tray.rs`, `tray_signal_exec.rs`).
  `ws_broadcaster.rs`'s `DisplayEvent` enum is a *different*, already-typed
  representation for the WS channel — leave it alone, do not fold it into
  `desk_events.rs`.
- **EventLogger fix** (T04): `event_logger.rs:25-35` panics in `new()`.
  `snapshot_logger.rs` never creates the dir eagerly — `ensure_day_dir`
  (`snapshot_logger.rs:123-127`) creates it lazily on each write and
  `SnapshotLogger::log_snapshot` already handles that `Result` gracefully.
  Make `EventLogger::new` do the same: try `create_dir_all`, `log::warn!`
  on failure instead of panicking, and rely on the existing per-call
  `ensure_day_dir` inside `EventLogger::log` (already there,
  `event_logger.rs:44-56`) to retry on each write.
- **`generate_handler!` parity** (T04): the two lists at `lib.rs:155-232`
  differ only by `commands::inject_reading` and
  `commands_config::get_overlay_state` (both `#[cfg(any(test,
  debug_assertions))]`-gated). Simplest fix: extract each command list into
  a `macro_rules!` or a single list with the two extra entries appended
  under `#[cfg(...)]` inside one `generate_handler!` call if the macro
  supports conditional entries; if it does not (check first —
  `tauri::generate_handler!` is a proc-macro over a fixed token list, cfg
  inside it may not expand), fall back to a compile-time-checked test: a
  `const ALWAYS: &[&str]` and `const DEBUG_ONLY: &[&str]` pair that a new
  unit test asserts contains exactly the debug-list minus the release-list.
  Either mechanism is acceptable; the test in the Evidence contract table
  must fail today and pass after the fix.
- **google_fit split** (T05): `google_fit.rs` is already the "low-level
  Google Fit REST client" per its own module doc comment
  (`google_fit.rs:1-20`) — it is not a service-vs-client naming problem, it
  is a size problem. Its `mod tests` block starts at line 352 (121 lines);
  moving tests to `google_fit_tests.rs` alone is not enough (351 remaining
  lines still exceeds 250). Split further: move `GoogleFitClient` and its
  `impl` (`google_fit.rs:166-351`, the actual HTTP calls —
  `refresh_access_token`, `list_step_sources`, `rank_step_sources`,
  `fetch_steps`) into a new `google_fit_client.rs`; keep `Endpoints`,
  `Credentials`, `FitError`, `classify_response` in `google_fit.rs`.
  `google_fit_service.rs` (328 lines, tests start at line 255) needs only
  the test-extraction to `google_fit_service_tests.rs` to land under 250.
- **serial_periodic split** (T06): `check_periodic`
  (`serial_periodic.rs:21-100`) does daily-reset, snapshot-logging,
  notification-dispatch, and profile-reload-triggering in one function;
  `handle_reading` (`serial_periodic.rs:104-224`) does reading processing,
  event emission, persistence, and three alert checks. Split by
  responsibility into private functions in new sibling files (mirror
  `tray_controller.rs` → `tray_signal_exec.rs`): e.g.
  `serial_periodic_reset.rs` (daily reset + telemetry), keep
  `check_periodic`/`handle_reading` in `serial_periodic.rs` as thin
  orchestrators calling into the new files. Do not change the public
  signatures called from `serial.rs`.
- **Remote-broadcast dedupe** (T07): `tray_controller.rs`'s `AppState` and
  `remote_server.rs`'s `RemoteState`
  (`remote_server.rs:23-34`) both hold `Arc<Mutex<SessionManager>>`,
  `Arc<Mutex<CommunicationPolicy>>`, `Arc<Mutex<TodaySummary>>` under those
  exact field names. Extract one function taking those three `&Arc<Mutex<_>>`
  references directly (not `AppState`/`RemoteState`, so it works from
  either) into a new `remote_display_state.rs`, returning
  `ws_broadcaster::RemoteDisplayState`. Both
  `tray_controller::broadcast_remote_state` (`tray_controller.rs:166-186`)
  and `remote_server::build_remote_display_state`
  (`remote_server.rs:158-187`) call it.
- **CLAUDE.md fix** (T08): `CLAUDE.md:83` — change "TrayController only
  executes signals — no decision logic" to something true, e.g.
  "`CommunicationPolicy` decides *what* to show; `TrayController` computes
  the per-tick inputs (elapsed seconds, standing laps, tooltip text) and
  calls it; `tray_signal_exec.rs` is the pure executor that turns a
  `Signal` into a UI call." Verified by
  `scripts/check-e019-t08-docs.mjs` (new file, part of this task's
  `write_set`) — it must fail if the old sentence is still present and pass
  once the new one is.

## Tasks

- [x] **T01** (3, ts-dev) — poison-safe mutex pattern in `commands.rs` (14
  sites) + new `commands_tests.rs` with a poisoned-lock regression test.
  Verify: `cargo test --manifest-path src-tauri/Cargo.toml --lib -- e019_t01_commands_poison`.
- [x] **T02** (8, ts-dev) — `.arch/ADR/014-persistence-precedence.md` +
  `today_totals.rs` (new) consolidating the duplicate "today's totals" path
  in `commands::get_today_summary`; doc comment in `commands_analyst.rs`
  pointing at the ADR. Verify:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib -- e019_t02_today_totals`.
- [x] **T03** (5, ts-dev) — `desk_events.rs` (new, 8 `pub const` event
  names) used at every emit/listen site; `src/events.ts` (new, mirrored
  constants only, no consumer rewiring). Verify:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib -- e019_t03_desk_events`
  and `npx vitest run --config vite.config.ts src/events.test.ts`.
- [x] **T04** (2, ts-dev) — `EventLogger::new` warns instead of panics;
  `generate_handler!` debug/release parity enforced by a test. Verify:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib -- e019_t04`.
- [x] **T05** (3, ts-dev) — split `google_fit.rs`/`google_fit_service.rs`
  into ≤250-line siblings. Verify:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib -- google_fit`.
- [x] **T06** (3, ts-dev) — split `serial_periodic::check_periodic`/
  `handle_reading` by responsibility. Verify:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib -- e019_t06_serial_periodic`.
- [x] **T07** (3, ts-dev) — dedupe `tray_controller`/`remote_server`
  remote-display-state derivation into `remote_display_state.rs`. Verify:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib -- e019_t07_remote_display_state`.
- [x] **T08** (1, main) — correct `CLAUDE.md`'s TrayController claim.
  Verify: `node scripts/check-e019-t08-docs.mjs`.

## Done means

All 10 acceptance criteria in PLAN.md hold, `cargo test --manifest-path
src-tauri/Cargo.toml --lib` reports 492+ passed / 0 failed, every evidence
record is `current`, `.plan/HISTORY.md` gets an entry, `STATE.md` is
updated, version is bumped per `.claude/rules/versioning.md` if this epic
starts the release cycle (check `Cargo.toml` for the current version first).

**Done (2026-09-06).** All eight tasks `[x]`. Run `E019-20260906-0822`,
promoted at `4649dc5`. Independently re-verified on `main` afterward: 533
Rust + 3 integration + 261 TS tests, `just check` exit 0. See
[JOURNAL.md](JOURNAL.md) for the run's two operator interventions (a
session-limit hit and a Windows Smart App Control block, neither a code
defect).

## Outside AO

None — all 8 tasks are automatable (7 Rust/TS code tasks with real test
assertions, 1 docs task with a script assertion). No manual browser pass is
required: none of these changes are user-visible UI (the browser-verification
exception for code-only backend changes still applies per
`CLAUDE.md`'s "Never claim it works" rule — `cargo test` is sufficient
evidence for internal refactors with no UI surface).

## AO

```yaml
project: MoveUp
epic: E019
base_ref: main
tasks:
  - id: E019-T01
    repo: MoveUp
    executor: ts-dev
    depends_on: []
    write_set: ["src-tauri/src/commands.rs", "src-tauri/src/commands_tests.rs", "src-tauri/src/lib.rs"]
    claims: ["src-tauri/src/commands.rs", "src-tauri/src/commands_tests.rs", "src-tauri/src/lib.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- e019_t01_commands_poison"
    budget_minutes: 45
  - id: E019-T08
    repo: MoveUp
    executor: main
    depends_on: []
    write_set: ["CLAUDE.md", "scripts/check-e019-t08-docs.mjs"]
    claims: ["CLAUDE.md", "scripts/check-e019-t08-docs.mjs"]
    verification: "node scripts/check-e019-t08-docs.mjs"
    budget_minutes: 30
  - id: E019-T02
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E019-T01"]
    write_set: ["src-tauri/src/commands.rs", "src-tauri/src/commands_analyst.rs", "src-tauri/src/today_totals.rs", "src-tauri/src/lib.rs", ".arch/ADR/014-persistence-precedence.md", ".arch/ARCHITECTURE.md"]
    claims: ["src-tauri/src/commands.rs", "src-tauri/src/commands_analyst.rs", "src-tauri/src/today_totals.rs", "src-tauri/src/lib.rs", ".arch/ADR/014-persistence-precedence.md"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- e019_t02_today_totals"
    budget_minutes: 60
  - id: E019-T03
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E019-T02"]
    write_set: ["src-tauri/src/desk_events.rs", "src-tauri/src/lib.rs", "src-tauri/src/serial.rs", "src-tauri/src/serial_periodic.rs", "src-tauri/src/commands.rs", "src-tauri/src/tray.rs", "src-tauri/src/tray_signal_exec.rs", "src-tauri/src/setup_helpers.rs", "src-tauri/src/tray_controller.rs", "src-tauri/src/ws_broadcaster.rs", "src/events.ts", "src/events.test.ts"]
    claims: ["src-tauri/src/desk_events.rs", "src-tauri/src/lib.rs", "src-tauri/src/serial.rs", "src-tauri/src/serial_periodic.rs", "src-tauri/src/commands.rs", "src-tauri/src/tray.rs", "src-tauri/src/tray_signal_exec.rs", "src-tauri/src/setup_helpers.rs", "src-tauri/src/tray_controller.rs", "src-tauri/src/ws_broadcaster.rs", "src/events.ts", "src/events.test.ts"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- e019_t03_desk_events"
    budget_minutes: 60
  - id: E019-T04
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E019-T03"]
    write_set: ["src-tauri/src/event_logger.rs", "src-tauri/src/lib.rs"]
    claims: ["src-tauri/src/event_logger.rs", "src-tauri/src/lib.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- e019_t04"
    budget_minutes: 45
  - id: E019-T05
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E019-T04"]
    write_set: ["src-tauri/src/google_fit.rs", "src-tauri/src/google_fit_service.rs", "src-tauri/src/google_fit_client.rs", "src-tauri/src/google_fit_tests.rs", "src-tauri/src/google_fit_service_tests.rs", "src-tauri/src/lib.rs"]
    claims: ["src-tauri/src/google_fit.rs", "src-tauri/src/google_fit_service.rs", "src-tauri/src/google_fit_client.rs", "src-tauri/src/google_fit_tests.rs", "src-tauri/src/google_fit_service_tests.rs", "src-tauri/src/lib.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- google_fit"
    budget_minutes: 60
  - id: E019-T06
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E019-T05"]
    write_set: ["src-tauri/src/serial_periodic.rs", "src-tauri/src/serial_periodic_reset.rs", "src-tauri/src/serial_periodic_tests.rs", "src-tauri/src/lib.rs"]
    claims: ["src-tauri/src/serial_periodic.rs", "src-tauri/src/serial_periodic_reset.rs", "src-tauri/src/serial_periodic_tests.rs", "src-tauri/src/lib.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- e019_t06_serial_periodic"
    budget_minutes: 60
  - id: E019-T07
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E019-T06"]
    write_set: ["src-tauri/src/tray_controller.rs", "src-tauri/src/remote_server.rs", "src-tauri/src/remote_display_state.rs", "src-tauri/src/lib.rs"]
    claims: ["src-tauri/src/tray_controller.rs", "src-tauri/src/remote_server.rs", "src-tauri/src/remote_display_state.rs", "src-tauri/src/lib.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- e019_t07_remote_display_state"
    budget_minutes: 45
```
