---
id: E022-T01
status: pending
---
# E022-T01: Protocol contract
## Acceptance
src-tauri/src/remote_protocol.rs (+ remote_protocol_tests.rs, mod lines in lib.rs): Envelope<T>, Hello, Welcome, Command, CommandResult, DeskStatus, ErrorBody, close-code consts, COMMAND_ALLOWLIST with arg bounds as data; ts_rs derives -> src/generated/; src/remote/protocol.ts (zod) + protocol.test.ts; tests/fixtures/relay-protocol/*.json (>=12 files: hello-desk, hello-viewer, welcome-online, welcome-offline, event-snapshot, event-state-changed, desk-status, command-ack, command-set-limits, command-switch-profile, command-result-ok, command-result-err, ping, error). Verify: cargo test --manifest-path src-tauri/Cargo.toml --lib -- remote_protocol and npx vitest run --config vite.config.ts src/remote/protocol.test.ts.
