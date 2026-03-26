# E000 Maintenance — Journal

## Session 2026-03-26 — Away DB fix + DRY refactor + overlay default

- **Goal**: Fix Away time inflating standing%, refactor state classification, fix overlay demo default
- **Done**:
  - **Away DB fix**: `db_sessions.rs` and `db_queries.rs` if/else lumped Away into standing. Fixed with `match` on state strings. (commit e8f8b09)
  - **DRY refactor**: Centralized state classification — `DeskState::from_db_str()`, `is_desk_position()`, `is_standing_like()`. `accumulate_state_duration()` shared helper eliminates 3x duplicated match blocks. Removed dead `away_secs` field. (commit 1a24df3)
  - **Overlay default**: Changed from Demo (debug) to Live (always). Demo now requires explicit flag. Updated docs + test. (commit f0496db)
  - **`/ergo-review` skill**: Created for usage log analysis + motivation UX discussion
  - **Memory**: Progressive break credit curve, motivation analytics process
  - **BACKLOG**: "Motivation Analytics & Adaptive Coaching" section (5 items)
  - **Refactored `get_yesterday_totals`** → `get_totals_for_date(date)` + wrapper (testable)
  - 345 Rust tests passing
- **Decisions**: Live is always default overlay mode; Demo only via explicit env var/flag
- **Findings this session**: 2
  1. DB loading path was never fixed in 4b368c3 (only in-memory path)
  2. Overlay Demo default was the designed behavior, not a regression — but wrong for dogfooding
- **Improvements logged**: 0 (all fixed in same session)
- **Next**: Dogfood with live overlay + correct standing%. Run `/ergo-review` after a day of data.

## Session 2026-03-25 — E009 Planning

- **Goal**: Design and plan Remote Display epic (phone as desk dashboard)
- **Done**:
  - Brainstormed architecture: web kiosk (PC serves React+WS to phone browser)
  - Created full epic E009 with 7 tasks across 4 waves (~16h estimated)
  - ADR 001: web kiosk over Tauri Mobile, PWA, standalone
  - Updated vision doc with Phase 1/2/3 remote display roadmap
  - CEO review (HOLD SCOPE): fixed metrics broadcast, dev proxy, today summary, error handling
  - Eng review (SMALL CHANGE): fixed SQLite hot path (→ cache), stale closure bug, Vite proxy test
  - Commits: `b1d937b` (epic), `addfb51` (eng review fixes)
- **Decisions**: axum for HTTP+WS server, same React build with hook abstraction, broadcast channel
- **Findings this session**: 0
- **Improvements logged**: 0 (planning only, no code changes)
- **Next**: Implement E009 (Wave 0→1→2→3)

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

## Session 2026-03-24 (evening)

- **Goal**: Audit BACKLOG.md accuracy, fix standing % bug, wire HourlyBreakTracker
- **Done**:
  - Backlog audit: 4 tasks marked done (E001-T09, E001-T11, E001-T12, E004-T05)
  - Removed 6 stale BACKLOG sections (State Machine, KPI Strip, KPI rename, NotificationService, Coach Messages, Height stabilizer)
  - Fixed standing % bug: Away time was counted as standing (85% instead of ~15%). Added `standing_bout_started` field to track actual Standing time independently from break duration (commits: 97e1862, 4b368c3)
  - Wired HourlyBreakTracker into session loop: tick_away/tick_active from accumulate_ongoing, exposed in SessionState, metric uses real data, removed dead_code annotation
  - Dead code cleanup: removed gap_handler.rs, height_readings DB schema, unused HourlyBreakTracker methods (serialize, deserialize, hours_missed)
  - Fixed 4 tests missing standing_bout_started, added bouncing test (Standing→Away→Standing→Sitting)
  - Commits: 97e1862, 4b368c3, c7a071b, 709a2fc, 47005d9
- **Decisions**: None architectural — maintenance fixes
- **Findings this session**: 1 (standing_seconds bug: Away→Sitting added full break_dur to standing_seconds)
- **Improvements logged**: 0 (all fixed in same session)
- **Next**: Dogfood app with fixes, pick next epic from BACKLOG

## Session 2026-03-24 (night)

- **Goal**: Fix standing % showing 100%, fix snapshot data quality, create debug skill
- **Done**:
  - **Root cause: standing % = 100%**: `sitting_seconds_total` not seeded from DB on restart → 0 → 100%. Fixed `load_today_totals()` to seed it.
  - **Root cause: standing % = 40%**: `sitting_seconds` reduced by break credit (full reset to 0), inflating standing_pct denominator. Added `sitting_seconds_total` field — raw accumulator never modified by break credit. Metric now uses this field.
  - **Fix: metrics empty in snapshots**: `serial_periodic.rs` passed `Vec::new()` instead of computed metrics. Now computes all 4 metrics before writing snapshot.
  - **Fix: position_changes = 0 after restart**: `load_today_totals()` didn't restore it. Refactored to return `TodayTotals` struct with `position_changes` (session row count - 1 as proxy).
  - **Fix: sitting_seconds_total in DTO**: Added to `SessionStateDto` so snapshots and IPC expose raw sitting time.
  - **Fix: metrics use live values**: `get_dashboard_state()` now injects live `sitting_seconds_total` and `standing_seconds` before passing to MetricEngine.
  - **Created `/desk-debug` skill**: Reads snapshots + event log from AppData, reconstructs timeline, verifies metrics against calculated values, reports inconsistencies.
  - **Gamification philosophy**: Saved to memory — negative score is valid gameplay, comeback mechanics over always-positive.
  - **BACKLOG updated**: New "Gamification & Scoring" section with 3 tasks (gamification research report, scoring redesign umbrella, notification flag persistence).
  - 327 Rust tests passing, 0 failures
- **Decisions**: Use `sitting_seconds_total` (raw) for KPI, keep `sitting_seconds` (credit-adjusted) for session limit tracking. Two fields, two purposes.
- **Findings this session**: 3
  1. `sitting_seconds` used for both session limit AND KPI — break credit corrupted KPI
  2. `load_today_totals()` didn't seed new fields → 100% after restart
  3. Notification flags not persisted → spam on every restart (added to BACKLOG)
- **Improvements logged**: 0 (all fixed in same session)
- **Next**: Dogfood with all fixes. Gamification research report. Pick next epic.

## Session 2026-03-24 (late night)

- **Goal**: Fix Away time counted as standing in DB queries
- **Done**:
  - **Root cause**: `db_sessions.rs` and `db_queries.rs` used `if state == "Sitting" { ... } else { standing += duration }` — any non-Sitting state (including Away) inflated standing totals when loaded from DB. In-memory logic was already correct (fixed in 4b368c3), but DB path was missed.
  - Fixed `load_today_totals()`, `get_yesterday_totals()`, `get_today_summary()` — all now use `match` on state string: Sitting → sitting, Standing|Walking → standing, _ → excluded
  - Added `away_secs` field to `TodayTotals` struct (tracked separately, not lost)
  - Refactored `get_yesterday_totals()` → `get_totals_for_date(date)` + wrapper (testable with any date)
  - 3 new tests: Away exclusion for today totals, today summary, date-parameterized query
  - 330 Rust tests passing
  - Commit: e8f8b09
- **Decisions**: None architectural — bugfix completing prior in-memory fix
- **Findings this session**: 1 (DB loading path was never fixed in 4b368c3, only in-memory path was)
- **Improvements logged**: 0
- **Next**: Dogfood with all fixes. Gamification research report. Pick next epic.
