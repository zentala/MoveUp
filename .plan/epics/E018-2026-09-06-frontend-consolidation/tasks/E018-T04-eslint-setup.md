---
id: E018-T04
epic: E018
status: pending
priority: high
effort: small
dependencies: []
tags: [frontend, tooling]
created_at: 2026-09-06
points: 3
agent: ts-dev
branch: feat/E018-T04-eslint-setup
---

# E018-T04: Install and configure ESLint

## Objective

Make `pnpm lint` actually run. Today it calls `eslint src/` with no
`eslint` devDependency and no config file anywhere in the repo — it fails
immediately with "not recognized as an internal or external command"
([`frontend.md`](../../../reports/_review-2026-09-06/frontend.md) finding #1).

## Tasks

- [ ] Add devDependencies: `eslint`, `typescript-eslint`,
  `eslint-plugin-react-hooks`, `eslint-plugin-react-refresh` (matches the
  Vite + React + TS stack already in `package.json`).
- [ ] Add `eslint.config.mjs` (flat config): TypeScript recommended rules,
  React hooks rules (`eslint-plugin-react-hooks`'s recommended set — this
  catches the exact class of bug `useDesk`/`useRemoteDesk` are prone to:
  stale closures in `useEffect`/`useCallback`), ignore `src-tauri/`,
  `dist/`, `node_modules/`, `src/generated/` (T02's codegen output).
- [ ] Run `pnpm lint` and fix or `// eslint-disable-next-line <rule> —
  <reason>` any violations it surfaces. If a rule flags widespread
  pre-existing style (not a real bug) across many files outside this
  task's own touched files, set that rule to `"warn"` rather than `"error"`
  for this initial baseline, and file a `.plan/BACKLOG.md` entry to tighten
  it later — do not silently disable it repo-wide with no record.
- [ ] Do not touch files claimed by other Wave 1 tasks (T01, T03, T06) to
  fix lint violations in them — if `pnpm lint` fails only on lines in
  those files, downgrade the specific rule to `"warn"` for now and note it
  in the BACKLOG, since those tasks' own diffs may already resolve it.

## Four shadow paths

- **happy** — `pnpm lint` exits 0 with zero errors.
- **error** — a rule set to `"error"` on real code still fails until fixed;
  this is the point of the task (a lint command that always passes because
  every rule is downgraded to `"warn"` is worse than no lint command).

## Acceptance Criteria

- `pnpm lint` exits 0.
- `eslint.config.mjs` exists and is a flat config (not `.eslintrc*`).
- Any rule downgraded to `"warn"` as a pragmatic baseline is recorded in
  `.plan/BACKLOG.md` with a path:line and a plan to tighten it.

## Cross-references

- [`PLAN.md`](../PLAN.md) §Test strategy T04.
- [`HANDOFF.md`](../HANDOFF.md) §Mental model — tooling.
- Source review: [`frontend.md`](../../../reports/_review-2026-09-06/frontend.md) finding #1, §Recommended target structure item 3.
