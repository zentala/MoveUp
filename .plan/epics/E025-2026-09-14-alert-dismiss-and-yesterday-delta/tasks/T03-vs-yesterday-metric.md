---
formatVersion: 1
type: task
id: E025-T03
epic: E025
status: todo
agent: ts-dev
points: 8
model: standard
effort: focused
confidence: 4
summary: "Bring back the today-vs-yesterday comparison as a Rust KPI metric shown in the KPI strip and the phone snapshot."
created: 2026-09-14
---
# E025-T03: T03-vs-yesterday-metric

## Acceptance

2026-10-18 test-only scope amendment: `src-tauri/src/commands_tests.rs` is accepted solely to update the existing exact poison-safe lock count and its message from 14 to 15. Actual commands.rs now adds one yesterday-cache lock with unwrap_or_else recovery; no unsafe lock().unwrap site was introduced. Preserve the no-unsafe-lock assertion and real poisoned-state recovery test unchanged. This remains within T03 8p; no production or contract expansion. Retain failed full-check/Cargo101 as observed, then run the relevant regression and required final checks after fresh effective path access.

Estimate updated 2026-10-17 from 3p to 8p after actual producer/consumer analysis; full epic now 17p, same queue and acceptance criteria. This supersedes the historical 3p retention wording below.

Additional authored test accepted: `src/hooks/useDesk.test.ts`, replacing absent-yesterday zeros with null and testing the real desktop hook transport's missing versus observed-zero behavior. This mocked transport regression is not native runtime evidence.

Existing exporter output permission is accepted for exactly these existing files under `src/generated/`: AppConfig.ts, BreakCredit.ts, ClientInfo.ts, Command.ts, CommandResult.ts, DashboardState.ts, DeskRecord.ts, DeskState.ts, DeskStatus.ts, ErrorBody.ts, ErrorKind.ts, HealthSnapshot.ts, HealthView.ts, Hello.ts, MetricLevel.ts, MetricResult.ts, MetricSnapshot.ts, PairingCode.ts, PortInfo.ts, RelayState.ts, RelayStatus.ts, Role.ts, SessionRow.ts, SessionStateDto.ts, StateChangedPayload.ts, TodaySummary.ts, Viewer.ts, VoiceNoteRow.ts, Welcome.ts. Run only the existing Rust exporter after fresh path access preflight. TodaySummary.ts is the intended semantic change; all other files may be rewritten identically by that exporter, not manually changed. Any additional content difference or new output path returns for assessment before acceptance. No new generator or provenance tooling.

Additional producer scope accepted by PO on 2026-10-17 after actual source inspection (not a new product decision):

- `src-tauri/src/db_sessions.rs`
- `src-tauri/src/db_queries.rs`
- `src-tauri/src/db.rs`
- `src-tauri/src/db_tests.rs`
- `src-tauri/src/tray_helpers.rs`
- `src-tauri/src/serial_periodic.rs`
- `src-tauri/src/serial_periodic_tests.rs`
- `src-tauri/src/commands_relay_tests.rs`
- `src-tauri/src/remote_routes_health_tests.rs`
- `src-tauri/src/remote_server_tests.rs`
- `src-tauri/src/ws_broadcaster.rs` (test fixtures only)
- `src/generated/TodaySummary.ts` (existing Rust exporter only)
- `src/hooks/deskReducer.ts`
- `src/hooks/deskReducer.test.ts`
- `src/test/setup.ts`

The existing tooltip owner is KpiStrip.tsx. Preserve no-observation versus observed zero using optional query results and nullable Rust TodaySummary fields. Invalidate only yesterday context on failed refresh and retain a warning; do not preserve a stale comparison or erase other cached data. Test no rows, actual zero, nonzero, unsupported states, query failure and cache invalidation, plus real logger/AppSnapshot propagation. serial_periodic.rs is a context-to-logger adapter exception to the old serial guard, not authority to change serial transport or session engine. No schema/live DB/new framework changes. Fresh effective absolute-path access is required in the next epic's isolated worktree before these writes. Any further path returns for scope reconciliation.

2026-10-17 refreshed write set: `src-tauri/src/metrics/mod.rs`, existing metric implementations under `src-tauri/src/metrics/`, new `src-tauri/src/metrics/vs_yesterday.rs`, `src-tauri/src/metrics/tests.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/commands_share.rs`, `src-tauri/src/remote_display_state.rs`, `src-tauri/src/serial_periodic_reset.rs`, `src/widgets/one-bar/KpiStrip.tsx`, `src/widgets/one-bar/KpiStrip.test.tsx`, existing KPI tooltip owner, and `src/test/scenarios.ts`. Bind the existing tooltip owner's actual path in the author handoff before edits. Add optional yesterday context to the pure metric API and update all four production compute_all callers; preserve existing metric behavior. No DB/clock access inside the pure metric. Unknown/missing/error omits the new metric rather than fabricating zero; an observed zero remains distinguishable.

Build on then-current main after E027/E014 branches are merged or explicitly parked. Do not import the unmerged fix/analyst-ui-20261009 KpiBadge rewrite. Replace/extend hard-coded zero yesterday remote fixtures with nonzero, equal, missing and error cases; prove the same comparison reaches desktop and LAN/relay. The serial_periodic_reset.rs context-only change is an explicit exception to the former serial*.rs guard. Serialize against T01 because commands.rs overlaps. Keep 3p as the existing estimate and report a concrete increase if implementation warrants it.

PLAN criteria 3-4, decision D2. metrics/vs_yesterday.rs + tests (5 cases), KpiStrip test, remote_display_state test, vs-yesterday mockup scenario.

## Verify

`cargo test --manifest-path src-tauri/Cargo.toml --lib -- e025_`

Details and file:line pointers: [../HANDOFF.md](../HANDOFF.md). Criteria and tests: [../PLAN.md](../PLAN.md).
