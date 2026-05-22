---
id: E012-T03
epic: E012
status: pending
created: 2026-05-16
branch: feat/E012-T03-mockup
title: E012-T03 — Frontend mockup: `/mockup/analyst` with fake data
---

# E012-T03 — Frontend mockup: `/mockup/analyst` with fake data

## Goal
Build the full Analyst UI as a mockup route driven by hardcoded fake fixtures. zentala opens `http://localhost:1443/#/mockup/analyst`, evaluates both tabs visually, asks for changes until approved. **No real Tauri data wired up in this task.**

## Why
`.claude/rules/ux-design-flow.md` mandates mockup-first for any UI change. Skipping this step has bitten this project before. The mockup is the contract for T05/T06 — once approved, frontend tasks copy its component tree and swap fixtures for real data.

## Scenario fixtures
Add to `apps/desk/src/test/scenarios.ts`:
- `analystCatalogFixture: DataCatalog` — 8 sources matching T02 shape
- `analystSnapshotsFixture: SnapshotRow[]` — 7 × 1440 = 10 080 entries (or downsampled to ~500 for perf), realistic state transitions
- `analystEventsFixture: EventRow[]` — ~200 entries spanning 7 days, mix of all 8 event kinds
- `analystSessionsFixture: SessionRow[]` — 7 days of sessions, ~3-6/day

## Route
Add `apps/desk/src/routes/mockup/analyst.tsx`:
- Renders `<AnalystWindow>` with fixtures injected via context
- Wired into existing `App.tsx` hash router under `/mockup/analyst`

## Components (new under `apps/desk/src/analyst/`)

```
analyst/
  AnalystWindow.tsx       — tab shell, date range picker, header
  CatalogTab.tsx          — table of sources, expandable rows with schema
  CatalogSourceRow.tsx    — one row
  ExplorerTab.tsx         — grid of 5 chart cards
  charts/
    DeskHeightTimeline.tsx       — line chart, height_cm over time
    StateGantt.tsx               — Sitting/Standing/Away horizontal bars per day
    DailyScoreTrajectory.tsx     — line chart, score by hour, one line per day
    BreakCreditHistogram.tsx     — bar chart, count of none/partial/full
    KpiTrend.tsx                 — small multiples for standing%, changes, longest
  DateRangePicker.tsx     — from/to (defaults: 7 days ago → today)
```

## Visual constraints
- Match desk app theme (read `.claude/rules/timeline-theme` if present; use semantic colors only)
- No UTF emoji icons — SVG line icons (or MUI Icons if already imported)
- Each chart card has: title, subtitle (1 line "what this shows"), legend, axis labels with units, hover tooltip
- Catalog table is sortable by name | kind | row_count | bytes
- Date range picker reuses existing `<DateInput>` component if there is one; else simple two `<input type="date">`
- Page width fluid; minimum 1280 px target (Analyst window default size)

## Tests
- `pnpm test:unit` covers each chart renders with the fixture without throwing
- One snapshot test for `<CatalogTab>` with fixture (Vitest + jsdom)

## Files touched
- `apps/desk/src/test/scenarios.ts` — add 4 fixtures
- `apps/desk/src/routes/mockup/analyst.tsx` (new)
- `apps/desk/src/analyst/**` (new directory)
- `apps/desk/src/App.tsx` — route registration

## DoD
- [ ] `pnpm dev` running; `http://localhost:1443/#/mockup/analyst` renders both tabs without errors
- [ ] zentala has opened the page and given verbal/written approval (recorded in JOURNAL.md)
- [ ] Unit tests pass; each chart component has at least 1 test
- [ ] Files ≤ 250 lines each
- [ ] No `console.log`, no `any` types in new code

## Notes for the implementing agent
- DO NOT register a new Tauri window in this task — T04 owns that
- DO NOT call any `invoke()` in this task — purely fixture-driven
- Charts: prefer **uPlot** (already in tree? grep first) or **Recharts**; if neither, propose in JOURNAL and ask before pulling a new dep
