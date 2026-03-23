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
