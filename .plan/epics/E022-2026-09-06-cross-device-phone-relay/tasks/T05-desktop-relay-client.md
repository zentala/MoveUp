---
id: E022-T05
status: done
updated: 2026-09-08
evidence: 8b7f5e4
---
# E022-T05: Desktop relay client
## Acceptance
src-tauri/src/relay_client.rs (+ relay_client_tests.rs, relay_status.rs): tokio task with tokio-tungstenite (rustls-tls-webpki-roots), hello -> welcome -> snapshot -> forward ws_tx subscription; ping 25s / 60s timeout; backoff 1->60s +/-20% jitter (injectable sleeper for tests); close-code -> status mapping with no reconnect on 4402/4403/4409; a ClientHandle { stop(), restart() }. Tests use an in-process tokio-tungstenite server on port 0. Add deps in Cargo.toml. Verify: cargo test --manifest-path src-tauri/Cargo.toml --lib -- relay_client relay_status.
