---
formatVersion: 1
type: handoff
status: todo
---

# E025 Handoff — alert dismiss without sensor + yesterday comparison

## TLDR

Read only this file and [`PLAN.md`](PLAN.md). 10 points → subagents, each in
its own worktree (`wt-add`). Wave 1: T01 + T03 in parallel (disjoint files).
Wave 2: T02 (needs T01). Wave 3: T04. Bump the minor epic version before T01
(next free `0.X.0` after E023/E024, `.claude/rules/versioning.md`).

## Mental model

- Popup: raw WinAPI window on its own thread (`alert_popup_window.rs:29`).
  On close it sets `user_dismissed` (`:114`). `AlertPopup` wraps it
  (`alert_popup.rs:38-90`). Today the only reader is `update_from_policy`
  (`tray_controller.rs:106-110`), registered on `DESK_DISTANCE` only
  (`:36-39`) — no sensor, no reader.
- Fix shape (D1): give `run_popup_window` an `AppHandle` (or a callback) and
  `emit("alert:user-dismissed")` where the flag is set today; listen in
  `tray_controller::setup` and call `comm_policy.dismiss()`. Delete
  `take_user_dismissed` and its call once the event path is tested.
- KPI metrics: `trait` + engine in `src-tauri/src/metrics/mod.rs`; existing
  metrics `standing_pct.rs`, `position_rate.rs`, `hourly_breaks.rs`,
  `longest_session.rs` are the pattern. Yesterday totals:
  `db_sessions.rs:224` `get_yesterday_totals`. Tooltips: `KPI_TOOLTIPS` used at
  `src/widgets/one-bar/KpiStrip.tsx:44`.
- Do not touch: `session_*.rs` (ADR 015), `serial*.rs` (E024), `relay/**` (E023).

## Tasks

- [ ] **T01** (3, ts-dev, Rust) — `alert:user-dismissed` event path (D1),
  remove the polling path. Tests `e025_dismiss_*` in
  `tray_controller_tests.rs`: dismiss without sensor reaches policy; dismiss
  with sensor still works. Verify:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib -- e025_`.
  [task](tasks/T01-dismiss-as-event.md)
- [ ] **T02** (3, ts-dev) — `tests/integration/alert_flow.test.ts` (closes
  E004-T04): limit → popup signal → dismiss → cooldown → re-show, time via
  `inject_reading`. Needs a debug-only IPC to trigger dismiss if none exists
  (gate like `commands.rs:217`). Verify: running debug app (PM3,
  `--owner worktree:<path>`) + `pnpm test:integration`, test count > 0.
  [task](tasks/T02-alert-flow-integration-test.md)
- [ ] **T03** (3, ts-dev) — `metrics/vs_yesterday.rs` (D2), registered in the
  engine, tooltip text, `vs-yesterday` mockup scenario, snapshot carries it.
  Tests: `metrics/tests.rs` (5 cases from PLAN), `KpiStrip.test.tsx`,
  `remote_display_state` test. Verify:
  `cargo test --manifest-path src-tauri/Cargo.toml --lib -- e025_` and
  `npx vitest run --config vite.config.ts src/widgets/one-bar`.
  [task](tasks/T03-vs-yesterday-metric.md)
- [ ] **T04** (1, verify + main) — `just check`; per-criterion verdict;
  `.arch/UX-FLOW.md` alert + KPI sections; D1-D3 into `decisions.jsonl`;
  re-tick E001-T12 and E004-T05 in `DONE.md`/`ORCHESTRATOR.md`; BACKLOG entry
  for the unused `get_today_summary` poll (D3); INDEX row `done`.
  [task](tasks/T04-verify-and-close.md)

## Waves

Fala 1: T01 + T03 — T01 touches `alert_popup*.rs`, `tray_controller*.rs`;
T03 touches `metrics/**`, `remote_display_state.rs`, `src/widgets/one-bar/**`,
`src/test/scenarios.ts`. Fala 2: T02 (tests the T01 path). Fala 3: T04.

## Done means

All five acceptance criteria hold, every new test seen failing on reverted
code, INDEX row `done`, `.plan/HISTORY.md` entry written.
