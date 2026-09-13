---
formatVersion: 1
type: epic
status: todo
readiness: ready
points: 10
agent: ts-dev
wave: 6
parallel: [E023, E024]
depends-on: []
blocked-by: ""
---

# E025 — Alert dismiss without sensor + yesterday comparison

Source: the E001-E009 status audit of 2026-09-13 found two items marked done
that are not in the running app. Moved here from `.plan/BACKLOG.md`
§"E001-E009 audit leftovers" and §"From E004" on 2026-09-14. Handoff:
[`HANDOFF.md`](HANDOFF.md). Board deck (Polish): [`PRES.md`](PRES.md).

## TLDR

Two regressions hidden by stale `DONE.md` ticks. (1) Clicking Dismiss on the
sit-limit popup does nothing while the sensor is disconnected, so the popup
comes back the moment the sensor returns (E004-T05, never actually fixed).
(2) The app computes yesterday's sitting and standing totals every 10 seconds
and shows them nowhere — the March `TodayStats` arrow was unmounted by the
widget rewrite and later deleted as dead code (E001-T12). This epic fixes the
dismiss path with an event instead of a sensor-driven poll, adds the missing
alert-flow integration test (E004-T04), and brings the yesterday comparison
back as a KPI in the existing strip. 4 tasks, 10 points, subagents.

## Problem

| # | What | Evidence |
|---|---|---|
| P1 | Dismiss is consumed only inside `update_from_policy`, which runs only on `desk:distance` | `src-tauri/src/tray_controller.rs:36-39`, `:106-110`; popup sets the flag at `alert_popup_window.rs:114`; `take_user_dismissed` at `alert_popup.rs:89` |
| P2 | No test covers popup → dismiss → snooze → no re-show | `tests/integration/alert_flow.test.ts` never created; `session-flow.test.ts:79-101` stops at "alert fires" |
| P3 | Yesterday totals are computed, polled and stored, never rendered | `db_queries.rs:52-59` → `useDesk.ts:103-104` (every 10 s) → `deskReducer.ts:152`; no component reads `todaySummary`. KPI strip renders only `dashboard.metrics` (`OneBarWidget.tsx:69`, `KpiStrip.tsx:60`) |
| P4 | History of P3 | `TodayStats.tsx` had the ↑/↓ delta (`git show 663ce98^:src/components/TodayStats.tsx:70-90`), unmounted in `e49a1f6`, deleted in `663ce98` (E018) |

## Decisions and ADRs

- [ADR 015](../../../.arch/ADR/015-pure-ergo-engine.md): engine stays pure; the
  dismiss fix lives in the tray/popup layer, not in `session_*.rs`.
- [ADR 010](../../../.arch/ADR/010-notification-escalating-silence.md): dismiss
  feeds the existing escalating-silence snooze; no new snooze logic.
- New decisions (record in `.plan/decisions.jsonl` in T04):
  - **D1** The popup window emits a Tauri event `alert:user-dismissed` the
    moment the user closes it; `tray_controller::setup` listens and calls
    `CommunicationPolicy::dismiss`. `take_user_dismissed` polling is removed,
    not kept as a second path.
  - **D2** The yesterday comparison is a **metric** (`metrics/vs_yesterday.rs`,
    id `sitting_vs_yesterday`) computed in Rust, not a React-side calculation
    over `todaySummary` — one source for desktop, LAN phone and relay, and it
    inherits the KPI tooltip + colour level. Display `−25 min` / `+10 min`;
    level green when today ≤ yesterday, yellow up to +30 min, red above; no
    badge when yesterday has no data (not `0`).
  - **D3** The now-unused `get_today_summary` 10-second poll is **not** removed
    in this epic (other consumers may appear); filed to BACKLOG instead.

Architecture impact: one new metric module, one new Tauri event; no new
component or integration. `.arch/UX-FLOW.md` KPI section and alert section
updated in T04.

## Approaches considered

| | A — minimum: poll `take_user_dismissed` from a 1 s timer; render delta in React from `todaySummary` | B — target: dismiss as an event; delta as a Rust metric | C — restore `TodayStats.tsx` from git |
|---|---|---|---|
| Effort / Risk | S / M | S-M / L | S / H |
| Plus | Smallest diff | No polling; one KPI source for all three viewers; tooltip and colours for free | Code already written |
| Minus | Keeps a polling path; delta logic in the UI means the phone needs its own copy | Metric needs yesterday totals inside the metric engine | Component predates the widget architecture and the design system; mounting it brings back the layout E005 removed |
| Reuses | `alert_popup.rs` | `MetricEngine`, `KpiStrip`, `KPI_TOOLTIPS`, `get_yesterday_totals` | old file |

**Recommendation: B.** A and B cost almost the same; B removes the class of
bug (a UI signal depending on sensor traffic) instead of adding a second
timer, and keeps KPI logic in one place. C reintroduces a layout the app
deliberately left.

## Acceptance criteria

1. With no sensor connected, a popup dismiss reaches `CommunicationPolicy`
   within 1 s and the popup does not re-appear when readings resume until the
   snooze cooldown passes.
2. `tests/integration/alert_flow.test.ts` exists and covers: limit reached →
   popup signal → dismiss → snooze active → no re-show inside cooldown →
   re-show after cooldown (time injected, no real waiting).
3. The KPI strip shows a `vs yesterday` badge with signed minutes and the
   level rule from D2; with no yesterday data the badge is absent.
4. The same metric appears in the LAN/relay snapshot (`remote_display_state.rs`).
5. `just check` green; `DONE.md` entries for E001-T12 and E004-T05 ticked again
   only after 1-4 pass.

## Test strategy

| Crit. | Kind | Assertion that fails today | File |
|---|---|---|---|
| 1 | Rust unit | emitting `alert:user-dismissed` with `sensor_connected=false` sets policy dismissed (today: flag never read) | `src-tauri/src/tray_controller_tests.rs` (`e025_`) |
| 2 | Integration (running debug app, `inject_reading`) | dismiss then readings → no popup signal inside cooldown | `tests/integration/alert_flow.test.ts` |
| 3 | Rust unit | today 60 min, yesterday 85 → `−25 min`, green; +20 → yellow; +40 → red; yesterday 0 rows → `None` (nil); yesterday exactly equal → `0 min` green (empty/edge) | `src-tauri/src/metrics/tests.rs` |
| 3 | TS unit | `KpiStrip` renders the badge with tooltip text from `KPI_TOOLTIPS.sitting_vs_yesterday` | `src/widgets/one-bar/KpiStrip.test.tsx` |
| 4 | Rust unit | `remote_display_state::build` snapshot contains `sitting_vs_yesterday` | `src-tauri/src/remote_display_state.rs` tests |

Error path: DB read of yesterday fails → metric absent and one `warn!`, never
`0`. Each new test is run once against reverted code and must fail.

## Constraints

- UI change → `.claude/rules/ux-design-flow.md`: `vs-yesterday` scenario in
  `src/test/scenarios.ts`, shown on `/#/mockup` before merging T03.
- Files ≤ 250 lines, functions ≤ 50.
