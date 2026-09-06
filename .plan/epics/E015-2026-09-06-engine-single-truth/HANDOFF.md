# E015 Handoff — one truth for the sitting counter

## TLDR

Implementing session reads only this file plus [`PLAN.md`](PLAN.md). T01
(Rust) first, then T02 (TS) and T03 (Rust) in parallel, then T04 docs and
T05 verification. Work in a worktree via `wt-add`, branch
`feat/E015-engine-single-truth`. Bump version to 0.6.0 first
(`.claude/rules/versioning.md`). Do not touch clock injection, persistence
unification or the frontend reducer merge (E018/E020).

## Decisions made by Paweł (2026-09-06, final)

- D1: default `break_credit_multiplier` = 3.0 in `standard` (45 min sit :
  15 min break), 2.0 in `relaxed`, `strict` unchanged. Update ADR 008 in the
  same commit.
- D2: `current_session_secs` is deleted from state, DTO and payload. The
  Debug tab (and only Debug) gets a new, honestly named field
  `secs_since_last_break` (seconds since the last position change). No
  timer, bar, colour or notification may read it; the DTO doc comment says so.
- D4: E015 ships before the public release (E017).
- Approach B from PLAN.md (strangler, no rewrite).

## Test naming for AO

Every new or inverted Rust test in this epic is named with the prefix
`e015_`; TS tests live under the paths listed in `verification`. Each task's
verification runs exactly its own tests. Zero matched tests is a failure.

## Mental model

- Credited counter: `SessionState.sitting_seconds`
  (`src-tauri/src/session_types.rs:82`), decremented in
  `session_breaks.rs:126-158` by `apply_break_credit`. This is the only
  number that may drive a limit timer, bar or colour.
- Raw daily counter: `sitting_seconds_total` (`session_types.rs:138`) for
  KPI only; compare it only with other raw counters.
- The wrong field: `current_session_secs` (`session_types.rs:114`, zeroed
  at `session_breaks.rs:34,63,85`, live-computed in `session_live.rs:26-33`,
  copied into the DTO at `session_reading.rs:143-157`). Consumers:
  `src/types.ts`, `src/hooks/useDesk.ts:106,170`,
  `src/hooks/useRemoteDesk.ts:74,96`, `src/hooks/remoteDesk.test-helpers.ts`,
  `src/widgets/one-bar/OneBarTimer.tsx:28,59-68`, `OneBarTimeline.tsx:49-52`,
  `useWidgetData.ts:39`, `DebugSection.tsx`,
  `src/components/SessionProgress.floating.test.tsx` (dead component).
- Test enshrining the bug: `src-tauri/src/session_tests_timers_live.rs:160-164`.
- Colour path that is already right: `src/widgets/one-bar/temperature.ts:29-32`
  (reads `limitUsedSecs`).

## Tasks

- [ ] **T01** (8, ts-dev Rust) — remove `current_session_secs` from state,
  DTO and payload; `limit_used_secs` is the DTO name for the credited value
  if it is not already exposed under a clear name; invert the reset test;
  add real-path scenario test (sit 30 min, stand 2 min, sit; DTO shows
  1800 − 120·m). Verify: `cargo test --lib`.
- [ ] **T02** (5, ts-dev) — TS consumers read the credited field; state in
  both hooks named `limitUsedSecs`; timer and colour from one value; DTO
  drift test against a JSON fixture emitted by a Rust test. Verify:
  `pnpm test:unit`, `pnpm typecheck`.
- [ ] **T03** (5, ts-dev Rust) — PostureBalance uses
  `sitting_seconds_total` vs `standing_seconds` (both raw);
  `break_credit` column on sessions table with migration in `db.rs`,
  written in `db_sessions.rs`, read by `commands_analyst.rs` instead of the
  threshold guess. Verify: `cargo test --lib -- db`.
- [ ] **T04** (2, main) — ADR 008 revision, `.arch/UX-FLOW.md:113,342-346`,
  `CLAUDE.md` Session Logic, `.arch/ARCHITECTURE.md`, create
  `.plan/decisions.jsonl` with D1 and D2.
- [ ] **T05** (1, verify + browser) — full `cargo test` and `pnpm test:unit`
  green; browser agent on the popup with `pnpm tauri:dev:mock` (mock sim
  passes through stand phases): timer after a stand is non-zero and its
  band matches the bar colour. Evidence records per PLAN.md.

## Outside AO

- T05 browser pass (browser agent, once, after wave 2 merges).
- Version bump to 0.6.0 and tag: main loop, after `ao promote`.

## AO

```yaml
project: MoveUp
epic: E015
base_ref: main
tasks:
  - id: E015-T01
    repo: MoveUp
    executor: ts-dev
    depends_on: []
    write_set: ["src-tauri/src/session_types.rs", "src-tauri/src/session_breaks.rs", "src-tauri/src/session_live.rs", "src-tauri/src/session_reading.rs", "src-tauri/src/session_persistence.rs", "src-tauri/src/session_tests_*.rs", "src-tauri/src/session_tests.rs", "src-tauri/src/session_daily.rs", "src-tauri/src/session_manager.rs", "src-tauri/src/commands_catalog_sources.rs", "src-tauri/src/ws_broadcaster.rs", "src-tauri/src/metrics/tests.rs", "src-tauri/src/lib.rs", "src-tauri/src/tray_controller.rs", "src-tauri/src/remote_server.rs", "src-tauri/src/ergonomic_profile.rs", "src-tauri/profiles/**"]
    claims: ["src-tauri/src/session_types.rs", "src-tauri/src/session_breaks.rs", "src-tauri/src/session_live.rs", "src-tauri/src/session_reading.rs", "src-tauri/src/session_tests_timers_live.rs", "src-tauri/src/session_tests_scenarios.rs", "src-tauri/src/session_tests.rs", "src-tauri/src/session_daily.rs", "src-tauri/src/session_manager.rs", "src-tauri/src/commands_catalog_sources.rs", "src-tauri/src/ws_broadcaster.rs", "src-tauri/src/metrics/tests.rs", "src-tauri/src/ergonomic_profile.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- e015_"
    budget_minutes: 90
  - id: E015-T02
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E015-T01"]
    write_set: ["src/types.ts", "src/hooks/**", "src/widgets/one-bar/**", "src/components/settings/DebugSection.tsx", "src/components/SessionProgress.floating.test.tsx", "src/test/**", "src-tauri/src/session_dto_fixture_tests.rs", "src-tauri/src/lib.rs"]
    claims: ["src/types.ts", "src/hooks/useDesk.ts", "src/hooks/useRemoteDesk.ts", "src/hooks/remoteDesk.test-helpers.ts", "src/widgets/one-bar/OneBarTimer.tsx", "src/widgets/one-bar/OneBarTimeline.tsx", "src/widgets/one-bar/useWidgetData.ts", "src/components/settings/DebugSection.tsx", "src/test/dto-drift.test.ts"]
    verification: "npx vitest run --config vite.config.ts src/hooks src/widgets/one-bar src/test/dto-drift.test.ts"
    budget_minutes: 60
  - id: E015-T03
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E015-T01"]
    write_set: ["src-tauri/src/notification_service.rs", "src-tauri/src/db.rs", "src-tauri/src/db_sessions.rs", "src-tauri/src/db_queries.rs", "src-tauri/src/db_tests.rs", "src-tauri/src/db_tests_break_credit.rs", "src-tauri/src/commands_analyst.rs", "src-tauri/src/commands_analyst_tests.rs", "src-tauri/src/session_breaks.rs", "src-tauri/src/serial_periodic.rs"]
    claims: ["src-tauri/src/notification_service.rs", "src-tauri/src/db.rs", "src-tauri/src/db_sessions.rs", "src-tauri/src/db_tests.rs", "src-tauri/src/db_tests_break_credit.rs", "src-tauri/src/commands_analyst.rs", "src-tauri/src/session_breaks.rs", "src-tauri/src/serial_periodic.rs"]
    verification: "cargo test --manifest-path src-tauri/Cargo.toml --lib -- e015_"
    budget_minutes: 60
  - id: E015-T04
    repo: MoveUp
    executor: main
    depends_on: ["E015-T01", "E015-T02", "E015-T03"]
    write_set: [".arch/ADR/008-proportional-break-credit.md", ".arch/UX-FLOW.md", ".arch/ARCHITECTURE.md", "CLAUDE.md", ".plan/decisions.jsonl", "scripts/check-e015-docs.mjs"]
    claims: [".arch/ADR/008-proportional-break-credit.md", ".arch/UX-FLOW.md", ".arch/ARCHITECTURE.md", "CLAUDE.md", ".plan/decisions.jsonl", "scripts/check-e015-docs.mjs"]
    verification: "node scripts/check-e015-docs.mjs"
    budget_minutes: 30
```

`scripts/check-e015-docs.mjs` (written by T04 itself, in its write set):
exits 1 if `.arch/UX-FLOW.md` still mentions `current_session_secs`, if
ADR 008 lacks `3.0`, or if `.plan/decisions.jsonl` is missing.

Wave 1 = T01 alone, then T02 and T03 in parallel (disjoint claims once T01
is merged — T01 and T03 both need to touch `session_breaks.rs`, T01 to drop
the dead field write, T03 to fix the PostureBalance comparison, so T03 now
depends on T01 instead of running alongside it; amended 2026-09-06 after
AO run `E015-20260906-0408` hit `write_set_out_of_scope_write` on both T01
and T03 — see the epic JOURNAL.md for the full list of files the original
write_sets missed).
Wave 2 = T04.

## Done means

All six acceptance criteria in PLAN.md hold, evidence records are
`current`, version 0.6.0 tagged, `.plan/HISTORY.md` entry written,
`STATE.md` updated.
