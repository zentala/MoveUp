---
formatVersion: 1
type: task
id: E024-T01
epic: E024
status: todo
agent: ts-dev
points: 5
model: standard
effort: focused
confidence: 4
summary: "Put serial I/O behind a SerialBackend trait with a production and an emulator backend so the real scan loop runs in tests."
created: 2026-09-13
---
# E024-T01: T01-serial-backend-seam-and-emulator

## Acceptance

SystemSerialBackend is moved code (probe/baud/PING unchanged); EmulatorSerialBackend scripts happy/no_ports/no_match/enumeration_failed/flapping; scan_once steppable with injected clock and event sink. Tests e024_seam_*: happy script -> DEVICE connected COM3 and readings reach the session. serial.rs stays <= 250 lines.

## Verify

`cargo test --manifest-path src-tauri/Cargo.toml --lib -- e024_`

Details, mental model and file:line pointers: [../HANDOFF.md](../HANDOFF.md). Criteria and tests: [../PLAN.md](../PLAN.md).
