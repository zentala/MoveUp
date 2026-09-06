---
id: E018-T11
epic: E018
status: pending
priority: medium
effort: small
dependencies: ["E018-T10"]
tags: [frontend, analyst, ui]
created_at: 2026-09-06
points: 3
agent: ts-dev
branch: feat/E018-T11-selected-day-pulse
---

# E018-T11: Pulse highlight on KPI donuts when selectedDay changes

## Objective

Implement the CSS-only pulse animation E012 designed but never shipped, so
KPI donut cards briefly highlight when the user navigates to a different
day.

## Full spec

Read the complete context, decisions, and acceptance criteria in
[`E012-T11-selected-day-pulse.md`](../../E012-2026-05-16-analyst-dashboard/tasks/E012-T11-selected-day-pulse.md)
— this task file does not repeat that content, only adds what's specific
to running it under E018/AO.

## What's different from the E012 original

- Depends on `E018-T10` (needs `KpiDonut.tsx`/`KpiDonutPanel.tsx` to exist).
- E012's task references `apps/desk/...` paths; use
  `src/analyst/charts/KpiDonut.tsx`, `KpiDonutPanel.tsx`, `kpi-donut.css`
  (no `apps/desk/` prefix).
- **Mockup-first is mandatory before this task is called done** (per
  `.claude/rules/ux-design-flow.md`) — the pulse must be shown live (CSS
  animation isn't visible in a static screenshot; use the mockup route's
  live day-navigation to trigger it) before the browser-verification pass.

## Acceptance Criteria

All acceptance criteria from `E012-T11-selected-day-pulse.md` §Acceptance
criteria apply unchanged, with paths adjusted per "What's different" above.
Additionally:

- [ ] `npx vitest run --config vite.config.ts src/analyst/charts/KpiDonut.test.tsx` passes.
- [ ] The pulse was demonstrated live (mockup route, changing `selectedDay`)
  before this task is marked done.

## Cross-references

- Original spec: [`E012-T11-selected-day-pulse.md`](../../E012-2026-05-16-analyst-dashboard/tasks/E012-T11-selected-day-pulse.md)
- Depends on: [`E018-T10-recharts-donut-kpis.md`](E018-T10-recharts-donut-kpis.md)
- [`HANDOFF.md`](../HANDOFF.md) §Mental model — wave 3 file overlap, §Outside AO.
