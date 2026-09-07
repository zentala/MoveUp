---
id: E022-T06
status: pending
---
# E022-T06: Desktop credentials + pairing + IPC
## Acceptance
relay_auth.rs (keyring with the mock feature under cfg(test); register, start_pairing, list_viewers, revoke_viewer, disable_relay over reqwest), commands_relay.rs (Tauri commands: relay_register {license_key}, relay_start_pairing -> {code, expires_at, qr_payload}, relay_list_viewers, relay_revoke_viewer {viewer_id}, relay_disable, get_relay_status), AppConfig fields, AppState.relay, client start/restart on config save, registration in lib.rs handler list. Verify: cargo test --manifest-path src-tauri/Cargo.toml --lib -- relay_auth commands_relay config.
