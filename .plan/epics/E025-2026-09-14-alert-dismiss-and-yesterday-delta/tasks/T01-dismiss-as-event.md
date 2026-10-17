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

2026-10-17 scope addition: `src-tauri/src/relay_commands.rs` (ack_alert and its stale flag comment) and `src-tauri/src/relay_commands_tests.rs` are in this task's write set alongside alert_popup*.rs, tray_controller*.rs and event wiring. Preserve exactly one policy dismiss/snooze step for a relay acknowledgement; test without sensor traffic and ensure the event path cannot double-advance snooze. No relay Worker changes.

PLAN criterion 1, decision D1. Tests e025_dismiss_* in tray_controller_tests.rs: without sensor and with sensor. take_user_dismissed polling removed.

## Verify

`cargo test --manifest-path src-tauri/Cargo.toml --lib -- e025_`

Details and file:line pointers: [../HANDOFF.md](../HANDOFF.md). Criteria and tests: [../PLAN.md](../PLAN.md).
