---
formatVersion: 1
type: task
id: E025-T03
epic: E025
status: todo
agent: ts-dev
points: 3
model: standard
effort: focused
confidence: 4
summary: "Bring back the today-vs-yesterday comparison as a Rust KPI metric shown in the KPI strip and the phone snapshot."
created: 2026-09-14
---
# E025-T03: T03-vs-yesterday-metric

## Acceptance

2026-10-17 refreshed write set: `src-tauri/src/metrics/mod.rs`, existing metric implementations under `src-tauri/src/metrics/`, new `src-tauri/src/metrics/vs_yesterday.rs`, `src-tauri/src/metrics/tests.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/commands_share.rs`, `src-tauri/src/remote_display_state.rs`, `src-tauri/src/serial_periodic_reset.rs`, `src/widgets/one-bar/KpiStrip.tsx`, `src/widgets/one-bar/KpiStrip.test.tsx`, existing KPI tooltip owner, and `src/test/scenarios.ts`. Bind the existing tooltip owner's actual path in the author handoff before edits. Add optional yesterday context to the pure metric API and update all four production compute_all callers; preserve existing metric behavior. No DB/clock access inside the pure metric. Unknown/missing/error omits the new metric rather than fabricating zero; an observed zero remains distinguishable.

Build on then-current main after E027/E014 branches are merged or explicitly parked. Do not import the unmerged fix/analyst-ui-20261009 KpiBadge rewrite. Replace/extend hard-coded zero yesterday remote fixtures with nonzero, equal, missing and error cases; prove the same comparison reaches desktop and LAN/relay. The serial_periodic_reset.rs context-only change is an explicit exception to the former serial*.rs guard. Serialize against T01 because commands.rs overlaps. Keep 3p as the existing estimate and report a concrete increase if implementation warrants it.

PLAN criteria 3-4, decision D2. metrics/vs_yesterday.rs + tests (5 cases), KpiStrip test, remote_display_state test, vs-yesterday mockup scenario.

## Verify

`cargo test --manifest-path src-tauri/Cargo.toml --lib -- e025_`

Details and file:line pointers: [../HANDOFF.md](../HANDOFF.md). Criteria and tests: [../PLAN.md](../PLAN.md).
