---
formatVersion: 1
type: task
id: E025-T01
epic: E025
status: todo
agent: ts-dev
points: 3
model: standard
effort: focused
confidence: 4
summary: "Popup dismiss becomes a Tauri event so it reaches CommunicationPolicy even with no sensor connected."
created: 2026-09-14
---
# E025-T01: T01-dismiss-as-event

## Acceptance

PLAN criterion 1, decision D1. Tests e025_dismiss_* in tray_controller_tests.rs: without sensor and with sensor. take_user_dismissed polling removed.

## Verify

`cargo test --manifest-path src-tauri/Cargo.toml --lib -- e025_`

Details and file:line pointers: [../HANDOFF.md](../HANDOFF.md). Criteria and tests: [../PLAN.md](../PLAN.md).
