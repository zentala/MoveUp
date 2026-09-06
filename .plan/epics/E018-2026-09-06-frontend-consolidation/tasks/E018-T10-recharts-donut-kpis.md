---
id: E018-T10
epic: E018
status: pending
priority: medium
effort: large
dependencies: ["E018-T09"]
tags: [frontend, analyst, ui]
created_at: 2026-09-06
points: 8
agent: ts-dev
branch: feat/E018-T10-recharts-donut-kpis
---

# E018-T10: Recharts donut KPIs with numeric centre

## Objective

Implement the Recharts-based donut KPI cards E012 designed but never
shipped, replacing `KpiTrend` bars + `BreakCreditHistogram` on the Analyst
Explorer tab.

## Full spec

Read the complete context, decisions, and acceptance criteria in
[`E012-T10-recharts-donut-kpis.md`](../../E012-2026-05-16-analyst-dashboard/tasks/E012-T10-recharts-donut-kpis.md)
— this task file does not repeat that content, only adds what's specific
to running it under E018/AO.

## What's different from the E012 original

- Depends on `E018-T09` (this task's `ExplorerTab.tsx` edits build on
  T09's layout restructuring; do not start before T09 lands).
- E012's task references `apps/desk/...` paths; this repo's paths have no
  `apps/desk/` prefix — use `src/analyst/charts/...`,
  `src/analyst/ExplorerTab.tsx`, `.arch/ADR/016-recharts-for-kpi-donuts.md`
  (not `apps/desk/.arch/ADR/...`).
- ADR number 014 is pre-claimed by the E012 task file's own text — keep
  it as 014 (E018-T02's codegen ADR uses 015, chosen specifically to avoid
  this collision — see `PLAN.md` §Decisions and ADRs).
- **Mockup-first is mandatory before this task is called done** (per
  `.claude/rules/ux-design-flow.md`) — show the donuts via the `/mockup`
  route before the browser-verification pass covers this and T09/T11
  together.
- `T11` (`E018-T11`) depends on `KpiDonut.tsx`/`KpiDonutPanel.tsx` existing
  before it starts — land this before T11.

## Acceptance Criteria

All acceptance criteria from `E012-T10-recharts-donut-kpis.md` §Acceptance
criteria apply unchanged, with paths adjusted per "What's different" above.
Additionally:

- [ ] `npx vitest run --config vite.config.ts src/analyst/charts/KpiDonut.test.tsx` passes.
- [ ] The change was shown via the mockup route before this task is marked done.

## Cross-references

- Original spec: [`E012-T10-recharts-donut-kpis.md`](../../E012-2026-05-16-analyst-dashboard/tasks/E012-T10-recharts-donut-kpis.md)
- Depends on: [`E018-T09-analyst-layout-flip.md`](E018-T09-analyst-layout-flip.md)
- Depended on by: [`E018-T11-selected-day-pulse.md`](E018-T11-selected-day-pulse.md)
- [`HANDOFF.md`](../HANDOFF.md) §Mental model — wave 3 file overlap, §Outside AO.
