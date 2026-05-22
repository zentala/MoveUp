---
created: 2026-03-15
status: completed
title: App Foundation
---
# E001 — App Foundation

## What
Establish the core architecture for the desk ergonomics tracker: persistent config store, session state machine, daily reset, notification system, settings UI, and test framework.

## Why
The app needed a solid foundation before adding visual features (overlay, alerts). Key decisions had to be locked early to prevent rework.

## Key Architectural Decisions

1. **Rust is single source of truth** — `SessionManager` (Rust) owns all session counters. Frontend reads via Tauri commands/events, never queries DB directly.

2. **`rusqlite` replaces `tauri-plugin-sql`** — `tauri-plugin-sql` only exposes SQLite to JavaScript; Rust had no API access. Switched to `rusqlite = { version = "0.31", features = ["bundled"] }`. `src/db.ts` deleted entirely.

3. **`on_reading()` returns `ReadingResult` struct** — not a bare Option. Contains `state_change` and `completed_session` fields. `serial.rs` reads `result.completed_session` and calls `db::insert_session()`.

4. **Config commands in `config.rs`**, not `commands.rs` — prevents 250-line pre-commit violation.

5. **`desk:db-error` event** on any rusqlite failure — `useDesk.ts` subscribes and shows error banner.

## Scope
- Config store with `rusqlite` + `tauri-plugin-store`
- Daily reset logic (midnight counter reset)
- Settings panel UI (replaces CalibrationWizard)
- Notification preference toggles (3 types)
- Stand reminder limit
- Position changes counter
- Fix `standing_secs` placeholder
- 3-layer test framework (unit, integration, E2E)

## Out of Scope
- `position_changes` DB persistence (T09 deferred to backlog)
- Dynamic tray icon (T10 cancelled, replaced by T016 in later epic)
- Rail pulse animation (T11 deferred)
- Yesterday delta arrow (T12 deferred)

## Acceptance Criteria
- [x] Calibration survives restart
- [x] Sit/stand limits survive restart
- [x] Corrupt store falls back to defaults
- [x] Daily reset clears counters at midnight
- [x] Settings panel replaces CalibrationWizard
- [x] 3 notification types independently configurable
- [x] Standing seconds tracked correctly
- [x] Position changes counter works
- [x] Test framework: unit + integration + E2E scaffolded
- [x] 80% coverage gate enforced
