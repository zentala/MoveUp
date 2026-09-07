---
id: E022-T12
status: pending
---
# E022-T12: Wiring + local e2e
## Acceptance
justfile recipes relay-dev, relay-test, relay-deploy; .giter.yaml guard; vite.config.ts /display proxy (closes the first "Dev-mode remote display" backlog entry -- mark it [x] with this task id); scripts/relay-e2e.mjs (+ tests/scripts/relay-e2e.test.ts smoke that the script parses args and refuses to run without a relay URL); PROJECT.xml new files/commands/test layer. Verify (AO manifest uses the smoke test): npx vitest run --config vitest.scripts.config.ts tests/scripts/relay-e2e.test.ts.
