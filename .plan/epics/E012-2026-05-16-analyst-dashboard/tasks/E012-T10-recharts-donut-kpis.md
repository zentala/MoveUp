---
id: E012-T10
epic: E012
status: pending
created: 2026-05-21
branch: feat/E012-T10-recharts-donut-kpis
title: Recharts donut KPIs with numeric center
---

# E012-T10: Recharts donut KPIs with numeric center

## Context

User wants the Analyst KPI panels (currently `KpiTrend` vertical bars +
`BreakCreditHistogram` bars) replaced with **donut/gauge charts that
show the percentage on the arc and big numeric values in the centre**.
Example layout per KPI:

```
   ┌─────────────┐
   │   ◌◌◌◌◌◌    │   ← donut arc (filled = standingPct%)
   │  ◌      ◌   │
   │ ◌   62%  ◌  │   ← centre primary (the percentage)
   │  ◌ 4h 32m ◌ │   ← centre secondary (the time it represents)
   │   ◌◌◌◌◌◌    │
   │  Standing % │   ← label below
   └─────────────┘
```

Library: **Recharts** (decision in ADR 014 — see below). Long-term
library evaluation is in a separate spike
(`apps/desk/.plan/reports/2026-05-21-time-viz-libraries-spike.md`)
but for "donut now" Recharts is the lowest-risk pick.

## Out of scope

- Layout flip — T09 ships that first.
- Pulse animation on day change — T11.
- Replacing TimelineDetail with a charting library — that's the
  long-term spike's recommendation.

## Decisions

- **Library: Recharts** (`recharts`, MIT). New ADR 014 captures the
  alternatives (Apache ECharts, Nivo, Chart.js, Visx) and why Recharts
  wins for this task: smallest tree-shaken bundle for `PieChart` only,
  React-native API, mature.
- **Donuts compute against `selectedDay`, not the whole range.** Range
  view stays in `DailyScoreTrajectory` (already shows per-day series);
  the donuts give the "today's snapshot" feel.
- **Numeric centre uses two stacked `<text>` SVG nodes** (Recharts
  exposes the chart container as plain SVG so labels overlay cleanly).
- **At least 3 donuts shipped in v1**: Standing % (of active time),
  Position changes (today), Daily score (0-100).
- **Hours formatter reused** — `formatDurationShort` from
  `@/utils/format`.

## Scope (concrete)

### New files

- `apps/desk/src/analyst/charts/KpiDonut.tsx` — generic donut card.
  Props: `label`, `valuePct: number` (0..1), `centerPrimary: string`,
  `centerSecondary?: string`, `color?: string` (defaults to
  `chartColors.standing`). ≤180 LoC.
- `apps/desk/src/analyst/charts/KpiDonutPanel.tsx` — composes 3-4
  donuts side by side using flex/grid. ≤120 LoC.
- `apps/desk/src/analyst/explorer-day-kpis.ts` — pure helper
  `aggregateDayKpis(snapshots, sessions, date) → { standingPct,
  positionChanges, score, ...}`. ≤120 LoC.
- Tests for each, mirroring naming.
- ADR `apps/desk/.arch/ADR/014-recharts-for-kpi-donuts.md`.

### Modified files

- `apps/desk/src/analyst/ExplorerTab.tsx` — swap the
  `KpiTrend` + `BreakCreditHistogram` cells for `<KpiDonutPanel>`
  driven by `aggregateDayKpis(..., selectedDay)`.
- `apps/desk/package.json` — `+ "recharts": "^2.x.x"`.

### Bundle budget

`pnpm build` baseline before install captured to task notes.
Recharts (only the PieChart + ResponsiveContainer imports) target
delta: **≤ +120 KB gzipped**. If over, audit which submodules are
pulled in and tree-shake harder.

## Acceptance criteria

- [ ] `recharts` declared as a direct dep with a pinned minor.
- [ ] ≥ 3 donut KPI cards rendered on the Analyst Explorer tab.
- [ ] Each donut: arc fill matches `valuePct`, centre shows
  `centerPrimary` (large) and `centerSecondary` (smaller below).
- [ ] Donuts react to `selectedDay` (data recomputes; arc transitions
  smoothly).
- [ ] `aggregateDayKpis` unit-tested for: zero data, all-sit, all-stand,
  partial day.
- [ ] Bundle delta ≤ 120 KB gzipped vs. baseline; numbers committed
  to task notes.
- [ ] ADR 014 written and cross-linked from CLAUDE.md.
- [ ] All new files ≤ 250 LoC; total suite still green.

## Cross-references

- T09 (layout flip prerequisite): `tasks/E012-T09-analyst-layout-flip.md`
- T11 (selectedDay pulse on donuts): `tasks/E012-T11-selected-day-pulse.md`
- Library spike: `apps/desk/.plan/reports/2026-05-21-time-viz-libraries-spike.md`
- Future ADR 014: `apps/desk/.arch/ADR/014-recharts-for-kpi-donuts.md`
