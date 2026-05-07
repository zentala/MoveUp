---
id: E011-T03
epic: E011
status: done
created: 2026-05-07
completed: 2026-05-07
branch: main
---

# E011-T03 — Commit pending OneBarTimeline.test.tsx fix

## Goal

Commit the already-edited test file fix (added `idleSecs: 0` and `continuousComputerSecs: 0` to `baseProps`) so the codebase typechecks cleanly and Wave 2 (T04) can install a `tsc --noEmit` precommit gate without it failing on existing code.

## Files

- `apps/desk/src/widgets/one-bar/OneBarTimeline.test.tsx` — already edited in working tree

## No worktree

Single file, < 5 LOC, no design decisions — exception per `.claude/rules/worktrees.md`.

## Commit

```
fix(desk): add missing WidgetProps fields to OneBarTimeline.test
```

## Done when

- [x] File staged + committed on main
- [x] `pnpm exec tsc --noEmit` returns clean
