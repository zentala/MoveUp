# E011 — Orchestrator

**Linked from:** [PLAN.md](./PLAN.md)

## Wave 1 — independent (parallel)

- [x] **E011-T01** — Autostart self-heal + dev guard + log to events
  - Branch: `feat/E011-T01-autostart-self-heal`
  - Worktree: `.claude/worktrees/E011-T01-autostart-self-heal`
  - Touches: `apps/desk/src-tauri/src/setup_helpers.rs`, `Cargo.toml`, new `setup_helpers_tests.rs`
  - Depends on: nothing
  - Spec: [tasks/E011-T01-autostart-self-heal.md](./tasks/E011-T01-autostart-self-heal.md)

- [x] **E011-T02** — EventLogger investigation + write reliability
  - Branch: `feat/E011-T02-event-logger-fix`
  - Worktree: `.claude/worktrees/E011-T02-event-logger-fix`
  - Touches: `apps/desk/src-tauri/src/event_logger.rs`, new investigation file, new integration test
  - Depends on: nothing
  - Spec: [tasks/E011-T02-event-logger-fix.md](./tasks/E011-T02-event-logger-fix.md)

- [x] **E011-T03** — Commit pending OneBarTimeline.test.tsx fix
  - No worktree (single file, < 5 lines, exception per worktrees.md)
  - Touches: `apps/desk/src/widgets/one-bar/OneBarTimeline.test.tsx`
  - Depends on: nothing
  - Spec: [tasks/E011-T03-commit-test-fix.md](./tasks/E011-T03-commit-test-fix.md)

## Wave 2 — depends on T03 merged

- [x] **E011-T04** — Precommit gate: tsc --noEmit on desk app
  - Branch: `feat/E011-T04-precommit-tsc-gate`
  - Worktree: `.claude/worktrees/E011-T04-precommit-tsc-gate`
  - Touches: `.husky/pre-commit`, `apps/desk/package.json`
  - Depends on: T03 (test must already be fixed, otherwise the new gate would fail on existing code)
  - Spec: [tasks/E011-T04-precommit-tsc-gate.md](./tasks/E011-T04-precommit-tsc-gate.md)

## Wave 3 — depends on T01 merged

- [x] **E011-T05** — Autostart `--minimized` + popup hidden on autostart
  - Branch: `feat/E011-T05-minimized-startup`
  - Worktree: `.claude/worktrees/E011-T05-minimized-startup`
  - Touches: `apps/desk/src-tauri/src/lib.rs`, `apps/desk/src-tauri/src/setup_helpers.rs`
  - Depends on: T01 (both touch `setup_helpers.rs` — sequential to avoid conflicts)
  - Spec: [tasks/E011-T05-minimized-startup.md](./tasks/E011-T05-minimized-startup.md)

## Final integration

Closed 2026-09-06 by E016-T03. Boxes that were genuinely done are checked;
the rest are replaced by a note saying why they are moot. Nothing here is
checked off for work that did not happen.

- [x] All worktrees merged to `main`
- **`pnpm tauri:build` succeeds** — moot as an E011 gate. The build has run
  many times since on the same `main`, most recently for the `v0.5.0` release
  (E012); a retroactive E011-scoped build would prove nothing about the state
  of the tree in May.
- **Manual end-to-end smoke per PLAN.md acceptance criteria** — moot as a
  ceremony step: the autostart behaviour has been dogfooded daily since
  2026-05-07 (the fresh `desk.exe` built against the release registry path is
  recorded in `.plan/STATE.md`). No smoke record was written at the time and
  one cannot be written now honestly.
- **Bump version to `0.4.0`** — moot; the version progressed to `0.5.0` via
  E012 before this ceremony closed. A retroactive `0.4.0` would not reflect
  real history.
- **Tag `v0.4.0`** — moot, same reason. `v0.5.0` is the tag that exists.
- [x] Update history with epic summary — done as
  [`.plan/HISTORY.md`](../../HISTORY.md) § E011 (the file lives under `.plan/`,
  not `.arch/`, per the plan-structure migration).
- [x] Triage [`IMPROVEMENTS.md`](./IMPROVEMENTS.md) — triaged 2026-09-06, no
  entries, nothing promoted.
- [x] Mark in [`.plan/DONE.md`](../../DONE.md) — all five tasks recorded under
  "E011 — Autostart Hardening (2026-05-07)".
