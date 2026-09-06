# ADR 016 — Recharts for the Analyst KPI donuts

- **Status**: accepted
- **Date**: 2026-09-06
- **Epic**: E018 (task E018-T10), originally specified as E012-T10

## TLDR

The Analyst Explorer tab needed donut/gauge KPI cards with a numeric centre.
We took a charting library rather than hand-rolling more SVG, and picked
**Recharts 2.15.4** because it is React-native, tree-shakes down to the
`PieChart` path, and costs the least to adopt next to the SVG code already in
`src/analyst/charts/`. Measured bundle cost: **+101.9 kB gzipped**, inside the
120 kB budget the task set but not comfortably.

## Context

`src/analyst/charts/` is hand-written SVG: `scaleLinear`/`ticks` helpers in
`chart-utils.ts`, `<path>`/`<rect>` drawn by each chart. That was fine for
bars, sparklines and timelines. A donut needs arc geometry, an animated arc
transition on day change (E018-T11), and text centred inside the ring — three
things that are tedious and easy to get subtly wrong by hand.

## Decision

Add `recharts` as a direct dependency, pinned to `2.15.4`, and use only
`PieChart` / `Pie` / `Cell` / `Label` in `KpiDonut.tsx`. The rest of the
Analyst charts stay hand-rolled; this is not a mandate to migrate them.

Two implementation constraints that fall out of the choice:

- **No `ResponsiveContainer`.** It measures its parent, and under jsdom that
  measurement is zero, so the chart renders nothing and unit tests can only
  assert on an empty SVG. `KpiDonut` takes an explicit `size` prop instead.
- **The centre text is a `<Label content={...}>`** returning two stacked
  `<text>` nodes, so the numbers are real SVG inside the chart rather than an
  absolutely positioned overlay that can drift from the arc.

## Alternatives considered

| Option | Why not |
|---|---|
| Keep hand-rolling SVG | Arc maths plus a smooth animated transition is real work, and E018-T11 needs the animation hook. No reuse for the next chart. |
| Apache ECharts | Largest bundle of the four, imperative canvas API, wants its own React wrapper. Overkill for one donut. |
| Nivo | Nice defaults, but pulls d3 submodules and its own theming layer that would fight `chartColors`. |
| Chart.js (+ react-chartjs-2) | Canvas, so the centre text cannot be SVG and is not queryable in unit tests. |
| Visx | Lowest level of the four — it gives arc generators, not a chart, so most of the hand-rolling stays. Reasonable second choice. |

## Consequences

- Recharts 2.x is **deprecated upstream** in favour of 3.x. We took 2.15.4
  because the task specified `^2.x` and T11 builds directly on this component;
  migrating to 3.x is follow-up work, not a blocker. Recorded so it is not
  discovered as a surprise.
- Bundle, measured with `vite build` immediately before and after wiring the
  component up: total gzipped output went from **97.30 kB to 199.23 kB
  (+101.93 kB)**, essentially all of it in the lazily loaded `AnalystWindow`
  chunk (10.47 kB → 112.29 kB gzipped). The Analyst window is a separate Tauri
  window loaded on demand, so the main app's startup payload is unchanged.
  Tree-shaking the barrel import down further is possible only via deep paths
  (`recharts/es6/chart/PieChart`), which ship no adjacent type declarations and
  would break `tsc --noEmit`. Left as-is; revisit on the 3.x migration.
- One more dependency to keep current. It is used by exactly one component, so
  removing it later means rewriting `KpiDonut.tsx` only.

## Links

- Task: [`E018-T10`](../../.plan/epics/E018-2026-09-06-frontend-consolidation/tasks/E018-T10-recharts-donut-kpis.md)
- Original spec: [`E012-T10`](../../.plan/epics/E012-2026-05-16-analyst-dashboard/tasks/E012-T10-recharts-donut-kpis.md)
- Component: `src/analyst/charts/KpiDonut.tsx`
- [ADR 012 — Analyst dashboard in a separate window](012-analyst-dashboard-separate-window.md)
