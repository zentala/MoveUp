---
id: E006-T16
epic: E006
status: done
created: 2026-03-23
completed: 2026-03-23
original_id: T044
---
# T044: Minute-by-minute snapshot logging + event log

## Goal

Two logging layers so the developer (and Claude agent) can diagnose
what happened at any point in the day — what data the app had and
what decisions it made.

**Motivation:** Recent git history shows 6+ debugging sessions (standing timer zero,
connected flag wrong, serde case mismatch) that would have been trivial to diagnose
with persistent state snapshots.

## Architecture Overview

- **Layer 1: Minute snapshots** — JSON files at `logs/YYYY-MM-DD/HH-MM.json`, ~500 bytes each
- **Layer 2: Event log** — append-only text at `logs/YYYY-MM-DD/events.log`
- **Cleanup**: 7-day retention, runs on startup
- **Key decision**: Loggers passed as `Arc<SnapshotLogger>` and `Arc<EventLogger>` through call chain, NOT through `AppState`
- **Key decision**: Snapshots use `SessionStateDto` directly via serde flatten — prevents field drift

## New Files
- `snapshot_logger.rs` (~100 lines)
- `event_logger.rs` (~50 lines)
- `setup_helpers.rs` (~60 lines, extracted from lib.rs)

## Tests (~9)
- 6 snapshot/cleanup tests
- 3 event logger tests

## Acceptance Criteria

- [ ] JSON snapshot every 60s
- [ ] Event log with 8 event types
- [ ] 7-day cleanup on startup
- [ ] All I/O errors -> log::warn + skip (never panic)
- [ ] 9 unit tests pass
- [ ] `cargo test` passes
