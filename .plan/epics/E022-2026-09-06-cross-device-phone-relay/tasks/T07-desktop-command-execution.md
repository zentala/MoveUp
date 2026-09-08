---
id: E022-T07
status: done
updated: 2026-09-08
evidence: 646935c
---
# E022-T07: Desktop command execution
## Acceptance
relay_commands.rs (+ tests): execute(cmd, &AppState-like struct) -> CommandResult; allowlist + bounds from remote_protocol.rs; stale ts > 30s rejected; extract switch_*_by_name from commands_profiles.rs; ack_alert per Mental model; events.log lines REMOTE <name> viewer=<id> ok|err=<code> and REMOTE DENIED <name>. Wire into relay_client.rs's receive arm. Verify: cargo test --manifest-path src-tauri/Cargo.toml --lib -- relay_commands commands_profiles.
