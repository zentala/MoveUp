---
formatVersion: 1
type: task
id: E023-T02
epic: E023
status: todo
agent: ts-dev
points: 3
model: standard
effort: focused
confidence: 4
summary: "Close the max_viewers race with one conditional D1 insert; the losing concurrent pair gets 409 viewer_limit."
created: 2026-09-13
---
# E023-T02: T02-pair-viewer-cap-atomic

## Acceptance

PLAN criterion 2. relay/test/http/pair.test.ts: two concurrent pairs at cap-1 -> statuses [201,409], one active viewers row. Pre-check at pairing.ts:61-65 stays.

## Verify

`just relay-test`

Details, mental model and file:line pointers: [../HANDOFF.md](../HANDOFF.md). Criteria and tests: [../PLAN.md](../PLAN.md).
