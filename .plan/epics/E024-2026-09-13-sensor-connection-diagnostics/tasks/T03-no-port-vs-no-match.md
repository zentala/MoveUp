---
formatVersion: 1
type: task
id: E024-T03
epic: E024
status: todo
agent: ts-dev
points: 3
model: standard
effort: focused
confidence: 4
summary: "Tell zero COM ports, ports without the sensor, and a failed enumeration apart in events.log, the device-missing event and the Debug tab."
created: 2026-09-13
---
# E024-T03: T03-no-port-vs-no-match

## Acceptance

PLAN criteria 1-3. Change-only logging; desk:device-missing payload {reason, ports} in Rust, src/types.ts and deskReducer; Debug row. Tests: serial_diagnostics_tests.rs transitions, serial_scan_emulator_tests.rs for the three scripts, tests/integration/device-diagnostics.test.ts for the three reasons.

## Verify

`cargo test --manifest-path src-tauri/Cargo.toml --lib -- e024_ && npx vitest run --config vite.config.ts src/hooks src/components/settings`

Details, mental model and file:line pointers: [../HANDOFF.md](../HANDOFF.md). Criteria and tests: [../PLAN.md](../PLAN.md).
