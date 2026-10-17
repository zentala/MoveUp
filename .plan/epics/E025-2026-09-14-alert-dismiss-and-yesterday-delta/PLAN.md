---
formatVersion: 1
type: epic
status: todo
readiness: ready
points: 17
agent: ts-dev
wave: 6
parallel: []
depends-on: []
blocked-by: ""
---

# E025 — Alert dismiss without sensor + yesterday comparison

## Accepted freshness amendments — 2026-10-17

Operator accepted this refreshed plan for delivery after E023, before E024, and only after overlapping E027/E014 branches are accepted and merged or explicitly parked with all source preserved. No current task is done. This serves the vision of configurable motivation and truthful comparable daily totals. Original estimate was 10p; actual author analysis disclosed T01 3→5p and T03 3→8p, making the current total 17p (5+3+8+1). PO accepts this technical estimate correction within the same product scope; direct Herdr routing and task history are unchanged.

Freshness findings: Metric::compute at metrics/mod.rs:57–62 receives only SessionState and ErgonomicProfile, not yesterday data. T03 must provide an explicit optional yesterday context through MetricEngine and all four production call sites: commands.rs:187, commands_share.rs:32, remote_display_state.rs:60, serial_periodic_reset.rs:55. Preserve existing metrics and distinguish missing/error from an observed zero; derive context in adapters from existing cached/query data, not by reading DB or clock in the pure metric. Any required additional producer/type path must be reconciled before writing.

T03 builds on then-current main after the branch gate; it does not import the unmerged fix/analyst-ui-20261009 KpiBadge rewrite. That branch remains preserved/parked, not silently abandoned. remote_display_state.rs:95–96 currently hard-codes yesterday totals to zero; tests must use nonzero, equal, missing and error contexts and prove desktop/LAN/relay parity, not reuse that fixture unchanged.

T01 also owns relay_commands.rs ack_alert comment/path and relay_commands_tests.rs acknowledgement regression: removing take_user_dismissed must preserve exactly one snooze step per relay acknowledgement, with or without sensor traffic.

### Producer scope accepted — 2026-10-17

Source inspection confirmed get_yesterday_totals currently returns a numeric tuple, TodaySummary exposes numeric yesterday fields, and refresh_today_cache retains old context after query failure. T03 therefore also includes the fifteen exact paths listed in its amended task: query/type owners, cache and logger adapters, their existing consumers/tests and generated wire. Existing query becomes Result<Option<(i64, i64)>, String>: no recognized observations is None; observed zero is Some((0, 0)). Rust TodaySummary yesterday fields become Option<i64>. On refresh failure, warn and invalidate yesterday context without erasing the remaining cached data. Pure metrics still read neither database nor clock. Generate the TypeScript wire using the existing exporter. No schema or session-engine change, live database, new transport or framework. Nullable fields describe observation availability, not a new stored-data format. The actual runtime alert test remains required and NOT_CHECKED until a safe existing native route is established; logic-only tests cannot close it. E023 v0.7.0 is delivered; take the next free minor for E025.

Version: 0.7.0, or next free after E023/E024; one minor bump per epic in actual merge order (planned E023 → E025 → E024). Direct Herdr delivery and existing checks replace obsolete orchestration recipes; no elapsed work caps, hashes or new admission machinery. Push verified delivered commits.

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
back as a KPI in the existing strip. 4 tasks, 17 points after the disclosed 2026-10-17 scope analysis, direct Herdr.

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
`0`. Regression tests must demonstrate the repaired behaviors. The blanket historical requirement to run every new test against reverted source is superseded by the proportional 2026-10-18 disposition below.

### Proportional regression evidence — 2026-10-18

Independent source review found no confirmed defect. Three source-compatible old bodies can meaningfully demonstrate failure: MetricEngine::with_defaults registration, EMPTY_TODAY_SUMMARY, and KPI_TOOLTIPS. The same author may temporarily restore only these exact old bodies, run their relevant behavioral assertions, then restore the accepted implementation and confirm GREEN/clean source. Serialize this with browser QA: no source mutation while the preview is being judged. Preserve failed exits and distinguish assertion failures from compilation/loader failures.

T01's historical polling/field API is incompatible with the new callback/listener API; rebuilding an obsolete multi-file implementation solely to make every new test run is unnecessary. Retain the observed listener-disabled mutation result explicitly as mutation evidence, alongside actual callback/listener regressions and independent source inspection; do not relabel it exact-old-source RED. Existing meaningful negative controls and review replace the blanket per-test reconstruction requirement. This removes an unnecessary procedure under the operator's lean-delivery direction; it does not waive real no-sensor behavior, cooldown, native T02, browser checks or independent final acceptance. No new framework, counter or admission machinery.

## Constraints

- UI change → `.claude/rules/ux-design-flow.md`: `vs-yesterday` scenario in
  `src/test/scenarios.ts`, shown on `/#/mockup` before merging T03.
- Files ≤ 250 lines, functions ≤ 50.
