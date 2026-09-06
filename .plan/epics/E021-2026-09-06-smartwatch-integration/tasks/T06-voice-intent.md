---
id: E021-T06
status: pending
---
# E021-T06: Voice intent parser + voice routes + notes DB
## Acceptance
voice_intent.rs (parse(&str) -> Intent with Snooze(u16)/Note/WalkStart/WalkEnd, table-driven regexes, PL+EN), remote_routes_voice.rs (POST /display/voice, body cap 4KiB, transcript/lang/captured_at_ms, token required; pipeline: EventLogger VOICE line -> db_voice_notes insert -> intent side effect (Snooze -> CommunicationPolicy snooze fields; Walk*/Note -> none) -> voice_ai::reply when configured -> VoiceAck broadcast -> webhook push via T07), db_voice_notes.rs (table + list_voice_notes(day) IPC in commands_health.rs, catalog entry in commands_catalog_sources.rs). Tests: voice_intent_tests.rs (12-phrase table), remote_routes_voice_tests.rs (real seam to parser + real temp SQLite), db_voice_notes_tests.rs. Verify: cargo test --manifest-path src-tauri/Cargo.toml --lib -- voice_intent remote_routes_voice db_voice_notes.
