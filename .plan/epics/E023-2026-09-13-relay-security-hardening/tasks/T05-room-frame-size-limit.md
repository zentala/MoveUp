---
formatVersion: 1
type: task
id: E023-T05
epic: E023
status: todo
agent: ts-dev
points: 2
model: standard
effort: focused
confidence: 4
summary: "Cap room WebSocket frames at 16 KiB before JSON.parse; oversize closes with 1009."
created: 2026-09-13
---
# E023-T05: T05-room-frame-size-limit

## Acceptance

PLAN criterion 5. relay/test/room/handshake.test.ts: 16 KiB+1 byte -> close 1009 unparsed, exactly 16 KiB -> handled, empty frame -> existing bad_frame. Measure bytes, not string length.

## Verify

`just relay-test`

Details, mental model and file:line pointers: [../HANDOFF.md](../HANDOFF.md). Criteria and tests: [../PLAN.md](../PLAN.md).
