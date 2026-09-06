---
id: E018-T09
epic: E018
status: pending
priority: medium
effort: medium
dependencies: []
tags: [frontend, analyst, ui]
created_at: 2026-09-06
points: 5
agent: ts-dev
branch: feat/E018-T09-analyst-layout-flip
---

# E018-T09: Analyst layout flip — date header on top, day-nav sticky at bottom

## Objective

Implement the layout E012 designed but never shipped: a centred date title
with `◀`/`▶` arrows at the top of the Analyst Explorer tab, the
`DateNavigator` moved to a sticky bottom strip.

## Full spec

Read the complete context, decisions, and acceptance criteria in
[`E012-T09-analyst-layout-flip.md`](../../E012-2026-05-16-analyst-dashboard/tasks/E012-T09-analyst-layout-flip.md)
— this task file does not repeat that content, only adds what's specific
to running it under E018/AO.

## What's different from the E012 original

- **Mockup-first is mandatory before this task is called done** (per
  `.claude/rules/ux-design-flow.md`, not called out in the original E012
  task since that predates the rule's current enforcement). See
  `HANDOFF.md`'s `## Outside AO` — show the change via the `/mockup` dev
  route before the browser-verification pass.
- E012's task references `apps/desk/src/analyst/...` paths from an older
  monorepo layout; this repo's paths are `src/analyst/...` (no `apps/desk/`
  prefix) — use the current paths.
- `T10` (`E018-T10`) depends on this task's `AnalystHeader.tsx` and the
  `ExplorerTab.tsx` restructuring existing before it starts — land this
  first.

## Acceptance Criteria

All acceptance criteria from `E012-T09-analyst-layout-flip.md` §Acceptance
criteria apply unchanged. Additionally:

- [ ] `npx vitest run --config vite.config.ts src/analyst/AnalystHeader.test.tsx` passes.
- [ ] The change was shown via the mockup route before this task is marked done.

## Cross-references

- Original spec: [`E012-T09-analyst-layout-flip.md`](../../E012-2026-05-16-analyst-dashboard/tasks/E012-T09-analyst-layout-flip.md)
- [`HANDOFF.md`](../HANDOFF.md) §Mental model — wave 3 file overlap, §Outside AO.
- Depended on by: [`E018-T10-recharts-donut-kpis.md`](E018-T10-recharts-donut-kpis.md)
