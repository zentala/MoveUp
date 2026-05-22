---
id: E011-T04
epic: E011
status: pending
created: 2026-05-07
branch: feat/E011-T04-precommit-tsc-gate
worktree: .claude/worktrees/E011-T04-precommit-tsc-gate
depends_on: E011-T03
title: E011-T04 — Precommit gate: tsc --noEmit on desk app
---

# E011-T04 — Precommit gate: tsc --noEmit on desk app

## Goal

Block commits that introduce TypeScript type errors in `apps/desk/`. Mirror the existing `apps/tray` gate so behavior is consistent across the monorepo.

## Files

- `.husky/pre-commit` (root of monorepo) — append a desk-app block after the existing tray-app block
- `apps/desk/package.json` — add `"typecheck": "tsc --noEmit"` script for clean invocation

## Implementation

Append to `.husky/pre-commit`:

```sh
DESK_STAGED=$(git diff --cached --name-only --diff-filter=ACMR | grep -E '^apps/desk/.*\.(ts|tsx)$' || true)
if [ -n "$DESK_STAGED" ]; then
  echo "[precommit] desk app TS files staged → running tsc --noEmit"
  (cd apps/desk && pnpm exec tsc --noEmit) || {
    echo "[precommit] desk tsc failed — fix errors before commit"
    exit 1
  }
fi
```

In `apps/desk/package.json` scripts:
```json
"typecheck": "tsc --noEmit"
```

## Smoke

1. `Add-Content apps/desk/src/temp-bad.ts "const x: string = 123;"`
2. `git add apps/desk/src/temp-bad.ts`
3. `git commit -m "test"` → must FAIL with `[precommit] desk tsc failed`
4. `git reset HEAD apps/desk/src/temp-bad.ts; Remove-Item apps/desk/src/temp-bad.ts`
5. Stage a valid file, commit → passes

## Done when

- [ ] Hook blocks bad commit, allows good commit
- [ ] `pnpm typecheck` runs in desk app dir
- [ ] Committed as `chore(monorepo): add desk tsc precommit gate`
