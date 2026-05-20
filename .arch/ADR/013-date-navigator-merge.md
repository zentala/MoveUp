# ADR 013: Merge range picker + day list into a single DateNavigator

- **Status**: accepted
- **Date**: 2026-05-18
- **Epic**: E012 (Analyst Dashboard) — task T07 (Daily Timeline)
- **Supersedes**: partial — the standalone `DateRangePicker` is no longer
  rendered by the Explorer tab. The file remains for the `DateRange` type
  it exports and for potential reuse by other views.

## Context

E012 shipped the Explorer tab with `DateRangePicker` (two date inputs)
above a grid of charts including `StateGantt` (multi-day rows). In E012-T07
we considered three options for the new Daily Timeline feature:

1. Add a separate **Context Strip** beside the existing `DateRangePicker`,
   keeping the range picker as the primary nav, with a brushable mini-strip
   to pick the focused day.
2. Replace `StateGantt` with a single hero strip that hosts its own date
   nav (`◀ ▶ + date pill`) and ignore the range picker.
3. **Merge** the range picker and the per-day overview into one component
   (`DateNavigator`) — header has range pills + presets, body is a row of
   day tabs that double as a grouped mini bar chart per day.

The brainstorm pass against the original "Context Strip + Hero Strip"
design (option 1) surfaced several concerns:

- C1 — time-axis distortion if side previews kept "6h compressed to 12%".
- C4 — range picker vs strip conflict: two independent date selectors
  competing for primacy in the same UI region.
- C6 — losing the multi-day pattern view from `StateGantt`.

Option 3 collapses C4 by definition (one component owns the date model),
and the per-day mini bars in the navigator preserve C6 (multi-day pattern
visibility) without committing to the heavy `StateGantt` rendering.

## Decision

Adopt **option 3**: a single `DateNavigator` component that combines the
range header (date inputs + 7d/14d/30d preset chips) with N day tabs
showing grouped mini bar charts. Below it, `TimelineDetail` renders one
continuous proportional strip for the entire range; clicking a day tab
smooth-scrolls the strip to that day's centre.

`StateGantt` is removed from the Explorer layout (and the source files
deleted) — its multi-day pattern role is taken over by the day-tab strip
of mini bars in `DateNavigator`.

## Alternatives considered

- **Option 1** (context strip + range picker) — rejected because of the
  range-vs-brush conflict (C4) and the additional vertical real estate
  cost.
- **Option 2** (hero strip with own nav, range picker scoped only to
  other charts) — rejected because the range model would be inconsistent
  across the dashboard (timeline-strip range ≠ other-charts range = user
  confusion).

## Consequences

- DateNavigator becomes the single date authority for the Explorer tab.
  All charts and the timeline read `range` from the same prop.
- `DateRangePicker` is no longer rendered in the Explorer tab. The file
  stays in tree because:
  - `DateRange` type is imported by `ExplorerTab` and other consumers.
  - The standalone picker may be reused in a future "compact view" or
    `AnalystWindow` header.
- `StateGantt` is gone — its tests and source are deleted. Anyone who
  imports it from external code will break (none exist today).
- File-size cap kept by splitting `DateNavigator` into three sibling
  files: `DateNavigator.tsx` (orchestration), `DateNavigatorHeader.tsx`
  (range pills + presets), `DateNavigatorColumn.tsx` (single day column).
- Future work: a "compact mode" (fewer days, no mini bars) could become a
  prop on `DateNavigator` instead of a different component.
