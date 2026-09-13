---
formatVersion: 1
type: task
id: E024-T02
epic: E024
status: todo
agent: ts-dev
points: 3
model: standard
effort: focused
confidence: 4
summary: "Debug-only IPC emulate_serial_scenario plus TS scenario helpers, so integration tests drive the running app on the emulator."
created: 2026-09-13
---
# E024-T02: T02-emulator-ipc-and-ts-scenarios

## Acceptance

Command gated like inject_reading (commands.rs:217, lib.rs:220); cfg test proves absence from release handler list; tests/emulator gains SerialScenario + emulateSerial(); device-reconnect.test.ts:40-56 rewritten to assert desk:device-connected port on scenario happy.

## Verify

`cargo test --manifest-path src-tauri/Cargo.toml --lib -- e024_`

Details, mental model and file:line pointers: [../HANDOFF.md](../HANDOFF.md). Criteria and tests: [../PLAN.md](../PLAN.md).
