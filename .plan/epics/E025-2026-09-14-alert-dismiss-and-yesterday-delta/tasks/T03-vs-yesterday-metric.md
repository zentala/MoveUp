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

PLAN criteria 3-4, decision D2. metrics/vs_yesterday.rs + tests (5 cases), KpiStrip test, remote_display_state test, vs-yesterday mockup scenario.

## Verify

`cargo test --manifest-path src-tauri/Cargo.toml --lib -- e025_`

Details and file:line pointers: [../HANDOFF.md](../HANDOFF.md). Criteria and tests: [../PLAN.md](../PLAN.md).
