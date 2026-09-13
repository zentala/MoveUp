---
formatVersion: 1
type: task
id: E023-T04
epic: E023
status: todo
agent: ts-dev
points: 2
model: standard
effort: focused
confidence: 4
summary: "Trust only CF-Connecting-IP for rate limiting; a missing header fails closed with 500 relay_misconfigured."
created: 2026-09-13
---
# E023-T04: T04-client-ip-fail-closed

## Acceptance

PLAN criterion 4. relay/test/http/router.test.ts: no header -> 500, X-Forwarded-For only -> 500, header present -> normal. helpers.ts accepts ip: null to omit the header.

## Verify

`just relay-test`

Details, mental model and file:line pointers: [../HANDOFF.md](../HANDOFF.md). Criteria and tests: [../PLAN.md](../PLAN.md).
