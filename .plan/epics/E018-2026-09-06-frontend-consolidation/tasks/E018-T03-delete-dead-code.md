---
id: E018-T03
epic: E018
status: pending
priority: high
effort: small
dependencies: []
tags: [frontend, cleanup]
created_at: 2026-09-06
points: 3
agent: ts-dev
branch: feat/E018-T03-delete-dead-code
---

# E018-T03: Delete dead components and the overlay.html build entry

## Objective

Remove five pre-`OneBarWidget` components and the dead `overlay.html`
Vite build entry, all unreferenced outside their own tests
([`frontend.md`](../../../reports/_review-2026-09-06/frontend.md) findings #6, #7).

## Tasks

- [ ] Grep for live imports before deleting anything:
  `grep -rn "AppProgressBar\|HeightRail\|SessionProgress\|TodayStats\|TransitionBanner\|overlay/main" src --include=*.tsx --include=*.ts`
  — every hit must be inside the files listed below or their own test file.
  If a hit exists elsewhere, stop and report it instead of deleting.
- [ ] Delete `src/components/AppProgressBar.tsx` + `AppProgressBar.test.tsx`.
- [ ] Delete `src/components/HeightRail.tsx` + `HeightRail.test.tsx`.
- [ ] Delete `src/components/SessionProgress.tsx` +
  `SessionProgress.floating.test.tsx`.
- [ ] Delete `src/components/TodayStats.tsx` + `TodayStats.test.tsx`.
- [ ] Delete `src/components/TransitionBanner.tsx` +
  `TransitionBanner.test.tsx`.
- [ ] Delete `src/overlay/main.tsx` and `overlay.html`.
- [ ] Remove the `overlay: resolve(__dirname, "overlay.html")` entry from
  `vite.config.ts`'s `build.rollupOptions.input`.
- [ ] Write `scripts/check-e018-t03-dead-code.mjs`: a small Node script
  (see template below) that asserts all 7 deleted paths are absent, prints
  the count of paths checked, and exits non-zero if any still exists.

```js
// scripts/check-e018-t03-dead-code.mjs
import { existsSync } from "node:fs";

const deletedPaths = [
  "src/components/AppProgressBar.tsx",
  "src/components/HeightRail.tsx",
  "src/components/SessionProgress.tsx",
  "src/components/TodayStats.tsx",
  "src/components/TransitionBanner.tsx",
  "src/overlay/main.tsx",
  "overlay.html",
];

const stillPresent = deletedPaths.filter(existsSync);
if (stillPresent.length > 0) {
  console.error(`FAIL: ${stillPresent.length} dead path(s) still exist:`, stillPresent);
  process.exit(1);
}
console.log(`OK: confirmed ${deletedPaths.length} dead paths absent`);
```

## Four shadow paths

- **happy** — all 7 paths absent, script prints `OK: confirmed 7 dead paths absent`.
- **error** — any path still present → script exits 1 with the list (never
  a silent pass on partial deletion).

## Acceptance Criteria

- `node scripts/check-e018-t03-dead-code.mjs` exits 0.
- `pnpm typecheck` and `npx vitest run --config vite.config.ts` (full
  suite, run once locally to confirm no dangling import — not part of AO's
  scoped verification) both pass.
- `vite.config.ts` no longer references `overlay.html`.

## Cross-references

- [`PLAN.md`](../PLAN.md) §Test strategy T03.
- [`HANDOFF.md`](../HANDOFF.md) §Mental model — dead code.
- Source review: [`frontend.md`](../../../reports/_review-2026-09-06/frontend.md) findings #6, #7, §Recommended target structure item 2.
