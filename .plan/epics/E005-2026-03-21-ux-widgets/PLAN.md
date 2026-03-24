---
id: E005
created: 2026-03-21
status: done
title: UX Communication + Widget System
---
# E005 — UX Communication + Widget System

## Goal

Transform the floating window from a broken tracker dashboard into a pluggable widget system with coaching, timeline visualization, and proper session state communication.

## Architecture — Key Decisions

### 1. Rust owns all state
- `SessionManager` (Rust) is the single source of truth for all counters
- Frontend receives data ONLY via Tauri commands or events (`desk:state-changed`, `desk:distance`)
- `AppState` in `commands.rs` contains: `session`, `overlay`, `alert_manager`, `alert_popup`

### 2. Config persistence via tauri-plugin-store
- `AppConfig` loaded from store in `config.rs` via `AppConfig::load(store)`
- All fields have `#[serde(default)]` — safe for missing keys
- `standing_target_mins` has `#[serde(alias = "stand_limit_mins")]` — automatic migration

### 3. EventBus — no direct imports between modules
- Events: `desk:distance` (~every 1s), `desk:state-changed` (on state transition)
- `tray_controller.rs` subscribes to both and updates overlay + tray + alert manager

### 4. File size limit: 250 lines
- Pre-commit hook blocks commits if any file exceeds 250 lines
- `session.rs` was 1348 lines — T027 split it first

### 5. Overlay color + variant are independent
- `overlay.update(progress, color_rgb)` — sets color and progress
- `overlay.set_variant(v)` — sets animation (0=solid, 1=gradient, 2=pulsing)
- Variant 2 used by: alert Stage1 (red) AND lap flash (gold) — depends on last color set

### 6. Score accumulation pattern
- `accumulate_score_tick(config)` — SessionManager method, called from `serial.rs` every ~1s
- Score lives in `SessionState.daily_score: f32`, reset at daily reset

### 7. Standing progress bar lap counting
- `standing_session_secs` = current continuous standing session (resets when user sits)
- `standing_seconds` = cumulative standing today (never resets mid-day)
- Gold bar uses `standing_session_secs` for progress and flash detection
- Total laps indicator uses `standing_seconds`

### 8. Widget system
- `WidgetProps` is the single interface between core and presentation
- Core computes `limitRemaining` (decreasing counter) — widgets don't compute session logic
- `active_widget: String` in AppConfig, persisted via tauri-plugin-store
- Widget registry is a static TS map, no dynamic loading
- Each widget is `FC<WidgetProps>` — pure presentation component

## Scope

- Wave 1: Split oversized Rust files (session.rs 1348L, db.rs 467L, serial.rs 464L, commands.rs 264L)
- Wave 2: Fix bugs (tooltip standing, notifications, floating window spec)
- Wave 3: Features (standing gold bar, points system, welcome popup)
- Wave 4: Fixes (floating window bugs, tray icon redesign)
- Wave 5: Widget system (architecture + One Bar + Timeline Zen)

## Acceptance Criteria

- All oversized files split under 250 lines
- Tooltip updates every second while standing
- Notifications working (test command added)
- Standing shows gold progress bar with lap flash
- Points system tracking daily score
- Widget system with 2 implementations (One Bar, Timeline Zen)
- All tests passing
