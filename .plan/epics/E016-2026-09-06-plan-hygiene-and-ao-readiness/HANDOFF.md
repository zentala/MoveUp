# E016 Handoff — plan hygiene and AO readiness

## TLDR

Five docs/config-only tasks, 8 points total, one wave (T03 waits on T02
because both touch `.plan/BACKLOG.md`). No application code changes. Every
task verifies with a small Node script under `scripts/`, already written.
Implementing session reads only this file plus
[`PLAN.md`](PLAN.md).

## Decisions already made (apply unless Paweł overrides)

- Root `BACKLOG.md`/`TASKS.md`/`ORCHESTRATOR.md` are deleted, not archived —
  their unique content is merged into `.plan/BACKLOG.md` first, verbatim.
- E011's "Final integration" checklist items that are now moot (retroactive
  version bump/tag) are recorded as moot with a one-line reason, never
  silently checked off as if performed.
- E013 is marked `superseded` in its own `PLAN.md`, split note: waves 1-2 →
  E017, waves 3-5 → E014. E013's file itself is not deleted — kept for its
  acceptance-criteria research.
- AO precondition 4 (no repo hook writes into the tree post-commit) is
  already satisfied — confirmed by absence of `.claude/settings.json` and
  `.husky/` in this repo. No guard task is created; a note is recorded
  instead (see E016-T05's own file).
- `.giter.yaml`'s `worktree.copy` may be empty — this repo has no `.env`
  file and no nested workspace to copy gitignored fixtures from.

## Mental model

- Where to edit: repo root (`.plan/STATE.md`, `.plan/BACKLOG.md`,
  `.plan/HISTORY.md` — new file, `BACKLOG.md`/`TASKS.md`/`ORCHESTRATOR.md` —
  deleted, `.gitignore`, new `.giter.yaml`, new `justfile`), plus
  `.plan/epics/E011-2026-05-07-autostart-hardening/ORCHESTRATOR.md` and
  `IMPROVEMENTS.md`, and
  `.plan/epics/E013-2026-08-28-signed-tauri-pm3-deployment/PLAN.md`.
- What NOT to touch: any file under `src/` or `src-tauri/src/` — this epic
  is documentation and repo config only. Do not run `pnpm tauri:build`, do
  not retag `v0.4.0` (moot, see Decisions above), do not touch E011's or
  E013's task files themselves (only their `ORCHESTRATOR.md`/`PLAN.md`).
- How to verify each task: run its own `node scripts/check-e016-t0N.mjs` —
  each is self-contained, reads the files it checks, and exits non-zero
  with a bulleted reason list on failure. Do not hand-verify by eyeballing
  the files; the scripts encode every acceptance criterion.

## Tasks

- [ ] **T01** (1, main) — Fix `.plan/STATE.md` body to match its frontmatter;
  reconcile E010's 3-vs-10 human-task count.
  Verify: `node scripts/check-e016-t01.mjs`
- [ ] **T02** (2, main) — Merge root `BACKLOG.md`+`TASKS.md` into
  `.plan/BACKLOG.md` verbatim; delete the three root files; fix
  README/CONTRIBUTING links.
  Verify: `node scripts/check-e016-t02.mjs`
- [ ] **T03** (2, main, depends on T02) — Close E011's ceremony
  (`.plan/HISTORY.md`, IMPRO triage note); mark E003-T07 and E013
  superseded.
  Verify: `node scripts/check-e016-t03.mjs`
- [ ] **T04** (1, main) — `git rm -r --cached coverage
  test-performance-report` + `.gitignore` entries.
  Verify: `node scripts/check-e016-t04.mjs`
- [ ] **T05** (2, main) — Create root `.giter.yaml` and `justfile` for AO
  readiness; record precondition-4 check result.
  Verify: `node scripts/check-e016-t05.mjs`

## Done means

All five acceptance-criteria checklists in `PLAN.md` hold, all five
`scripts/check-e016-t0N.mjs` exit 0, `.plan/STATE.md` reflects this epic's
own closure (add a line to `current_wave` noting E016 done, E017 next),
`.plan/HISTORY.md` carries an E016 entry alongside the E011/E012 ones it
creates.

## Outside AO

None — every task in this epic is docs/config with a mechanical `node`
verification; nothing here requires a human or a browser pass.

## AO

```yaml
project: MoveUp
epic: E016
base_ref: main
tasks:
  - id: E016-T01
    repo: MoveUp
    executor: main
    depends_on: []
    write_set: [".plan/STATE.md", "scripts/check-e016-t01.mjs"]
    claims: [".plan/STATE.md", "scripts/check-e016-t01.mjs"]
    verification: "node scripts/check-e016-t01.mjs"
    budget_minutes: 30
  - id: E016-T02
    repo: MoveUp
    executor: main
    depends_on: []
    write_set: ["BACKLOG.md", "TASKS.md", "ORCHESTRATOR.md", ".plan/BACKLOG.md", "README.md", "CONTRIBUTING.md", "scripts/check-e016-t02.mjs"]
    claims: ["BACKLOG.md", "TASKS.md", "ORCHESTRATOR.md", ".plan/BACKLOG.md", "README.md", "CONTRIBUTING.md", "scripts/check-e016-t02.mjs"]
    verification: "node scripts/check-e016-t02.mjs"
    budget_minutes: 45
  - id: E016-T03
    repo: MoveUp
    executor: main
    depends_on: ["E016-T02"]
    write_set: [".plan/HISTORY.md", ".plan/BACKLOG.md", ".plan/epics/E011-2026-05-07-autostart-hardening/ORCHESTRATOR.md", ".plan/epics/E011-2026-05-07-autostart-hardening/IMPROVEMENTS.md", ".plan/epics/E013-2026-08-28-signed-tauri-pm3-deployment/PLAN.md", "scripts/check-e016-t03.mjs"]
    claims: [".plan/HISTORY.md", ".plan/BACKLOG.md", ".plan/epics/E011-2026-05-07-autostart-hardening/ORCHESTRATOR.md", ".plan/epics/E011-2026-05-07-autostart-hardening/IMPROVEMENTS.md", ".plan/epics/E013-2026-08-28-signed-tauri-pm3-deployment/PLAN.md", "scripts/check-e016-t03.mjs"]
    verification: "node scripts/check-e016-t03.mjs"
    budget_minutes: 45
  - id: E016-T04
    repo: MoveUp
    executor: main
    depends_on: []
    write_set: [".gitignore", "coverage/**", "test-performance-report/**", "scripts/check-e016-t04.mjs"]
    claims: [".gitignore", "scripts/check-e016-t04.mjs"]
    verification: "node scripts/check-e016-t04.mjs"
    budget_minutes: 20
  - id: E016-T05
    repo: MoveUp
    executor: main
    depends_on: []
    write_set: [".giter.yaml", "justfile", "scripts/check-e016-t05.mjs"]
    claims: [".giter.yaml", "justfile", "scripts/check-e016-t05.mjs"]
    verification: "node scripts/check-e016-t05.mjs"
    budget_minutes: 30
```
