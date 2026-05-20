# Spike — Timeline visual upgrades (demo gallery)

- **Type**: design spike (visual exploration)
- **Status**: open (to be executed)
- **Created**: 2026-05-21
- **Triggered by**: E012-T07 visual review — current `TimelineDetail` is
  functional but feels utilitarian. Before adopting a heavier library
  (separate spike at `2026-05-21-time-viz-libraries-spike.md`), see how
  far we can take our own SVG strip with pure CSS / SVG tricks.
- **Output**: an HTML gallery at
  `apps/desk/.plan/reports/spike-demos/timeline-visual-upgrades.html`
  with 5-8 side-by-side variants of the same TimelineDetail rendered
  with different visual treatments. User picks 1-2 directions to
  productise; this file becomes the recommendation summary.

## Goal

Show the user concrete visual options for improving the existing
timeline before committing engineering effort. Each demo is a static
HTML rendering of the same 14-day fixture data with one stylistic
direction applied.

## Variants to prototype (minimum)

Each variant gets a panel in the gallery. Use the same data so the
comparison is honest.

### V1. Current baseline
Today's render with hour ticks + dark scrollbar. Reference for "what we
already ship".

### V2. Soft gradients per segment
Each state segment fills with a subtle vertical gradient (top 15%
lighter, bottom 5% darker) instead of flat fill. Adds depth without
adding chartjunk. CSS-only.

### V3. State silhouette / cardiogram
Instead of stacked horizontal rects, render each state as a vertical
"heartbeat" amplitude (Standing = tall spike, Sitting = mid, Away =
flat baseline). Time on X axis, intensity on Y. Inspired by Apple
Health step graphs.

### V4. Hour-bucketed dot matrix
Per hour: 4 dots (Sit/Stand/Walk/Away) sized by share of that hour. So
each day = 24 mini-stacks of 4 dots. Lighter density, easier multi-
day overview.

### V5. Stream graph (overlapping ribbons)
Render the day as overlapping ribbons summing to 100% width, one
ribbon per state. Time on X. Visualises composition rather than
sequence. Closer to Spotify Wrapped aesthetic.

### V6. Editorial typographic header per day
Keep the segments as-is but add big italic weekday name + small
ALL-CAPS day number anchored to the left of each day's segment block.
References the "Instrument Serif" treatment from the original
HTML mockup that didn't make it into the React build.

### V7. Animated time cursor with breadcrumb trail
When hovering the strip, a vertical cursor follows the mouse and shows
a "what was happening here" tooltip (timestamp + state + duration of
contiguous segment). On click → a breadcrumb pin stays visible. Pure
SVG + React.

### V8. Mini-multiples row instead of one giant strip
14 small strips (one per day), each 24h wide, stacked vertically. Easier
to compare "what time did I start standing on Tuesday vs Thursday".
Mirrors small-multiples principle (Tufte).

## Optional variants if time allows

- **V9. Heatmap calendar** — Github contributions style for the last
  30/60 days as a single panel above the detail strip.
- **V10. Vertical timeline** — rotate the strip 90° so time runs
  top-to-bottom (Apple Health Sleep style). Different mental model
  entirely.

## How to execute

- One HTML file at
  `apps/desk/.plan/reports/spike-demos/timeline-visual-upgrades.html`
  with all variants embedded.
- Reuse the same dark theme tokens (#0a0a12 / #2a2a3a / sit/stand/walk/
  away colors) so visual comparison is consistent with the live app.
- Use real fixture shape — copy ~14 days of mock snapshots inline
  (no actual data fetch). Match the structure produced by
  `buildSnapshots(today)` from `analyst-fixtures-builders.ts`.
- Each variant gets:
  - Short heading (V1 — Baseline, V2 — Gradients, ...)
  - 1-2 sentence rationale
  - The actual render at ~1100×120 px so they stack nicely
- Open the file in Chrome at end of spike, user reviews and picks
  directions to productise (likely 1-2 wins, rest discarded).

## Deliverables

1. The HTML gallery file (with all variants).
2. Update THIS spike file with: ranked recommendation, screenshots
   committed alongside, and 1-3 follow-up tasks proposed
   (e.g. "V6 typographic header → new task in E012 or E013").
3. Any new tasks created get linked from `TASKS.md` and the
   `BACKLOG.md` "Visualization & Charts" section.

## Out of scope

- Implementation in the React app — this is HTML-only mockups.
- The library-research spike (separate file).
- Donut KPI design (covered by T10).
- Rebuilding TimelineDetail in production — that's a follow-up task
  after the user picks variants.

## Cross-references

- Long-term library spike: `2026-05-21-time-viz-libraries-spike.md`
- E012-T07 implementation:
  `apps/desk/.plan/epics/E012-2026-05-16-analyst-dashboard/tasks/E012-T07-timeline-strip.md`
- E012-T07 HTML mockup (origin of typographic ideas):
  `apps/desk/.plan/epics/E012-2026-05-16-analyst-dashboard/tasks/E012-T07-preview.html`
- Fixture builder for honest demo data:
  `apps/desk/src/test/analyst-fixtures-builders.ts`
