# E019 — Backend hardening

Status: planned (2026-09-06). Version bump on start: 0.7.0 (after E015/E017/E018
land per `.plan/reports/2026-09-06-pelny-przeglad-architektury-i-release.md`
§6; if this epic starts first, bump from whatever `Cargo.toml` shows at the
time — see `.claude/rules/versioning.md`).
Source review: [`../../reports/_review-2026-09-06/backend.md`](../../reports/_review-2026-09-06/backend.md),
§3 and §6 of [`../../reports/2026-09-06-pelny-przeglad-architektury-i-release.md`](../../reports/2026-09-06-pelny-przeglad-architektury-i-release.md).
Board deck (Polish): [`PRES.md`](PRES.md). Handoff: [`HANDOFF.md`](HANDOFF.md).

## TLDR

The backend review found the Rust side generally solid (492/492 tests green)
but flagged eight concrete risks: `commands.rs` uses plain `.unwrap()` on
every mutex lock instead of the poison-safe pattern used elsewhere; four
persistence mechanisms disagree with no documented precedence; 8 event names
are stringly typed with no shared constants; `EventLogger` panics where its
sibling `SnapshotLogger` only warns; the debug/release `generate_handler!`
lists can silently drift; `google_fit.rs`/`google_fit_service.rs` breach the
250-line file cap; `serial_periodic::check_periodic`/`handle_reading` do five
jobs in one function; and `tray_controller`/`remote_server` independently
re-derive the same remote-display state. This epic fixes all eight, corrects
the one documentation claim in `CLAUDE.md` that no longer matches the code,
and does **not** attempt a Tauri-free domain crate — the reviewer explicitly
recommends against it because the session/policy logic already tests cleanly
without a running Tauri app. 28 points, 8 tasks, mostly sequential because
`commands.rs` and `lib.rs` are each touched by more than one task.

## Problem

- `src-tauri/src/commands.rs:44,74,75,80,86,89,95,97,102,155,166,182,193,199,216`
  — 14 plain `.unwrap()` calls on `Mutex::lock()`. Any panic while a lock is
  held anywhere in the app (e.g. inside `session.on_reading`) poisons that
  mutex; every subsequent IPC call through `commands.rs` then panics too,
  because it does not recover with `unwrap_or_else(|e| e.into_inner())` the
  way `tray_controller.rs`, `tray_signal_exec.rs`, and `remote_server.rs`
  already do. `commands.rs` is the hottest, most load-bearing IPC file — one
  panic anywhere takes down the whole IPC surface.
- Four persistence mechanisms (SQLite `sessions` table, `tauri-plugin-store`,
  per-minute JSON snapshots, plain-text `events.log`) overlap with no
  document saying which wins. "Today's totals" is independently computed at
  `commands.rs:60-82` (`ensure_initialized`, DB + store), again in
  `commands::get_today_summary` (DB via `db_queries::get_today_summary` +
  live `position_changes` patched from the in-memory session), and Analyst
  charts read a third source (`commands_analyst.rs:129-159`,
  `collect_snapshots`) that was never meant to be an alternative
  implementation of the same scalar but reads that way from the outside. A
  drift between the DB-derived summary and the live session state is
  currently undetectable at runtime.
- `src-tauri/src/tray_controller.rs`, `serial.rs`, `serial_periodic.rs`,
  `commands.rs`, `tray.rs`, `tray_signal_exec.rs`, `setup_helpers.rs`,
  `ws_broadcaster.rs` — 8 distinct `desk:*` Tauri event names, each retyped
  as a string literal at every `emit`/`listen` site. A typo at any of the
  ~15 call sites fails silently (`listen` never fires, no compile error).
- `src-tauri/src/event_logger.rs:25-35` — `EventLogger::new` panics
  (`unwrap_or_else(|e| panic!(...))`) if the log directory cannot be
  created, and this runs early in `perform_app_setup`, before the tray or
  serial scan exist — a permissions problem there crashes the whole app at
  startup. `snapshot_logger.rs:39-43`, structurally identical, only warns.
- `src-tauri/src/lib.rs:155-232` — two hand-maintained `generate_handler!`
  lists (debug/test vs release) differing by exactly `inject_reading` and
  `get_overlay_state`. Nothing checks they stay in sync; a new command added
  to one list and forgotten in the other silently only exists in one build
  profile.
- `src-tauri/src/google_fit.rs` (473 lines) and
  `src-tauri/src/google_fit_service.rs` (328 lines) both breach the repo's
  250-line file cap (`CLAUDE.md`, `rules/code-style.md`), unlike every other
  oversized cluster, which already follows the `<module>_tests.rs` /
  `<module>_helpers.rs` split convention (`communication_profile*.rs`,
  `tray_controller.rs` → `tray_signal_exec.rs`,
  `commands_catalog.rs` → `commands_catalog_sources.rs`).
- `src-tauri/src/serial_periodic.rs:21-100,104-224` —
  `check_periodic`/`handle_reading` each do five unrelated jobs (daily
  reset, snapshot logging, notification dispatch, profile hot-reload,
  DB writes) inline, contradicting the file's own doc comment ("Extracted
  ... to keep serial.rs focused on I/O").
- `src-tauri/src/tray_controller.rs:167-186` and
  `src-tauri/src/remote_server.rs:159-190` — `broadcast_remote_state` and
  `build_remote_display_state` independently re-derive the identical
  `RemoteDisplayState { session, metrics, today }` shape from the same three
  `Arc<Mutex<_>>` fields (`session`, `comm_policy`, `today_cache`), with two
  separate lock-ordering sequences. A metrics-computation change must be
  made twice.
- `CLAUDE.md:83` — "TrayController only executes signals — no decision
  logic" is false: `tray_controller.rs:53-186` computes `PolicyInput`,
  standing-lap math, tooltip text, and remote broadcasting itself. The real
  pure executor is `tray_signal_exec.rs`.

## Decisions and ADRs

- `.plan/decisions.jsonl` does not exist yet — none to read. T02 does not
  create it (E015 already claimed that file for D1/D2); this epic's durable
  decision is architectural and gets a full ADR instead.
- No existing ADR covers persistence precedence. T02 creates
  `.arch/ADR/014-persistence-precedence.md` (this epic's file, this repo
  numbers ADRs under `.arch/ADR/`, not `.plan/ADR/` — see
  `.arch/ARCHITECTURE.md`).
- Reviewer's explicit recommendation (`backend.md` §Recommended target
  structure, point 4): do **not** extract a Tauri-free domain crate. This
  plan follows that recommendation — see Alternatives below.
- Depends on **E015** for anything touching `session_types.rs`,
  `db_sessions.rs`, or `session_persistence.rs` internals — those files are
  claimed by E015's engine-single-truth migration. No task in this epic
  edits those three files; T02's `today_totals.rs` only *calls* their
  existing public functions.

## Alternatives

| | A. Minimum (per-finding patches only) | B. Targeted hardening (this plan) | C. Tauri-free domain crate |
|---|---|---|---|
| Summary | Fix only the two `High`-importance findings (mutex, persistence doc) and stop | Fix all 8 findings + the doc claim, following the review's own recommended target structure | Extract `session`/`policy` into a crate with no `tauri`/`AppHandle` dependency, force every module onto it |
| Effort | S (11) | M (28) | XL (55+) |
| Risk | L | M | H |
| Pros | Cheapest; closes the two riskiest gaps | Closes every named gap while every file stays ≤250 lines; matches existing split conventions exactly | "Cleanest" on paper |
| Cons | Leaves 6 real findings (event-name typos, panic-vs-warn asymmetry, handler drift, two oversized files, a 5-job function, duplicate broadcast logic) unaddressed for another review cycle | Touches `lib.rs` in 6 of 8 tasks, so most tasks must run sequentially, not in parallel | Reviewer explicitly says this is not worth it: 492 tests already pass without a running Tauri app; a crate boundary would not remove a single `AppHandle` parameter from the hot call paths (`serial_periodic.rs`, `commands_*.rs`), it would just move where the parameter is declared |
| Reuses | `unwrap_or_else(\|e\| e.into_inner())` pattern already in 3 files | Same, plus the `<module>_tests.rs`/`<module>_helpers.rs` split convention already used 4+ times in this codebase | Nothing — new crate boundary, new `Cargo.toml`, new dependency graph |

**Recommendation: B.** The review is explicit that C is over-engineering for
a single-user desktop app and would not remove any of the eight concrete
problems it found — it would just relocate them behind a crate boundary.
A (fix only High findings) leaves real, named risk on the table for no
schedule benefit: every remaining finding was already scoped, already has a
`path:line`, and none require touching E015's claimed files. B is the
smallest plan that closes the whole review, at the review's own recommended
target structure.

## Scope

**In**: `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`,
`src-tauri/src/commands_analyst.rs`, `src-tauri/src/event_logger.rs`,
`src-tauri/src/serial.rs`, `src-tauri/src/serial_periodic.rs`,
`src-tauri/src/tray.rs`, `src-tauri/src/tray_signal_exec.rs`,
`src-tauri/src/tray_controller.rs`, `src-tauri/src/setup_helpers.rs`,
`src-tauri/src/ws_broadcaster.rs`, `src-tauri/src/remote_server.rs`,
`src-tauri/src/google_fit.rs`, `src-tauri/src/google_fit_service.rs`, new
Rust files this epic creates (`today_totals.rs`, `desk_events.rs`,
`remote_display_state.rs`, google_fit split siblings, serial_periodic split
siblings, new `*_tests.rs` files), `.arch/ADR/014-persistence-precedence.md`,
`CLAUDE.md` (Communication Architecture section, one sentence), one new file
`src/events.ts` (constants only — no consumer rewiring, that is E018's job).

**Out**: `session_types.rs`, `session_manager.rs`, `session_breaks.rs`,
`session_live.rs`, `session_reading.rs`, `session_daily.rs`,
`session_persistence.rs`, `db_sessions.rs`, `db.rs`'s schema (all E015);
rewriting TS consumers of `desk:*` events (E018); the frontend reducer merge
(E018); clock injection, config-out-of-state, one persistence snapshot,
signals-from-`step()` (E020); a Tauri-free domain crate (explicitly rejected
above); code signing and release docs (E017).

## Tasks

| ID | Task | Points | Agent | Wave |
|---|---|---|---|---|
| T01 | `commands.rs`: 14 `.lock().unwrap()` → poison-safe `unwrap_or_else(\|e\| e.into_inner())`; new `commands_tests.rs` with a poisoned-lock regression test | 3 | ts-dev | 1 |
| T02 | ADR-014 (persistence precedence) + `today_totals.rs` (new): one function both `commands::get_today_summary` and the analyst doc comment point to as the authoritative "today's totals" path | 8 | ts-dev | 2 |
| T03 | `desk_events.rs` (Rust `pub const` event names, used at all 8×N `emit`/`listen` sites) + `src/events.ts` (mirrored constants, new file, not yet consumed) | 5 | ts-dev | 3 |
| T04 | `EventLogger::new` warns instead of panics (matches `SnapshotLogger`); `generate_handler!` parity enforced by a test that fails if the two lists diverge | 2 | ts-dev | 4 |
| T05 | Split `google_fit.rs`/`google_fit_service.rs` into ≤250-line siblings following the `_client`/`_tests` convention already used elsewhere | 3 | ts-dev | 5 |
| T06 | Split `serial_periodic::check_periodic`/`handle_reading` by responsibility (daily-reset, snapshot-logging, notification-dispatch, profile-reload) | 3 | ts-dev | 6 |
| T07 | Dedupe `tray_controller::broadcast_remote_state` / `remote_server::build_remote_display_state` into one `remote_display_state.rs` helper | 3 | ts-dev | 7 |
| T08 | Docs: correct `CLAUDE.md`'s "TrayController only executes signals" claim to what the code does | 1 | main | 1 |

Every wave after wave 1 is exactly one task — see HANDOFF.md → Mental model
for why (both `commands.rs` and `lib.rs` are touched by most tasks, so
running two of them in parallel would double-claim a file). T08 runs
parallel to T01 in wave 1 (docs-only, zero file overlap with any Rust task).

## Test strategy

Four shadow paths per new/changed behavior, per `rules/testing.md`:

- **T01 — happy/nil/empty/error**: happy = a normal `.lock()` still returns
  `Ok` data unchanged (existing tests cover this; verify they still pass).
  Error = the new regression test manually poisons `AppState.session`
  (spawn a thread that panics while holding the lock, join and ignore the
  panic) then calls a `commands.rs` function through the same `AppState`
  and asserts it returns data instead of panicking. Fails today (plain
  `.unwrap()` panics on a poisoned lock). nil/empty do not apply — a lock is
  never itself nil or empty.
- **T02 — happy/nil/empty/error**: happy = `today_totals::resolve(...)` on a
  DB with two completed sessions returns the same numbers
  `commands::get_today_summary` returns today, asserted by a new unit test
  that fails today (function does not exist). nil = no sessions today →
  zeroed totals, not an error. empty = DB open but `sessions` table empty →
  same as nil (already covered by existing `db_tests.rs`, referenced not
  duplicated). error = DB connection `None` → `today_totals::resolve`
  returns `Err`, asserted.
- **T03 — happy/nil/empty/error**: happy = a Rust unit test asserts
  `desk_events::STATE_CHANGED == "desk:state-changed"` (and 7 more, one per
  event) so any future rename breaks compilation at the `emit`/`listen`
  call site, not at runtime. A companion Vitest test in the same commit
  reads `src/events.ts` and asserts the same 8 string values, byte-for-byte
  against the Rust constants (a small JSON fixture, same drift-test pattern
  as E015-T02). Fails today (no `desk_events.rs`, no `src/events.ts`).
  nil/empty/error do not apply — these are compile-time string constants.
- **T04 — happy/nil/empty/error**: happy = `EventLogger::new` on a normal
  writable dir still works (existing test). error = `EventLogger::new` on an
  unwritable dir (e.g. a path under a file, not a directory) returns/logs a
  warning and the struct is still usable (falls back to per-write retry the
  way `SnapshotLogger`/`ensure_day_dir` already does) instead of panicking —
  new test, fails today (current code panics). Handler parity: a new test
  parses `lib.rs`'s source at compile time is not feasible in a unit test;
  instead the test asserts a single canonical `Vec<&str>` of command names
  used to build both `generate_handler!` invocations agrees with a hardcoded
  expected list — see T04 task file for the exact mechanism decided during
  implementation (cfg-gated single list is preferred if `generate_handler!`
  supports a runtime-built list; otherwise the test enumerates both macro
  outputs via `tauri::generate_handler!`'s public introspection, or, failing
  that, the test asserts the two `match` arms in a new tiny
  `registered_commands()` function used by both cfg branches). Fails today
  (no such function exists).
- **T05 — happy/nil/empty/error**: existing `google_fit_http_tests.rs` /
  `google_fit_service_tests.rs` must still pass unchanged after the split
  (mechanical move, not a rewrite) — this is the regression net. No new
  behavior, so no new happy/nil/empty/error path; the "test" here is that
  the existing 2xx/4xx/5xx and revoked-auth suites (already covering
  error/empty) keep passing post-split.
- **T06 — happy/nil/empty/error**: existing `serial_periodic` tests (if any)
  or, if none exist today, a new test per extracted function: happy = daily
  reset fires the reset event when due; nil = no notification events this
  tick → `NotificationService::dispatch` receives an empty slice, not called
  with garbage; empty = `check_notification_conditions` returns `vec![]` →
  no panic, no persisted-state write; error = DB write failure on
  `insert_session` inside `handle_reading` is logged via `error!`, not
  propagated as a panic (already true — regression test locks this in).
- **T07 — happy/nil/empty/error**: happy = new `remote_display_state::build`
  called with a session that has one completed session today returns
  identical `RemoteDisplayState` from both call sites (tray tick and REST
  poll) — one test replaces the need for two. error = a poisoned
  `comm_policy` mutex still returns a value (poison-safe pattern applies
  here too). nil/empty = an empty `today_cache` (zeroed `TodaySummary`)
  round-trips as zeros, not a panic — existing test pattern, adapted.
- **T08 — happy/nil/empty/error**: docs-only. The "test" is
  `scripts/check-e019-t08-docs.mjs`: happy = `CLAUDE.md` contains the new,
  accurate sentence; error = the old false claim ("TrayController only
  executes signals — no decision logic") is still present → script exits 1.
  nil/empty do not apply to a doc-content assertion.

## Evidence contract

| check_id | class | procedure | expected | record |
|---|---|---|---|---|
| mutex-poison | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- e019_t01_commands_poison` | pass | `evidence/records/T01-mutex-poison.json` |
| today-totals | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- e019_t02_today_totals` | pass | `evidence/records/T02-today-totals.json` |
| desk-events-rust | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- e019_t03_desk_events` | pass | `evidence/records/T03-desk-events-rust.json` |
| desk-events-ts | test | `npx vitest run --config vite.config.ts src/events.test.ts` | pass | `evidence/records/T03-desk-events-ts.json` |
| logger-parity | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- e019_t04` | pass | `evidence/records/T04-logger-parity.json` |
| google-fit-split | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- google_fit` | pass | `evidence/records/T05-google-fit-split.json` |
| serial-periodic-split | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- e019_t06_serial_periodic` | pass | `evidence/records/T06-serial-periodic-split.json` |
| remote-display-dedupe | test | `cargo test --manifest-path src-tauri/Cargo.toml --lib -- e019_t07_remote_display_state` | pass | `evidence/records/T07-remote-display-dedupe.json` |
| docs-tray-claim | manual | `node scripts/check-e019-t08-docs.mjs` | exits 0, prints the corrected sentence found | `evidence/records/T08-docs-tray-claim.json` |

## Architecture impact

- `.arch/ARCHITECTURE.md`: no structural change to the component map — the
  file already shows `tray_signal_exec.rs` as the executor separate from
  `tray_controller.rs`; only `CLAUDE.md`'s prose claim is wrong (T08).
- New ADR: `.arch/ADR/014-persistence-precedence.md` (T02), status
  `accepted`, linked from `.arch/ARCHITECTURE.md`'s persistence section (add
  one line if that section does not already exist — see T02 task file).
- No new component in the diagram; `today_totals.rs`, `desk_events.rs`,
  `remote_display_state.rs`, and the google_fit/serial_periodic split
  siblings are internal helper modules at the same layer as their parents,
  not new architectural layers.

## Acceptance criteria

1. `grep -c "\.lock()\.unwrap()"` on `src-tauri/src/commands.rs` returns 0.
2. `.arch/ADR/014-persistence-precedence.md` exists, status `accepted`, and
   states which of the four mechanisms is authoritative for "today's
   totals"; `commands::get_today_summary` and the Analyst doc comment both
   reference it.
3. `src-tauri/src/desk_events.rs` exists with 8 named constants, used at
   every `emit`/`listen` call site listed in the backend review's Event/IPC
   inventory; `src/events.ts` mirrors the same 8 strings.
4. `EventLogger::new` no longer panics on a dir-create failure.
5. A test fails if the debug and release `generate_handler!` lists diverge.
6. `google_fit.rs` and `google_fit_service.rs` (and every new sibling file)
   are each ≤250 lines.
7. `serial_periodic::check_periodic` and `handle_reading` are each ≤50 lines
   (per `rules/code-style.md`'s function-size rule), with responsibilities
   split into named helper functions/files.
8. `tray_controller::broadcast_remote_state` and
   `remote_server::build_remote_display_state` both call one shared
   function; no duplicated derivation logic remains.
9. `CLAUDE.md`'s Communication Architecture section no longer claims
   "TrayController only executes signals — no decision logic".
10. `cargo test --manifest-path src-tauri/Cargo.toml --lib` still reports
    492+ passed, 0 failed after all eight tasks land.
