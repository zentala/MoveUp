---
id: "E016-T05"
title: "AO readiness: .giter.yaml and root justfile"
status: pending
priority: high
effort: small
dependencies: []
tags: [ao, tooling]
created_at: 2026-09-06
---

# AO readiness — `.giter.yaml` and root `justfile`

## Objective

`skills/ao/SKILL.md` §1 lists five preconditions before any epic can be
dispatched through the Agent Orchestrator. This repo has neither of the two
config files AO needs (confirmed: no `.giter.yaml`, no `justfile`/`Justfile`
at repo root), and per `rules/just.md` every repo Claude touches should have
one regardless of AO. This task creates both, plus records that precondition
4 (no repo hook writes into a worktree after a worker's commit) is already
satisfied.

## Tasks

- [ ] Create root `.giter.yaml`. `worktree.copy` must list every gitignored
      file a build in a fresh worktree needs — this repo has **no** `.env`
      file anywhere (confirmed: `test -f .env` and `test -f src-tauri/.env`
      both report missing, and there is no `apps/` directory — the repo root
      layout is flat), so `copy` may be empty or omitted; state this
      explicitly in a comment rather than guessing a file into existence.
      `worktree.guards` must cover `node_modules` and `src-tauri/target`
      (the two directories `.gitignore` already excludes that a build
      recreates) — for a workspace with only one `node_modules` at root, a
      plain `node_modules` guard is sufficient (no nested pnpm workspace
      here, confirmed via `.gitignore`'s single `node_modules/` line and
      absence of a `pnpm-workspace.yaml`).
- [ ] Create root `justfile` per `rules/just.md`'s template, with
      `set windows-shell := ["pwsh", "-NoProfile", "-Command"]` (this repo
      runs on mATX), and targets `setup`/`dev`/`build`/`test`/`check`/
      `typecheck`/`lint`/`clean` wrapping the existing `package.json`
      scripts one-for-one (no logic moves into the justfile — see
      `package.json` scripts already present: `build`, `test:unit`,
      `test:unit:rust`, `test:all`, `typecheck`, `lint`, `dev`,
      `tauri:dev`). `check` = `typecheck` + `test:unit` + `cargo test`
      (i.e. depends on `typecheck` and the two test recipes, matching
      `package.json`'s own `test:all`). `lint` maps to `pnpm lint` even
      though the underlying `eslint` install is broken today (confirmed:
      `pnpm lint` fails — no `eslint` dependency/config) — fixing that is
      E018 scope, not this task's.
- [ ] Record precondition 4 explicitly: this repo has no `.claude/settings.json`
      and no `.husky/` directory (confirmed by direct listing 2026-09-06),
      so no repo hook writes into the tree after a worker's commit. Add one
      line to this task's own file (below, in the Notes section) rather than
      creating a guard for a hook that does not exist — do not invent
      protection against a non-existent mechanism.
- [ ] Run `node scripts/check-e016-t05.mjs` and fix until it exits 0.

## Notes — precondition 4 check result

Confirmed 2026-09-06: `find . -maxdepth 2 -iname ".husky"` and
`ls -la .claude/settings.json` both report nothing found. No hook in this
repo can dirty a worktree after a worker's last commit. Nothing to guard.
If a `.husky/` or `.claude/settings.json` is added later, re-run this check
before the next `ao run`.

## Acceptance Criteria

- `node scripts/check-e016-t05.mjs` exits 0.
- `just --list` exits 0 and lists all eight required targets.
- `.giter.yaml` has non-empty `worktree.guards` covering `node_modules` and
  `src-tauri/target`.

## Verify

```
node scripts/check-e016-t05.mjs
```
