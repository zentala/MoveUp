# E012 — Analyst Dashboard (Data Catalog + Explorer)

**Status:** planned
**Started:** 2026-05-16
**Linked from:** [.plan/BACKLOG.md](../../BACKLOG.md), [.plan/STATE.md](../../STATE.md)

## What

Add a dedicated **Analyst Window** to the desk app — a separate Tauri window (not replacing the popup) that surfaces every data source the app produces, with two tabs:

1. **Data Catalog** — for each source (sensor, SQLite, snapshots, events.log, profiles, store, system signals): schema, sample rows, cardinality, retention, on-disk path. The reference for "what data do I actually have?"
2. **Explorer** — date-range scoped (MVP = last 7 days) visualizations on top of that data: desk height timeline, state Gantt, idle vs state correlation, daily score trajectory, break credit histogram, KPI trends.

Backed by the research in [`reports/2026-05-16-data-sources.md`](./reports/2026-05-16-data-sources.md) (§6 catalogues 10 candidate views — Explorer MVP picks ~5).

## Why

Today the app logs ~20 fields per minute snapshot plus event lines plus session rows, but only a fraction surfaces in the OneBar popup. zentala (the developer-user, per `apps/desk/CLAUDE.md`) treats this app as an "experimentation platform for self-motivation" — to redesign scoring rules, escalation policies, or break-credit logic, he needs to *see* what the existing data actually says. Without a dashboard the only options are: open snapshot JSONs by hand, grep events.log, or query SQLite manually. All three are friction that suppresses experimentation.

A live analyst window also makes ergonomic-policy debates concrete: instead of "I think the break credit is too lenient," he can show the histogram of partial-vs-full credits over the past 7 days.

## Scope

**MVP — 6 tasks across 3 waves.** Worktree per task, parallel dispatch within a wave.

| Task | Wave | Title |
|------|------|-------|
| E012-T01 | 1 | Backend: snapshot/event range query commands |
| E012-T02 | 1 | Backend: data catalog metadata command |
| E012-T03 | 1 | Frontend: mockup `/mockup/analyst` route (Catalog + Explorer with fake data) |
| E012-T04 | 2 | Frontend: Analyst window registration + real route wiring |
| E012-T05 | 2 | Frontend: Catalog tab (live data) |
| E012-T06 | 3 | Frontend: Explorer tab (5 charts, live data) + tests + UX-FLOW + PROJECT.xml + version bump 0.5.0 |

**Wave 1 → Wave 2 gate:** user approves T03 mockup before T05/T06 code lands (per `.claude/rules/ux-design-flow.md`).

## Acceptance criteria

- [ ] Tray menu has "Open Analyst" item — clicking opens a second Tauri window (not the popup overlay)
- [ ] Window has two tabs: Catalog | Explorer
- [ ] Catalog lists ≥ 6 sources with schema, sample rows, on-disk path, retention
- [ ] Explorer date picker defaults to "last 7 days," shows ≥ 5 visualizations driven by real data
- [ ] All new Tauri commands have unit + integration tests; coverage stays ≥ 80%
- [ ] Mockup approved by user before live wiring lands
- [ ] `UX-FLOW.md`, `PROJECT.xml`, `package.json` / `tauri.conf.json` / `Cargo.toml` bumped to `0.5.0`
- [ ] No regression in existing 590+ TS / 420+ Rust tests

## Test strategy

**Unit (Rust):**
- Snapshot file walker — date range filter, missing days, malformed JSON tolerance
- events.log parser — line tokenizer for each event type, malformed-line tolerance
- Catalog builder — counts match what's on disk (mock tempdir fixtures)

**Unit (TS):**
- Each chart component renders with fixture data
- Date range picker emits expected `{ from, to }` ISO dates
- Catalog table renders schema rows correctly

**Integration:**
- Tauri commands round-trip real snapshot/events fixtures placed in a tempdir; assert returned shape

**E2E (manual smoke in T06):**
- Open Analyst window from tray
- Switch tabs, change date range — chart re-renders within 500ms for 7d window

## Out of scope (deferred)

- CSV/JSON export → BACKLOG
- Date range > 7 days (would need SQLite-only history mode) → BACKLOG
- Anomaly detection / ML on snapshots → BACKLOG
- Live-tail of today (auto-refresh while open) → BACKLOG (MVP refreshes on tab focus)
- Profile-switch causality view → BACKLOG (depends on profile-switch event logging which doesn't exist yet)

## Files referenced

- [ORCHESTRATOR.md](./ORCHESTRATOR.md) — execution order + waves
- [JOURNAL.md](./JOURNAL.md) — live findings + session summaries
- [IMPROVEMENTS.md](./IMPROVEMENTS.md) — open TODOs surfaced during this epic
- [reports/2026-05-16-data-sources.md](./reports/2026-05-16-data-sources.md) — full research backing the plan
- Task specs: [tasks/](./tasks/)
- UX flow updates: `.arch/UX-FLOW.md` (T06)
- Versioning rule: `.claude/rules/versioning.md`
- UX design flow (mockup-first rule): `.claude/rules/ux-design-flow.md`
