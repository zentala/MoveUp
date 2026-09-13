---
formatVersion: 1
type: task
id: E023-T03
epic: E023
status: todo
agent: ts-dev
points: 1
model: standard
effort: focused
confidence: 4
summary: "Mask the licence key input in Settings (type=password, autoComplete=off)."
created: 2026-09-13
---
# E023-T03: T03-licence-key-masked

## Acceptance

PLAN criterion 3. src/components/settings/RemoteSection.test.tsx asserts type=password on the Licence key field.

## Verify

`npx vitest run --config vite.config.ts src/components/settings/RemoteSection`

Details, mental model and file:line pointers: [../HANDOFF.md](../HANDOFF.md). Criteria and tests: [../PLAN.md](../PLAN.md).
