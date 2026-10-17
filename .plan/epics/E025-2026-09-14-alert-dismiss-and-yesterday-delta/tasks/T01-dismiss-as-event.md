---
formatVersion: 1
type: task
id: E025-T01
epic: E025
status: todo
agent: ts-dev
points: 5
model: standard
effort: focused
confidence: 4
summary: "Popup dismiss becomes a Tauri event so it reaches CommunicationPolicy even with no sensor connected."
created: 2026-09-14
---
# E025-T01: T01-dismiss-as-event

## Acceptance

2026-10-17 Windows test-executable correction: add exactly `src-tauri/build.rs` to the source write set. Actual E025 Cargo test compiled but returned STATUS_ENTRYPOINT_NOT_FOUND before executing any test; retain exit1/zero tests, not behavioral RED. After confirming the existing Common Controls v6 manifest/import mismatch, use the existing Tauri-generated resource and normal toolchain: preserve tauri_build::build and add Windows/MSVC OUT_DIR link search; an already accepted test module may link that resource into the unit-test executable using the supported static resource/-bundle form. No parked-branch merge/import, system/DLL/registry changes, dependency updates, new runner or generator. Fresh one-path effective access before build.rs writes. Verify with the same meaningful E025 test command; a new failure calls for bounded diagnosis, not unrelated tooling repair. This routine test-linkage correction is included in the disclosed 5p T01 estimate unless actual additional complexity is reported.

Estimate updated 2026-10-17 from 3p to 5p after actual author analysis; existing product acceptance and task history remain.

2026-10-17 scope addition: `src-tauri/src/relay_commands.rs` (ack_alert and its stale flag comment) and `src-tauri/src/relay_commands_tests.rs` are in this task's write set alongside alert_popup*.rs, tray_controller*.rs and event wiring. Preserve exactly one policy dismiss/snooze step for a relay acknowledgement; test without sensor traffic and ensure the event path cannot double-advance snooze. No relay Worker changes.

PLAN criterion 1, decision D1. Tests e025_dismiss_* in tray_controller_tests.rs: without sensor and with sensor. take_user_dismissed polling removed.

## Verify

`cargo test --manifest-path src-tauri/Cargo.toml --lib -- e025_`

Details and file:line pointers: [../HANDOFF.md](../HANDOFF.md). Criteria and tests: [../PLAN.md](../PLAN.md).
