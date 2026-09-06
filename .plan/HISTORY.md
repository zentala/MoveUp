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

## E015 — One truth for the sitting counter (2026-09-06, v0.6.0) — code-complete, T05 open

Plan: [PLAN.md](./epics/E015-2026-09-06-engine-single-truth/PLAN.md) ·
Handoff: [HANDOFF.md](./epics/E015-2026-09-06-engine-single-truth/HANDOFF.md)

The popup timer read `current_session_secs`, which hard-reset to 0 on every
return to sitting — the credited counter the engine actually maintained
(`sitting_seconds`) was never surfaced. E015 deleted the wrong field
entirely, renamed the Debug-only field honestly to `secs_since_last_break`,
fixed PostureBalance to compare two raw counters instead of a credited one
against a raw one, added a `break_credit` column so the Analyst reads what
the engine decided instead of guessing from duration, and raised the
`standard` profile's break-credit multiplier to 3.0. Ran through the Agent
Orchestrator (3rd attempt — two earlier ones caught real `write_set` gaps
and a `dirty_worktree` caused by this machine's `core.autocrlf`, see the
epic's `JOURNAL.md`), promoted at `7a8bbf3`, version bumped to 0.6.0 and
tagged. 504 Rust + 255 TS tests pass, typecheck clean.

**Not fully closed**: T05's browser/visual pass on the popup could not run —
two pre-existing dev-mode gaps (no Vite proxy for the remote-display route,
and mock mode never drives the real session engine) blocked it, filed to
`.plan/BACKLOG.md`. The fixed scenario (sit 30 min, stand 2 min, sit) is
proven by an automated Rust test, not by a screenshot of the running app.

## E017 — Release readiness (2026-09-06, v0.6.0 tagged)

Plan: [PLAN.md](./epics/E017-2026-09-06-release-readiness/PLAN.md) ·
Handoff: [HANDOFF.md](./epics/E017-2026-09-06-release-readiness/HANDOFF.md) ·
Journal: [JOURNAL.md](./epics/E017-2026-09-06-release-readiness/JOURNAL.md)

Every user-facing document still described a retired product — "zntlDesk",
old data paths, an old repo, an auto-updater that was never built — and the
app shipped with no LICENSE despite an open-core business model. E017
rewrote the five user docs, added MIT licensing, fixed a CI workflow still
pointed at a monorepo layout (`apps/desk/`) that no longer exists, wrote
firmware flashing instructions, and refreshed the stale perf baseline. Ran
through the Agent Orchestrator (one stale/transient `merge_conflict` on
T01 — a dry-run merge proved the branch clean, `ao resume` completed it),
promoted at `91b3572`, all 8 tasks independently re-verified on `main`
afterward (506 Rust + 259 TS tests, typecheck, all 8 per-task check
scripts).

Three of the epic's four "Outside AO" release blockers were closed in the
same session: the full `pnpm tauri:build` gate ran for real (producing
`MoveUp_0.6.0_x64_en-US.msi` and `MoveUp_0.6.0_x64-setup.exe`, ~4-6 MB each —
correcting a stale 60-70 MB target that had never been true for this app),
and a `browser`-agent first-run pass found and — after a second `ts-dev`
pass fixed — three real bugs: mangled Polish diacritics in the welcome
popup, a CWD-relative static-file path in `remote_server.rs` that could 404
depending on launch context, and unguarded Tauri calls throwing in
remote-display mode. The fourth item, choosing a code-signing provider, was
presented to Paweł as a real cost/vendor decision and deliberately deferred
— recorded open in `.plan/BACKLOG.md`, not silently dropped. Calibration and
the no-sensor state could not be visually confirmed even after the fixes:
calibration is deliberately hidden from the remote-display path by CSS
(desktop-only by design), and the no-sensor state needs the physical
sensor unplugged, which a browser agent cannot do.

## E019 — Backend hardening (2026-09-06)

Plan: [PLAN.md](./epics/E019-2026-09-06-backend-hardening/PLAN.md) ·
Handoff: [HANDOFF.md](./epics/E019-2026-09-06-backend-hardening/HANDOFF.md) ·
Journal: [JOURNAL.md](./epics/E019-2026-09-06-backend-hardening/JOURNAL.md)

Fourteen sites in `commands.rs` unwrapped a mutex lock directly, so one
poisoned lock (a panic while holding it) would have crashed every subsequent
command instead of failing gracefully. `get_today_summary` computed "today's
totals" two different ways in two places, IPC event names were duplicated as
string literals at every emit/listen site instead of named constants, and
`EventLogger::new` panicked rather than warned when its base directory was
unusable. E019 fixed all four, plus split two over-length files
(`google_fit.rs`, `serial_periodic.rs`) and deduped remote-display-state
derivation between the tray and the remote server.

Ran through the Agent Orchestrator (`E019-20260906-0822`) with two operator
interventions, neither a code defect: the first wave's two workers both hit
a Claude session-limit wall at the same moment (diagnosed by pulling raw
`stdout` out of the AO ledger — `ao status` does not surface it directly);
after the reset, task T04 then failed verification because Windows Smart App
Control had started blocking freshly-compiled, unsigned Rust proc-macro DLLs
system-wide (confirmed via `Microsoft-Windows-CodeIntegrity/Operational`,
193 events) — not specific to this epic or to AO, a plain `cargo build` in
the main checkout hit the same wall. Fixed live with `CiTool.exe --refresh`
(no reboot needed) after explicit user consent, since disabling a Windows
security feature is treated as irreversible. Promoted at `4649dc5`; all 8
tasks independently re-verified on `main` afterward (533 Rust + 3
integration + 261 TS tests). Both root causes filed to
`dispatch.internal/.plan/BACKLOG.md` as gaps in AO's `executor_result_error`
classification.

## E018 — Frontend consolidation (2026-09-06)

Plan: [PLAN.md](./epics/E018-2026-09-06-frontend-consolidation/PLAN.md) ·
Handoff: [HANDOFF.md](./epics/E018-2026-09-06-frontend-consolidation/HANDOFF.md) ·
Journal: [JOURNAL.md](./epics/E018-2026-09-06-frontend-consolidation/JOURNAL.md)

Two hooks (`useDesk`, `useRemoteDesk`) duplicated one state machine at
0.78% coverage, five pre-OneBar components stayed built and tested despite
being dead, Rust DTOs were hand-copied into TypeScript with no codegen or
drift check, the coverage gate declared in `vite.config.ts` was never
actually invoked by any script, and E012's last three tasks (Analyst layout
flip, Recharts KPI donuts, a pulse highlight) had never landed. E018 closed
all of it: a shared `deskReducer` behind thin Tauri/WS adapters, the five
dead components deleted, ts-rs codegen wired end to end (`.arch/ADR/017`),
ESLint installed and configured, the coverage gate wired into `pnpm build`
for real, and E012's three remaining UI tasks implemented under E018's own
numbering (`.arch/ADR/016` for Recharts).

Ran through the Agent Orchestrator (`E018-20260906-1209`) with three
write_set widenings and one transient flake, none a worker mistake: T03
correctly deleted a dead component's *two* test files but the plan only
declared one; T02's ts-rs codegen legitimately touched 18 files across
Rust and TypeScript that the plan's narrow write_set never anticipated
(confirmed by diffing every violated path before widening); T10 needed its
new dependency's lockfile, a forgotten test sibling, and a genuine
`CLAUDE.md` documentation update. T05's `pnpm build` verification failed
once from build-cache contention with a parallel worker and passed clean on
a bare re-run. Promoted at `7566de7`; all 11 tasks independently
re-verified on `main` (534 Rust + 3 integration + 304 TS tests).

All three "Outside AO" items were closed in the same session: the
`browser` agent confirmed all four UI checkpoints for the new Analyst
layout (date header/nav, sticky day-nav, three donut KPI cards, and a
pulse on day-change, verified via `MutationObserver`) against `pnpm dev`'s
mockup route rather than the live Tauri window, honestly recorded as such
since the live window needs a physical sensor this session did not have;
the coverage-threshold call (decision D4) was surfaced as a dated code
comment plus a `.plan/BACKLOG.md` follow-up rather than decided silently,
since T05 could not reach the original 80/80/75% target within budget.

## E020 — Engine: pure core (2026-09-06)

Plan: [PLAN.md](./epics/E020-2026-09-06-engine-pure-core/PLAN.md) ·
Handoff: [HANDOFF.md](./epics/E020-2026-09-06-engine-pure-core/HANDOFF.md) ·
Journal: [JOURNAL.md](./epics/E020-2026-09-06-engine-pure-core/JOURNAL.md)

The session engine took `Utc::now()` directly instead of an injected clock
(making deterministic tests depend on backdating tricks), `SessionState`
carried six ergonomic-limit fields that belonged to configuration, not
state, persistence was ad hoc rather than one versioned snapshot, and
`tray_controller.rs` re-derived signals (`elapsed_secs`, standing-lap
progress) the engine should have owned outright. E020 closed all four: the
clock is injected end to end with a dedicated midnight/DST test, the limit
fields moved out into `&ErgonomicProfile`, persistence became one
`PersistedEngineState { schema_version, .. }` with a migration from the old
shape, and the engine now computes every policy-facing derived field
itself — `tray_controller.rs::compute_standing_lap` is gone.

Ran through the Agent Orchestrator (`E020-20260906-1418`) mostly
sequentially (every task edits `SessionState`/`SessionManager` or a file
depending on the previous task's shape). One Claude session-limit hit on
the first task, resolved by waiting past the reset. Four write_set
widenings, all legitimate: three narrow ones (a clock-consistency fix in
`session_manager.rs`, a `lib.rs` mod line plus a new limits test file, a
test sibling for the engine-owned signals), and one genuine cross-epic fix
— this epic's refactor made two assertions in the already-closed E019's own
doc-consistency script permanently false, and the worker corrected them
rather than leaving a check that would fail on every future `just check`,
credited in a code comment. Also discovered and fixed a self-inflicted
class of false positive: editing `HANDOFF.md` mid-run to widen a write_set
leaves the frozen `ao/integration` branch's copy stale, which then flags
completely unrelated doc drift as an out-of-scope write on the next task —
fixed by syncing the integration branch's copy directly rather than
generating a fresh run-id each time. Promoted at `244de29`; all 8 tasks
independently re-verified on `main` (569 Rust + 3 integration + 304 TS
tests). No Outside-AO items — pure Rust engine refactor, no UI surface.
