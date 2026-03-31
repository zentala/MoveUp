# E000 Maintenance — Journal

## Session 2026-03-31 (B) — Timeline debug, overlay fix, seeding fix

- **Goal**: Debug timeline hour markers, fix overlay visibility, clean up widget system
- **Done**:
  - Timeline hour markers fixed — midnight crossing bug (`startHour > now.getHours()` loop never ran). Extracted `computeHourMarkers()` to `@/utils/timeline.ts` (efe5ba2)
  - DB seeding filter — `load_today_totals()` now accepts `after` timestamp to exclude pre-daily-reset sessions. Reset timestamp persisted to store (c1d286f)
  - Overlay NEUTRAL color raised from `#2C2920` to `#5A5548` — visible on black background (committed in prior widget cleanup 38bd754)
  - Widget cleanup (ZenTimeline, Placeholder, WidgetPicker) — already committed by prior agent (38bd754), confirmed files deleted
  - Impro fixes: tauri-dev.ps1 $pid conflict, CSS signal-neutral sync, App.tsx doc (d05ad7e)
- **Decisions**: Single widget (OneBar) — no need for widget picker/registry. ZenTimeline had no unique value.
- **Findings this session**: 4
  - NEUTRAL color near-invisible on black overlay background
  - Timeline hour loop broken across midnight
  - DB seeding ignores daily reset (loads pre-reset data)
  - HourlyBreakTracker not persisted across restarts
- **Improvements logged**: 9 (all implemented via impro)
- **Next**: Verify seeding fix after next daily reset. Break tracker persistence (BACKLOG). Test coverage for timeline edge cases.

## Session 2026-03-31 — Unified Work Cycle + Activity Status

- **Goal**: Add screen break awareness (computer time tracking) and activity status to UI
- **Done**:
  - Screen break nudge toast when standing + computer_time > 60 min (configurable in ergonomic profile)
  - Activity status "Active" / "Idle Xm Ys" in StateIndicator (toggle in Settings)
  - Sitting limit toast changed to "stand up or step away from the screen"
  - Configurable `computer_break_reset_secs` (was hardcoded 300s)
  - `idle_secs`, `away_bout_secs` exposed to frontend
  - 2 research reports (screen time + sit-stand-walk cycle), ADR 011
  - Article backlog item for screen time content
  - Commits: da21cda..66e6304 (16 commits, worktree-based subagent development)
- **Decisions**:
  - Unified cycle (ADR 011) — no separate screen timer, one flow: sit→stand→walk away
  - Screen nudge only when Standing (Sitting has own escalation) — avoids channel conflict
  - `position_changes` counts both desk moves and away returns (= posture changes, not just desk changes)
  - `enable_computer_time_tracking` toggle removed (profile-level control is sufficient)
- **Findings this session**:
  - `position_changes` had confusing semantics (counted away resets but tooltip said "sitting↔standing") — clarified as "posture changes"
  - `communication_policy.rs` hit 276 lines — extracted `maybe_apply_screen_nudge()` → 244 lines
  - `show_activity_status` was not persisted to Rust AppConfig — fixed
  - Standing at desk ≠ screen break (research confirmed: Cornell, CCOHS, CVS studies)
- **Improvements logged**: 0 (all found issues fixed inline)
- **Next**:
  - Dogfood unified cycle with real sensor data
  - Split position_changes into desk_changes + posture_changes (backlog P3)
  - Write screen time article for landing page (backlog P2)
  - Consider 20-20-20 micro-break visual cue (future)

## Session 2026-03-30 — Timeline Skin System

- **Goal**: Fix timeline readability (all segments were indistinguishable brown-gray)
- **Done**: Implemented 3 switchable timeline skins (Semantic, Amber, Clinical) with CSS var-driven architecture. Settings dropdown in More tab. Fixed walking state inconsistency in OneBar. (commits: e43b1c5, 6d22b87)
- **Decisions**: Skins are independent from communication/ergonomic profiles (visual preference ≠ behavioral logic). No external library needed — pure CSS vars + flexbox. Research confirmed Fitbit/Garmin use the same pattern.
- **Findings this session**: 0
- **Improvements logged**: walking state was mapped to standing in OneBar (fixed), `:root` CSS duplication (fixed), DRY hook extraction deferred (LOW)
- **Next**: Dogfood skins with real sensor data, consider skin preview strip in Settings, extract generic `usePersistedSetting` hook if more settings added

## Session 2026-03-30 — Notification Anti-Spam & Backlog Triage

- **Goal**: Fix duplicate notification spam, clean up autostart registry, triage backlog
- **Done**:
  - Fixed dual notifications (toast + popup) firing at sit limit — removed popup from escalation step, visual-only at +5 min (259ce46)
  - Added escalating silence cooldown: 0→5m→15m→30m→silence, configurable per profile via `snooze.notify_cooldowns_secs`
  - Dismiss now resets notify_count (fresh schedule after snooze)
  - Cleaned 3 stale autostart registry entries (zntlDesk, Smart Desk, SmartDesk)
  - Updated all 3 JSON profiles (default, aggressive, demo)
  - Split test file to stay under 250 lines (21 total communication_policy tests)
  - Created ADR 010 — notification escalating silence philosophy
  - Updated CLAUDE.md with notification philosophy section
  - Updated BACKLOG.md: added Bugs section (marked fixed), timeline readability issue, per-profile cooldown tuning
- **Decisions**: Visual-only escalation + escalating silence → [ADR 010](.arch/ADR/010-notification-escalating-silence.md)
- **Findings this session**: 1 (timeline all-gray after color palette change — unreadable)
- **Improvements logged**: 0 (all findings addressed inline)
- **Next**: Timeline color redesign (discuss colors: muted burgundy sitting, soft green standing, gray away)

---

## Session 2026-03-30 — Day Break Credit & PostureBalance Fix

- **Goal**: Fix misleading "You've been sitting most of today" notification firing after 2h sitting + 2h break
- **Done**:
  - Two-level break credit system: Session Break Credit (existing, ADR 008) + Day Break Credit (new, ADR 009)
  - Day Break Credit: breaks >= 6h reset notification flags + daily_score (not KPI counters)
  - PostureBalance guard: requires sitting_seconds_total >= 6h before firing
  - Notification message: concrete "Sitting Xh Ym vs standing Xh Ym" instead of vague "most of today"
  - Event logging: `CREDIT day_break dur=Xs` in events.log
  - New ergonomic profile fields: `day_break_min_secs`, `posture_balance_min_sitting_secs`
  - 8 new tests, 3 fixed existing tests. 393 total Rust tests passing.
  - Commit: 659f8d6
- **Decisions**: [ADR 009](../../.arch/ADR/009-day-break-credit.md) — two break credit levels, 6h threshold
- **Findings this session**: 1
  1. PostureBalance condition `sitting > standing * 2` uses mixed counters (sitting_seconds reduced by credit, sitting_seconds_total not) — edge case where heavy break-taker never triggers despite sedentary day. Tracked in BACKLOG.
- **Improvements logged**: 0 (all addressed inline during impro)
- **Next**: Dogfood day break credit. Timeline readability fix (BACKLOG). Communication policy fixes (duplicate notifications, notification spam — other agent's work unstaged).

## Session 2026-03-30 — Communication Architecture & Profile System

- **Goal**: Design and implement unified communication architecture with profile-driven signal decisions. Fix break credit bug (timer not resetting after 2h sleep).
- **Done**:
  - **Brainstorming**: Identified color inconsistency problem (green/gold/gray ambiguous), designed unified color dictionary (4 colors: none/yellow/red/gray), designed two-profile system (ergonomic + communication)
  - **Design spec**: `docs/superpowers/specs/2026-03-30-communication-architecture-design.md` — full spec with CEO + Eng review
  - **Implementation (13 tasks, 2 waves)**:
    - Wave 1: CommunicationPolicy module, signal types, profile structs+loader, hot-reload, Settings UI dropdowns, AlertManager absorbed, AppConfig slimmed (20 fields → profiles)
    - Wave 2: Tray blink engine (dedicated 50ms thread), green/gold removed from colors, 7 built-in profiles
  - **Break credit fix**: Proportional system (1 min break = 2 min sitting off, configurable multiplier). Sleep gap now applies break credit. [ADR 008](.arch/ADR/008-proportional-break-credit.md)
  - **Impro review**: Path traversal fix, mutex safety, hot-reload tracking active profile, signal warning logs, BlinkPattern validation
  - **Tests**: 381 Rust + 197 TypeScript = 578 total (was 363+187=550 at session start)
  - Commits: 047de75..109ec67 (25 commits)
- **Decisions**:
  - Green removed from system — sitting is never "green" ([ADR 008](../../.arch/ADR/008-proportional-break-credit.md))
  - AlertManager absorbed into CommunicationPolicy — one source of truth
  - Two profile types: ergonomic (limits/scoring/KPI) + communication (escalation/channels/patterns)
  - Proportional break credit replaces 3-tier system
- **Findings this session**: 2
  1. Sleep gap detection existed but didn't apply break credit — only rewound timestamps
  2. Autostart had 3 conflicting registry entries (zntlDesk, Smart Desk, SmartDesk)
- **Improvements logged**: 11 (all fixed: path traversal, mutex safety, hot-reload tracking, signal warnings, blink validation, multiplier clamp, DEFAULT_BLINK_PATTERN, docs updated)
- **Next**: Dogfood with new communication architecture. Test profile switching. Tune escalation thresholds via profiles. Clean up autostart registry entries.

## Session 2026-03-29 — Build fix, autostart, tray icon improvements

- **Goal**: Fix broken build, enable autostart for dev builds, improve tray icon visibility
- **Done**:
  - Fixed `build-report.cjs` monorepo bundle path (2147449)
  - Auto-enable autostart on first run for dev builds (d56c2a0)
  - Tray icon: show icon when sensor disconnected, bigger desk silhouette, 12x12 status dot (6faf2e8)
  - Extract tray icon magic numbers to named constants, improve autostart error handling (37ad1fe)
- **Decisions**: No ADRs — cosmetic/UX changes only
- **Findings this session**: 0
- **Improvements logged**: 0 (all addressed inline)
- **Next**: Generate PNG tray icons (tray-standing.png missing), consider round dot instead of square

## Session 2026-03-26 (night) — Away detection investigation + DB backup + scenario tests

- **Goal**: Debug why Standing+inactive doesn't transition to Away; fix DB persistence; improve test coverage
- **Done**:
  - **Investigation**: Full pipeline trace (serial→is_active→on_reading→handle_state_exit→tray_controller). State machine logic is correct — tests pass. Event logs confirm Standing→Away DOES work at runtime (idle=60s transitions logged). Real bug was overnight sleep inflating sitting_seconds (41954s after restart).
  - **Sleep gap detection**: `on_reading()` now detects >5min gap between readings, rewinds timestamps to prevent inflation. Const `SLEEP_GAP_THRESHOLD_SECS` in session_types.rs.
  - **DB auto-backup**: New `db_backup.rs` module — timestamped backups on every app start, max 30 retained, restore with safety copy. Eager init in `setup()`. IPC commands `list_db_backups`/`restore_db_backup`. Skill `/db-backup` created.
  - **PowerShell launcher**: `tauri-dev.ps1` replaces broken `tauri-dev.sh` (bash couldn't find node). Package.json updated. Old bash script deleted.
  - **8 scenario tests**: Realistic multi-cycle patterns from actual event logs: typical morning (6 transitions), standing oscillation, short breaks, timeline records, app restart, rapid changes, user's exact bug (Stand 18m→Away→Sit), daily reset.
  - **DB round-trip test**: Multi-cycle insert→load_today_totals→get_today_summary verifying persistence.
  - **Diagnostic logging**: `serial_periodic.rs` logs idle_secs on every state transition + debug log when Standing idle>30s.
  - **Path fix**: `com.zentala.desk` → `io.zntl.desk` in 4 docs.
  - **Fullscreen Debug Dashboard**: Added to BACKLOG — timeline + event log overlay + counter dashboard + DB viewer.
  - **Domains added** to global CLAUDE.md (zentala.io, devstage.io, infopill.news, etc.)
  - 356 Rust tests passing (was 340)
- **Decisions**: Sleep detected by gap in readings (not OS events). Break credit logic unchanged. PowerShell is primary shell.
- **Findings this session**: 3
  1. Standing→Away works at runtime (confirmed by event logs) — earlier report was stale or intermittent
  2. Overnight sitting inflation: `sitting_started` timestamp includes sleep duration → 41954s after restart
  3. App identifier is `io.zntl.desk` not `com.zentala.desk` — docs were wrong
- **Improvements logged**: 5 (sleep gap const, dead bash script, DB round-trip test, injectable clock needed, simulate_elapsed is hacky)
- **Next**: Implement Fullscreen Debug Dashboard. Dogfood with sleep detection fix. Consider injectable clock for time-based tests.

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

## Session 2026-03-30 — Session State Persistence

- **Goal**: Persist notification flags and break credit across app restarts (P2 bug from BACKLOG)
- **Done**: New `session_persistence.rs` module — persists flags, credited sitting_seconds, and daily_score to tauri-plugin-store. Date-guarded (discards stale data after midnight). Store cleared on daily reset. DRY `save_via_app()` + `clear()` helpers. 10 new tests including full round-trip scenario. (commit: 5bed2aa)
- **Decisions**: Used tauri-plugin-store (same as AppConfig) instead of SQLite — simpler, no migration. Date guard approach instead of explicit expiry timestamps.
- **Findings this session**: 2
  - Break credit was also lost on restart (not just flags) — `load_today_totals` loaded raw DB value without credit reduction
  - `setup_helpers.rs` had dual init path with `commands.rs::ensure_initialized` — both now load persisted state
- **Improvements logged**: 1 (CommunicationPolicy escalation state not yet persisted — deferred, LOW)
- **Next**: Dogfood persistence with real usage. Consider persisting escalation cooldown state.
