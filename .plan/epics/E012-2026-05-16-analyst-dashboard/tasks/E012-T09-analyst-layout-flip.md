---
id: E012-T09
epic: E012
status: pending
created: 2026-05-21
branch: feat/E012-T09-analyst-layout-flip
title: Analyst layout flip — date header on top, day-nav on bottom
---

# E012-T09: Analyst layout flip — date header on top, day-nav on bottom

## Context

User feedback after running E012-T07 visually: the day-navigator at the
top of the Analyst window feels secondary to the timeline strip, and the
classical "tab bar above content" pattern doesn't match how the dashboard
is read (timeline + KPIs are the focus, day picking is the navigation
chrome). They want:

- **Top of the window**: centred date title (`Tuesday · 17 May 2026`)
  flanked by `◀` / `▶` arrows. This is the "what am I looking at" line.
- **Middle**: TimelineDetail strip + KPI panels (same content as today).
- **Bottom**: DateNavigator (range + day tabs with mini bars) acting as
  a sticky / persistent bottom nav strip.

Donut KPIs and pulse-on-day-change are tracked separately in T10 / T11
so the layout change can ship first.

## Out of scope

- New chart components — donuts come in T10.
- Animation pulse on day change — that's T11.
- Any change to data plumbing (`useTimelineNav`, hooks, fixtures).

## Scope (concrete)

- Refactor `apps/desk/src/analyst/ExplorerTab.tsx`:
  - Add a new `AnalystHeader` component (`apps/desk/src/analyst/AnalystHeader.tsx`)
    showing `◀ {weekday · day month year} ▶`. Reuse `useTimelineNav`
    `prev` / `next` callbacks.
  - `TimelineDetail` loses its internal `TimelineDetailHeader`
    (date + nav buttons move to the new top-level header).
  - Move `DateNavigator` to the bottom of the page; add a
    `placement: "top" | "bottom"` prop or a separate `BottomDayNav`
    wrapper if styling differs significantly.
- Sticky behaviour: bottom nav stays in view on scroll
  (`position: sticky; bottom: 0`). Add subtle top border.

## Decisions

- **Header is a separate component, not a prop on `TimelineDetail`.**
  Reason: header now drives nav for the whole page (not just the
  timeline). Keeping it separate avoids muddling responsibilities.
- **DateNavigator stays one component with a `placement` prop**, not
  a fork. Reason: 95% of the JSX is identical; the only difference is
  outer container styling + maybe weekday label density.
- **Bottom nav is sticky, not fixed.** Reason: fixed positioning
  competes with the existing window chrome; sticky scrolls with the
  page until it pins at the bottom edge.

No ADR — this is a layout rework, not an architectural choice between
viable options. If the placement prop grows into a forked component, an
ADR may be added during execution.

## Acceptance criteria

- [ ] Top of ExplorerTab shows `◀ Tuesday · 17 May 2026 ▶` centred.
- [ ] Clicking arrows changes `selectedDay`; TimelineDetail scrolls.
- [ ] DateNavigator is at the bottom and sticks to the viewport edge.
- [ ] No regression in keyboard navigation (← / →).
- [ ] All existing E012-T07 tests still pass.
- [ ] New tests for `AnalystHeader` (renders title + fires nav).
- [ ] All new files ≤ 250 LoC.

## Cross-references

- E012-T07 implementation: `tasks/E012-T07-timeline-strip.md`
- T10 (donut KPIs): `tasks/E012-T10-recharts-donut-kpis.md`
- T11 (selectedDay pulse): `tasks/E012-T11-selected-day-pulse.md`
- Library research spike: `apps/desk/.plan/reports/2026-05-21-time-viz-libraries-spike.md`
- Timeline visual-upgrades spike: `apps/desk/.plan/reports/2026-05-21-timeline-visual-upgrades-spike.md`
