---
formatVersion: 1
type: task
id: E023-T07
epic: E023
status: done
agent: verify
points: 2
model: standard
effort: focused
confidence: 4
summary: "Verify the merged wave, security re-review of F1-F6, record decisions, close the epic."
created: 2026-09-13
---
# E023-T07: T07-verify-and-rereview

## Acceptance

PLAN criterion 7: just check + just relay-test pass on the independently reviewed source integrated into main; security-reviewer confirms F1-F6 closed; review report statuses become FIXED with source/test references; D1-D5 in decisions.jsonl; INDEX row done. The obsolete hash-reporting requirement is retired by the 2026-10-17 controlling handoff.

Completed 2026-10-17: independent rounds 1–2 PASS. Ordinary diff confirms integrated product source is unchanged from the tested candidate, so duplicate full-suite execution was unnecessary. Exact counts, retained failures and behavioral RED/GREEN are in [JOURNAL.md](../JOURNAL.md).

## Verify

`just check && just relay-test`

Details, mental model and file:line pointers: [../HANDOFF.md](../HANDOFF.md). Criteria and tests: [../PLAN.md](../PLAN.md).
