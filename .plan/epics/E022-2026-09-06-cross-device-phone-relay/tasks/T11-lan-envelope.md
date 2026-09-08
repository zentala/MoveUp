---
id: E022-T11
status: done
updated: 2026-09-08
evidence: 71b186d
---
# E022-T11: LAN path on the shared envelope
## Acceptance
remote_server.rs envelope wrap + remote_lan_enabled gate in setup_helpers.rs + read-only test; docs/REMOTE_DISPLAY.md LAN section mentions the toggle (T14 rewrites the rest). Verify: cargo test --manifest-path src-tauri/Cargo.toml --lib -- remote_server.
