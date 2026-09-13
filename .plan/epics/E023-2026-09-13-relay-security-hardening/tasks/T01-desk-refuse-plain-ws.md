---
formatVersion: 1
type: task
id: E023-T01
epic: E023
status: todo
agent: ts-dev
points: 2
model: standard
effort: focused
confidence: 4
summary: "Desk refuses a non-loopback ws:// or http:// relay URL so the desk token never travels over plain TCP."
created: 2026-09-13
---
# E023-T01: T01-desk-refuse-plain-ws

## Acceptance

PLAN criterion 1. Tests e023_ in src-tauri/src/relay_client_tests.rs: remote ws:// and http:// refused, loopback ws:// allowed, wss:// allowed, empty URL -> RELAY_DEFAULT_URL. Refusal visible as relay status last_error.

## Verify

`cargo test --manifest-path src-tauri/Cargo.toml --lib -- e023_`

Details, mental model and file:line pointers: [../HANDOFF.md](../HANDOFF.md). Criteria and tests: [../PLAN.md](../PLAN.md).
