# E012 — Orchestrator

## Wave 1 — Backend queries + mockup (parallel, 3 worktrees)

- [x] **E012-T01** Backend: snapshot/event range query commands
  Branch: `feat/E012-T01-range-queries`
- [x] **E012-T02** Backend: data catalog metadata command
  Branch: `feat/E012-T02-catalog-meta`
- [x] **E012-T03** Frontend mockup `/mockup/analyst` with fake data
  Branch: `feat/E012-T03-mockup`

**Gate before Wave 2:**
- zentala opens `http://localhost:1443/#/mockup/analyst`, reviews both tabs, approves visually
- `pnpm test:unit` and `cargo test` pass for T01/T02 commands

## Wave 2 — Real window + Catalog wiring (parallel, 2 worktrees)

- [x] **E012-T04** Analyst window registration (Tauri config + tray menu item)
  Branch: `feat/E012-T04-window`
  Depends on: T01, T02 (no UI lock yet — wires commands)
- [x] **E012-T05** Catalog tab live wiring (replaces fake data with `get_data_catalog()`)
  Branch: `feat/E012-T05-catalog-live`
  Depends on: T02, T03 (mockup approved)

## Wave 3 — Explorer + release polish (1 worktree)

- [x] **E012-T06** Explorer tab live + tests + UX-FLOW + PROJECT.xml + version 0.5.0
  Branch: `feat/E012-T06-explorer-release`
  Depends on: T01, T03, T04, T05

## Merge order (after each wave's tasks complete)

1. T02 (catalog meta — pure read, smallest blast radius) → main
2. T01 (range queries) → main
3. T03 (mockup — frontend-only) → main
4. T04 (window registration — Rust config + tray) → main
5. T05 (catalog tab live) → main
6. T06 (explorer + version bump) → main, then `git tag -a v0.5.0`

## Definition of done for the epic

- All 6 task checkboxes flipped to `[x]`
- `apps/desk/.plan/DONE.md` (if exists, else root `.plan/DONE.md`) has entry per task with commit SHAs
- `apps/desk/.arch/UX-FLOW.md` describes the Analyst window
- `apps/desk/PROJECT.xml` lists new commands + window
- `apps/desk/CHANGELOG.md` (if exists) entry for v0.5.0
- IMPROVEMENTS.md triaged with zentala (deferred items → `.plan/IMPROVEMENTS.md` or dropped)
- `.arch/HISTORY.md` (if exists in apps/desk/.arch/) gets E012 summary line

## Subagent prompt template

```
Work from: <absolute worktree path>
Read: apps/desk/.plan/epics/E012-2026-05-16-analyst-dashboard/PLAN.md
      apps/desk/.plan/epics/E012-2026-05-16-analyst-dashboard/tasks/<task>.md
      apps/desk/.plan/epics/E012-2026-05-16-analyst-dashboard/reports/2026-05-16-data-sources.md
Run: pnpm test:unit, cargo test (per the task file)
Commit: feat(E012-TXX): <short>
Constraint: file ≤ 250 lines, function ≤ 50 lines, no UTF icons in UI.
```
