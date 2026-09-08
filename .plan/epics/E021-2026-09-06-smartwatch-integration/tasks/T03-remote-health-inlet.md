---
id: E021-T03
status: done
updated: 2026-09-08
evidence: fe9594a
---
# E021-T03: Remote health push inlet + auth
## Acceptance
remote_auth.rs (require_token constant-time compare), remote_routes_health.rs (POST /display/health, body cap 1KiB, schema steps_today/heart_rate_bpm/hrv_rmssd_ms/source_id/measured_at_ms -> PushHealthSource registered in aggregator, stale after 1h), mount in build_router, RemoteDisplayState.health, /display/api includes it, docs/REMOTE_DISPLAY.md gets curl example + token setup. Tests: remote_routes_health_tests.rs, remote_auth_tests.rs (real seam route->source->aggregator). Verify: cargo test --manifest-path src-tauri/Cargo.toml --lib -- remote_routes_health remote_auth.
