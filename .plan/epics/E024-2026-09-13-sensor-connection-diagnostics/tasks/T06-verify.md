---
formatVersion: 1
type: task
id: E024-T06
epic: E024
status: todo
agent: verify
points: 1
model: standard
effort: focused
confidence: 4
summary: "Run the emulator integration suite against the running debug app and give a per-criterion verdict."
created: 2026-09-13
---
# E024-T06: T06-verify

## Acceptance

just check green; pnpm test:integration against the PM3-started debug app with non-zero test count; release build without emulate_serial_scenario; PASS/FAIL/NOT_CHECKED per PLAN criterion; INDEX row done.

## Verify

`just check && pnpm test:integration`

Details, mental model and file:line pointers: [../HANDOFF.md](../HANDOFF.md). Criteria and tests: [../PLAN.md](../PLAN.md).
