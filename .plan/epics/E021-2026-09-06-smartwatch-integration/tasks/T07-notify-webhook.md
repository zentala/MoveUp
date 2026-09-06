---
id: E021-T07
status: pending
---
# E021-T07: Ntfy/generic webhook notifier
## Acceptance
notify_webhook.rs: WebhookNotifier::from_env_or_config, send(&Notification) spawned on tokio (never awaited), 5s timeout, one retry on 5xx/timeout, JSON body {title, message, priority, tags}. Call it where the sit-limit toast fires. Settings toggle + URL field bound to AppConfig.notify_webhook_url. Tests: notify_webhook_tests.rs (wiremock, four paths). Verify: cargo test --manifest-path src-tauri/Cargo.toml --lib -- notify_webhook.
