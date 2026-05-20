# Spike — Time-series visualization & timeline libraries (research)

- **Type**: research spike (long-term vision)
- **Status**: open (to be executed)
- **Created**: 2026-05-21
- **Triggered by**: E012-T07 Daily Timeline visual review (commit `9f8581a`).
  Donut/KPI redesign will use Recharts short-term; this spike scopes the
  longer-term migration of our custom SVG timeline to a library so we can
  spend less code on chart primitives and more on insight.
- **Output**: written research report at the same path, replacing this
  task body. Sections: shortlist, demos, recommendation, migration cost.

## Why now

Desk app is fundamentally a **time-series product**: every feature is a
function of "what was the state at minute X over day Y across week Z".
Today the analyst dashboard uses ~700 LoC of hand-rolled SVG to render
one timeline strip + per-day mini bars. The user wants more — zoom, brush,
multi-day overlay, status markers, possibly a Gantt-like view — and rolling
those features from scratch is slow vs. picking a library that has them.

The dashboard also has KPI panels that should likely become donuts /
gauges with numeric centers. A single library that handles BOTH the
timeline strip AND the KPI gauges would tighten the bundle and the
design system.

## Research questions

Answer each with concrete evidence (code snippets + bundle size + demo
URL screenshot) — no vague summaries.

### Q1. Charting libraries with timeline + KPI support

For each library, document:
- Bundle size when only the timeline + donut components are imported
  (use webpack-bundle-analyzer or equivalent on a min repro app).
- Does it support a "continuous state strip" (1 segment per minute,
  colored by state)? If not, what's the closest pattern?
- Does it support brushable + zoomable time axes out of the box?
- Donut / radial gauge component with numeric center label?
- License (must be MIT or Apache 2.0 for open-core compatibility).
- Tree-shakeability (per-component imports work?).

**Libraries to evaluate (minimum):**
- `recharts` — already shortlisted for KPI donuts
- `apache-echarts` (`echarts-for-react`) — has built-in `timeline`,
  `dataZoom`, `gauge`, heatmap calendar
- `@visx/*` (Airbnb) — low-level D3 in React; need to compose ourselves
- `nivo` — beautiful donuts; check time-series story
- `chart.js` + `react-chartjs-2` — popular, simpler than echarts
- `react-vis` (Uber) — discontinued? worth confirming
- `plotly.js-react` — interactive scientific; large bundle
- `d3` directly — control vs. effort baseline

### Q2. Specialised timeline / Gantt libraries

These are NOT general charting — they are purpose-built for "events on
a horizontal time axis", which is structurally close to our needs.

- `vis-timeline` (formerly vis.js Timeline) — event bands, brushing,
  zoom. Does the "continuous state strip" pattern work, or only point
  events?
- `react-calendar-timeline` — Gantt-like; multi-row resource × time
- `frappe-gantt` — minimal Gantt
- `dhtmlx-gantt` — heavyweight, commercial license tier — eliminate?
- `react-gantt-chart` — small community variants
- `timeline.js` (Knight Lab) — designed for narrative timelines; check
- `d3-timeline` — D3 plugin

For each: does it model **state ranges per resource** (our "day" = a
resource, "Sit/Stand/Walk/Away" = bar segments)? If not, the API will
fight us.

### Q3. Pure-TS libraries for time-data manipulation

The viz layer is half the story; the data layer matters too. Today
we use raw `Date` + ad-hoc helpers (`enumerateDays`, `aggregateDayTotals`,
`buildTimelineScale`). These could be replaced/augmented by:

- `date-fns` — already idiomatic; check if our helpers are reinvented
- `luxon` — Intl-aware, immutable Dates; better TZ handling
- `temporal` (TC39 proposal, polyfill) — future-proof
- `dayjs` — small alternative to moment
- `@js-temporal/polyfill` — Temporal API
- **Range / interval libraries**:
  - `interval-tree` / `@flatten-js/interval-tree` — O(log n) overlap
    queries (useful for "what segments overlap this hour bucket?")
  - `range-utils` / generic interval merging
- **Time-series specifically**:
  - `apache-arrow` (JS) — columnar in-browser; overkill?
  - `tsfresh-js` / `time-series-js` — feature extraction
  - `crossfilter` (Square) — D3-era multi-dimensional crosstab;
    powers `dc.js` dashboards; battle-tested for "filter by date range
    AND state AND chart updates everywhere"

**Question**: would adopting `crossfilter` as the data layer cut the
custom hooks (`useSnapshotsRange`, `useDataCatalog`, `useTimelineNav`)
in half? Test with a small repro.

### Q4. Heatmap / calendar / overview libraries

Bonus — for the "30+ days at a glance" view:
- `cal-heatmap` (Github-contributions style)
- `@nivo/calendar`
- `react-calendar-heatmap`

Lightweight overview component to pair with the deep timeline strip.

## Deliverables (in this same file when spike completes)

1. **Shortlist** of 2-3 candidates per category (charts, timeline,
   time-data, calendar).
2. **One mini-demo per shortlist candidate** — a sandbox or local
   `.html` file in `apps/desk/.plan/reports/spike-demos/` rendering
   the desk app's actual fixture data
   (`apps/desk/src/test/analyst-fixtures-builders.ts`) so apples-to-
   apples comparisons are possible.
3. **Recommendation matrix** — table comparing bundle size, license,
   timeline fit, KPI fit, time-data fit, learning curve, animation
   quality.
4. **Migration cost estimate** — for the top choice, what would it
   take to replace `TimelineDetail.tsx + DateNavigator.tsx +
   timeline-utils.ts + timeline-segments.ts`? Rough LoC delta,
   bundle delta, perf delta.
5. **No-go reasons** — explicitly list libraries that look good but
   fail for our use case, with one-line reasons.

## How to execute

- Spike timebox: 1 working day (8h). If a candidate needs >2h to
  evaluate, score it on what we know and move on.
- Set up a throwaway repo `~/scratch/timeline-spike` so the demos
  don't pollute the desk app's bundle.
- Use real fixture data — `buildSnapshots(today)` produces 504 rows
  spanning 7 days at 20-min intervals. Same shape as production.
- Take screenshots of each demo at 1280×800 (matches Analyst window
  size) so visual comparison is fair.
- Reuse the existing HTML mockup
  `apps/desk/.plan/epics/E012-2026-05-16-analyst-dashboard/tasks/E012-T07-preview.html`
  as the visual ground truth — anything we adopt should at minimum
  match its readability.

## Out of scope

- Implementation. This spike produces a written recommendation only;
  any migration is its own task.
- The donut/KPI redesign — that's E012-T08 (separate, short-term).
- Mobile (deck app) — desk app focus for now.

## Cross-references

- E012-T07 Daily Timeline implementation: `apps/desk/.plan/epics/E012-2026-05-16-analyst-dashboard/tasks/E012-T07-timeline-strip.md`
- ADR 013 DateNavigator merge decision: `apps/desk/.arch/ADR/013-date-navigator-merge.md`
- Fixture builder: `apps/desk/src/test/analyst-fixtures-builders.ts`
