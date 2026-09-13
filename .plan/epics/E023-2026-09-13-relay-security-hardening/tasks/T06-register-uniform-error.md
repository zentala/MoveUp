---
formatVersion: 1
type: task
id: E023-T06
epic: E023
status: todo
agent: ts-dev
points: 1
model: standard
effort: focused
confidence: 4
summary: "register returns the same 403 invalid_license for an unknown and an expired licence key."
created: 2026-09-13
---
# E023-T06: T06-register-uniform-error

## Acceptance

PLAN criterion 6. relay/test/http/register.test.ts: responses deep-equal for unknown vs expired key.

## Verify

`just relay-test`

Details, mental model and file:line pointers: [../HANDOFF.md](../HANDOFF.md). Criteria and tests: [../PLAN.md](../PLAN.md).
