---
id: E018-T05
epic: E018
status: pending
priority: high
effort: medium
dependencies: ["E018-T01"]
tags: [frontend, tooling]
created_at: 2026-09-06
points: 5
agent: ts-dev
branch: feat/E018-T05-coverage-gate-justfile
---

# E018-T05: Wire the coverage gate into `pnpm build`; add a `justfile`

## Objective

`vite.config.ts:22-31` defines a real 80/80/75% coverage threshold, but
`pnpm build`/`pnpm tauri:build` call `test:all` (`test:unit && test:unit:rust`)
and never `test:coverage` — the gate has never blocked a build
([`frontend.md`](../../../reports/_review-2026-09-06/frontend.md) finding #2).
This repo also has no `justfile`, contradicting the standing convention
that every repo exposes `just build`/`just dev`/`just test`/`just check`
(finding #14, `~/.claude/rules/just.md`).

## Tasks

- [ ] Change `package.json`'s `"build"` script (currently
  `"pnpm test:all && tsc && vite build"`) to run `test:coverage` instead
  of (or in addition to) `test:unit`, so the coverage threshold actually
  gates the build.
- [ ] Run `pnpm test:coverage` and read the result. After T01's reducer
  split, `useDesk.ts`/`useRemoteDesk.ts` should be meaningfully more
  covered than the 76.5/68.1/66.0% baseline in the source review.
- [ ] **Decision D4**: if lines/functions/branches now meet 80/80/75%,
  leave `vite.config.ts`'s thresholds as-is. If not, lower them in
  `vite.config.ts` to the measured number, with a comment
  `# lowered 2026-0X-XX, was blocking every build at 0% enforcement — see BACKLOG #<n>`,
  and add that `.plan/BACKLOG.md` entry in the same commit. Do not leave
  the original 80/80/75% numbers in place if the build cannot pass them —
  that reproduces the exact bug this task fixes (a threshold nobody
  actually meets, silently never enforced).
- [ ] Add `justfile` at the repo root per `~/.claude/rules/just.md`'s
  template: `setup` (`pnpm install`), `dev` (`pnpm tauri:dev`), `build`
  (`pnpm build`), `test` (`pnpm test:unit`), `check` (`pnpm typecheck &&
  pnpm lint && pnpm test:unit`... note T04 must have landed for `lint` to
  be real — if T04 hasn't merged yet when this task runs, `check` still
  references `pnpm lint`; it will simply fail until T04 lands, which is
  correct), `lint` (`pnpm lint`), `typecheck` (`pnpm typecheck`), `clean`
  (remove `dist/`, `src-tauri/target/`). Include
  `set windows-shell := ["pwsh", "-NoProfile", "-Command"]` — this repo
  runs on Windows.
- [ ] Add `src/hooks/useDesk.test.ts` coverage improvements if needed to
  clear the threshold — this may overlap with T01's own test file of the
  same name; if T01 already wrote it, extend it rather than replacing it.

## Four shadow paths

- **happy** — `pnpm build` runs coverage, meets threshold, exits 0.
- **error** — coverage under threshold on a fresh clone (no prior
  artifacts) fails the build loudly, not silently.

## Acceptance Criteria

- `pnpm build` runs `test:coverage` (or equivalent) as part of the build
  pipeline and exits 0.
- `vite.config.ts`'s thresholds either meet reality or were lowered with a
  dated comment and a linked `.plan/BACKLOG.md` entry (D4).
- `justfile` exists at the repo root with at least `setup`, `dev`, `build`,
  `test`, `check`, `lint`, `typecheck`, `clean`.

## Cross-references

- [`PLAN.md`](../PLAN.md) §Decisions and ADRs (D4), §Test strategy T05.
- [`HANDOFF.md`](../HANDOFF.md) §Outside AO — D4 needs Paweł's confirmation
  if thresholds must be lowered.
- Source review: [`frontend.md`](../../../reports/_review-2026-09-06/frontend.md) finding #2, #14, §Recommended target structure item 3.
