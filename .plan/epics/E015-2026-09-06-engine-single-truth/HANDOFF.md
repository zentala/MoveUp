# E015 Handoff — one truth for the sitting counter

## TLDR

Implementing session reads only this file plus [`PLAN.md`](PLAN.md). One
wave of three parallel tasks (T01 Rust, T02 TS, T03 Rust), then T04 docs and
T05 verification. Work in a worktree via `wt-add`, branch
`feat/E015-engine-single-truth`. Bump version to 0.6.0 first
(`.claude/rules/versioning.md`). Do not touch clock injection, persistence
unification or the frontend reducer merge (E018/E020).

## Decisions already made (apply unless Paweł overrides)

- D1: default `break_credit_multiplier` = 3.0 in `standard`, 2.0 in
  `relaxed`, `strict` unchanged. Update ADR 008 in the same commit.
- D2: `current_session_secs` is deleted, not renamed. If Debug tab wants
  "seconds since last position change", add `secs_since_last_break` as a
  clearly separate field never used by the limit bar.
- Approach B from PLAN.md (no rewrite).

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

## Done means

All six acceptance criteria in PLAN.md hold, evidence records are
`current`, version 0.6.0 tagged, `.plan/HISTORY.md` entry written,
`STATE.md` updated.
