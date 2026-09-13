---
formatVersion: 1
type: task
id: E024-T05
epic: E024
status: todo
agent: main
points: 1
model: standard
effort: focused
confidence: 4
summary: "Document the new diagnostics where the next person looks first."
created: 2026-09-13
---
# E024-T05: T05-docs

## Acceptance

CLAUDE.md Hardware diagnosis order, rules/logging.md, UX-FLOW.md, ARCHITECTURE.md serial section, PROJECT.xml, D1-D5 in decisions.jsonl.

## Verify

`grep -q 'DEVICE scan ports=' CLAUDE.md`

Details, mental model and file:line pointers: [../HANDOFF.md](../HANDOFF.md). Criteria and tests: [../PLAN.md](../PLAN.md).
