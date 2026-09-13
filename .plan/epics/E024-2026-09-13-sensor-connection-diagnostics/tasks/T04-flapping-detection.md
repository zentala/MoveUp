---
formatVersion: 1
type: task
id: E024-T04
epic: E024
status: todo
agent: ts-dev
points: 5
model: standard
effort: focused
confidence: 4
summary: "Detect 3 sub-5-second connections within 60 s and warn in events.log, tray tooltip and Debug tab; clear after a 5-minute connection."
created: 2026-09-13
---
# E024-T04: T04-flapping-detection

## Acceptance

PLAN criterion 4. desk:device-unstable / desk:device-stable; tooltip line; sensor-unstable mockup scenario in src/test/scenarios.ts. Tests: unit (3 short, 2 short, spread over 61 s, 5-min clears), emulator flapping script, TS integration flapping case.

## Verify

`cargo test --manifest-path src-tauri/Cargo.toml --lib -- e024_ && npx vitest run --config vite.config.ts src/hooks src/components/settings`

Details, mental model and file:line pointers: [../HANDOFF.md](../HANDOFF.md). Criteria and tests: [../PLAN.md](../PLAN.md).
