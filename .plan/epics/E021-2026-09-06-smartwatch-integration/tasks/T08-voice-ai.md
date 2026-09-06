---
id: E021-T08
status: pending
---
# E021-T08: Voice AI reply client (BYOK OpenRouter)
## Acceptance
voice_ai.rs: VoiceAi::from_env (OPENROUTER_API_KEY, endpoint overridable for tests), reply(transcript, &SessionStateDto) -> Result<Option<String>, VoiceAiError>, fixed system prompt (<=60-word reply, PL/EN matching transcript, ergonomics coaching only), model from AppConfig.voice_ai_model else cheap default, 8s timeout. Not wired to any route yet (T06 does). Tests: voice_ai_tests.rs (wiremock, five cases). Verify: cargo test --manifest-path src-tauri/Cargo.toml --lib -- voice_ai.
