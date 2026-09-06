# MoveUp — epic history

## TLDR

Condensed narrative of finished epics: what each one changed and why, with
links to its `PLAN.md`, its `JOURNAL.md`, and its section in
[`DONE.md`](./DONE.md). The task-by-task record stays in `DONE.md`; this file
is the story, not the list. One entry per closed epic, newest last.

Epics E001–E010 predate this file and are recorded only in
[`DONE.md`](./DONE.md); they are not backfilled here.

---

## E011 — Autostart Hardening (2026-05-07)

Plan: [PLAN.md](./epics/E011-2026-05-07-autostart-hardening/PLAN.md) ·
Journal: [JOURNAL.md](./epics/E011-2026-05-07-autostart-hardening/JOURNAL.md) ·
Tasks: [DONE.md § E011](./DONE.md#e011--autostart-hardening-2026-05-07)

The app registered itself for autostart once and then trusted the registry
forever. Any rebuild that moved `desk.exe` left a stale
`HKCU\...\Run` value pointing at a path that no longer existed, so the app
simply stopped launching at login and said nothing about it — the classic
"silence read as success" failure. The epic made autostart verify itself.

What changed:

- **Autostart self-heal** — on start the app reads the registry value, compares
  it against its own executable path, and re-registers when they disagree.
  Dev builds skip registration entirely (`#[cfg(debug_assertions)]`), so a
  `pnpm tauri:dev` run can no longer hijack the autostart entry. Every branch
  writes an `AUTOSTART …` line to `events.log`, so the decision is auditable
  after the fact instead of being invisible.
- **EventLogger write reliability** — the log the point above depends on was
  itself dropping writes; the investigation and fix landed in the same epic.
- **`--minimized` autostart launch** — a login-time start no longer throws the
  popup in the user's face; the single-instance `show()` path still works when
  the user launches the app by hand.
- **Precommit `tsc --noEmit` gate** on `apps/desk`, plus the pending
  `OneBarTimeline.test.tsx` fix that gate would otherwise have blocked.

Close-out note: the epic's code was complete on 2026-05-07, but its ceremony
(history entry, IMPRO triage, checklist) stayed open until E016-T03 on
2026-09-06. The planned `0.4.0` bump and `v0.4.0` tag never happened and are
recorded as moot in the epic's
[ORCHESTRATOR.md](./epics/E011-2026-05-07-autostart-hardening/ORCHESTRATOR.md) —
E012 had already carried the version to `0.5.0` by then.

## E012 — Analyst Dashboard (2026-05-16, v0.5.0)

Plan: [PLAN.md](./epics/E012-2026-05-16-analyst-dashboard/PLAN.md) ·
Journal: [JOURNAL.md](./epics/E012-2026-05-16-analyst-dashboard/JOURNAL.md) ·
Tasks: [DONE.md § E012](./DONE.md#e012--analyst-dashboard-2026-05-16-v050)

The app had been producing data for months — minute snapshots, an events log,
SQLite sessions, profiles, the store — with no way to look at any of it beyond
the live Debug tab. Answering "why did that notification fire on Tuesday"
meant reading JSON by hand. E012 gave the data a window.

What changed:

- **Range-query backend** — `get_snapshots_range`, `get_events_range` and
  `get_sessions_range`, with an `events.log` parser and date-range bounds
  (`commands_analyst.rs` plus its sibling test file).
- **Data catalog** — `get_data_catalog` describes all 8 sources the app writes
  (sensor, sqlite_sessions, snapshots, events_log, `profiles_*`, store,
  remote_ws), so the dashboard documents the app's own data surface rather
  than hardcoding a list.
- **Analyst window** — a separate 1280×800 Tauri window opened from the tray,
  not a route in the popup ([ADR 012](../.arch/ADR/012-analyst-dashboard-separate-window.md)).
  Two tabs: Catalog (sortable source table) and Explorer (DateNavigator plus
  five SVG charts — DeskHeightTimeline, StateGantt, DailyScoreTrajectory,
  BreakCreditHistogram, KpiTrend).
- **Mockup-first flow** — `/#/mockup/analyst` with fixtures shipped before the
  live wiring, per the repo's UX rule that mockups come before code.

Shipped as `v0.5.0`. Follow-on decision: [ADR 013](../.arch/ADR/013-date-navigator-merge.md)
(DateNavigator merge).

## E016 — Plan hygiene and AO readiness (2026-09-06)

Plan: [PLAN.md](./epics/E016-2026-09-06-plan-hygiene-and-ao-readiness/PLAN.md) ·
Handoff: [HANDOFF.md](./epics/E016-2026-09-06-plan-hygiene-and-ao-readiness/HANDOFF.md)

Documentation and repo config only, no application code. The 2026-09-06
architecture review found the plan tree had drifted: `STATE.md`'s body
disagreed with its own frontmatter, three stale planning files sat in the repo
root competing with `.plan/`, `HISTORY.md` did not exist despite twelve closed
epics, E011's ceremony was still open, and the repo lacked the `.giter.yaml`
and `justfile` the Agent Orchestrator needs to dispatch work here. E016 closed
all of it, each task verified by its own `scripts/check-e016-t0N.mjs`.

This file is one of its outputs.
