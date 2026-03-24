# E000 Maintenance — Journal

## Session 2026-03-23

- **Goal**: Implement T044 snapshot logging + event log
- **Done**:
  - Created `snapshot_logger.rs` — per-minute JSON snapshots to `logs/YYYY-MM-DD/HH-MM.json`
  - Created `event_logger.rs` — append-only event log to `logs/YYYY-MM-DD/events.log`
  - Created `setup_helpers.rs` — extracted window positioning + notification setup from lib.rs
  - Added `break_credit` field to `ReadingResult` for CREDIT event logging
  - Wired loggers through `serial.rs` → `serial_periodic.rs` call chain
  - All 8 event types: START, DEVICE, STATE, CREDIT, ALERT, NOTIF, RESET, DEVICE lost
  - 7-day log retention with cleanup on startup
  - Post-review: 5 improvements (port_name in snapshots, pub(crate) Loggers, APP_VERSION const, removed redundant lock, 3 break_credit tests)
  - Commits: cae3b36, 81e37cf (merge), 66fb32f (improvements)
- **Decisions**: Loggers as separate `Loggers` Tauri managed state (not in AppState) since they need app_data_dir from setup
- **Findings this session**: 0
- **Improvements logged**: 5 (all fixed in same session)
- **Next**: Manual verify with running app; PM structure migration

## Session 2026-03-24

- **Goal**: Full PM structure migration (.claude/ + .agent/ → .plan/epics/ + .arch/)
- **Done**:
  - Created 6 retroactive epics E001-E006 for all pre-migration work
  - Renumbered existing E001→E007, E002→E008 (chronological ordering)
  - Moved 44 task files to epic tasks/ folders with new IDs and frontmatter
  - Moved overlay docs to .arch/overlay/ (DEVELOPER-GUIDE, KNOWLEDGE-BASE, MODE-COMPARISON, decisions/)
  - Moved 4 vision docs from .agent/ to .plan/vision/
  - Created .arch/ scaffolds: ARCHITECTURE.md, HISTORY.md, DDD.md, ADR/_template.md
  - Rebuilt STATE.md (8 epics), DONE.md (72 tasks), BACKLOG.md (open + future)
  - Archived root TASKS.md to .plan/reports/, replaced with epic pointer
  - Updated CLAUDE.md path references
  - Cleaned up: .claude/tasks/, journals/, plans/, raports/, alerts/, overlay/, test-runs/, .agent/
  - Commit: cfb77c1
- **Decisions**: Epic numbering follows chronology (oldest=lowest). Open tasks from old epics go to BACKLOG, not kept in done epics.
- **Findings this session**: 0
- **Improvements logged**: 0
- **Next**: Pick E009 from BACKLOG.md candidates (State Machine Redesign, SQLite time-series, or dogfood E008)
