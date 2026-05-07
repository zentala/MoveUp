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

- [x] All worktrees merged to `main`
- [ ] `pnpm tauri:build` succeeds
- [ ] Manual end-to-end smoke per PLAN.md acceptance criteria
- [ ] Bump version to `0.4.0` per `.claude/rules/versioning.md` (new epic = MAJOR bump)
- [ ] Tag `v0.4.0`
- [ ] Update `.arch/HISTORY.md` with epic summary
- [ ] Triage `IMPROVEMENTS.md` with user
- [ ] Mark in `.plan/DONE.md`
