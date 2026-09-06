---
id: E018-T06
epic: E018
status: pending
priority: low
effort: small
dependencies: []
tags: [frontend, cleanup]
created_at: 2026-09-06
points: 3
agent: ts-dev
branch: feat/E018-T06-consolidate-format-helpers
---

# E018-T06: Consolidate local formatXxx helpers into utils/format.ts

## Objective

Six local date/duration formatting helpers duplicate the concern
`src/utils/format.ts` already exists to own
([`frontend.md`](../../../reports/_review-2026-09-06/frontend.md) finding #9).

## Tasks

- [ ] Move `formatDate` (`src/analyst/charts/TimelineDetailHeader.tsx:28`)
  into `src/utils/format.ts`, export it, import it back at the call site.
- [ ] Move `formatRefreshedAt` and `formatRangeLabel`
  (`src/analyst/ExplorerTab.tsx:44,54`) the same way.
- [ ] Move `formatIdleTime` (`src/components/StateIndicator.tsx:24`) the
  same way.
- [ ] Move `formatTime` (`src/widgets/one-bar/OneBarTimeline.tsx:37`) the
  same way.
- [ ] Where a moved function's logic overlaps with the existing
  `formatDuration`/`formatDurationShort` primitives in `utils/format.ts`,
  compose from them instead of keeping separate padding/rounding logic.
- [ ] Add or extend `src/utils/format.test.ts` to cover the newly-moved
  functions' existing test cases (carry over assertions from each
  satellite file's test, don't just delete coverage).

## Four shadow paths

- **happy** — each moved function produces the same output as before for
  its existing call sites (regression-free move).
- **nil** — a `null`/`undefined` timestamp input is handled the same way
  post-move as pre-move (check each function's current guard before
  moving it).
- **empty** — an empty/zero duration input (e.g. `0` seconds) still
  formats sensibly.
- **error** — an invalid date string does not throw uncaught out of the
  formatter.

## Acceptance Criteria

- `src/utils/format.ts` exports all 6 previously-local functions.
- No file outside `utils/format.ts` defines its own `formatDate`,
  `formatRefreshedAt`, `formatRangeLabel`, `formatIdleTime`, or
  `formatTime`.
- `npx vitest run --config vite.config.ts src/utils/format.test.ts` passes.
- All changed files ≤ 250 lines.

## Cross-references

- [`PLAN.md`](../PLAN.md) §Test strategy T06.
- [`HANDOFF.md`](../HANDOFF.md) §Mental model — format consolidation.
- Source review: [`frontend.md`](../../../reports/_review-2026-09-06/frontend.md) finding #9, §Recommended target structure item 5.
