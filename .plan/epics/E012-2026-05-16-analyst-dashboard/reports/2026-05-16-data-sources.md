# Data Sources Research — desk app

**Date:** 2026-05-16
**For epic:** [E012 Analyst Dashboard](../PLAN.md)
**Linked from:** [PLAN.md](../PLAN.md)

## 1. Main UI data contract

**Component**: `apps/desk/src/widgets/OneBarWidget.tsx:56-83`

**Props/fields rendered (`WidgetProps`):**
- `state` → StateIndicator (Sitting / Standing / Walking / Away)
- `deskHeightCm` → StateIndicator display
- `currentSessionSecs` / `limitSecs` → OneBarTimer (elapsed + progress bar)
- `limitRemaining`, `limitRatio` → progress bar fill (green→yellow→red)
- `idleSecs` → "Active" / "Idle Xm Ys" badge
- `todayChanges`, `todayStandingSecs`, `todaySittingSecs`, `todayScore` → KpiStrip (4 KPI badges)
- `todaySessions` → OneBarTimeline (history bar chart)
- `continuousComputerSecs` → screen break nudge logic
- `metrics` → KPI metrics (standing%, position changes/hr, hourly breaks, longest session)
- `breakSecs`, `breakResetThreshold`, `breakResetProgress` → break credit visualizations
- `previousSession` → last break effectiveness indicator

## 2. Data sources

### 2.1 Hardware sensor (VL53L1X ToF, COM)
- Driver: `apps/desk/src-tauri/src/serial.rs:1-130` | `serial_parser.rs:40-47`
- Baud 115200, probe `"DEVICE: zntl-desk-sensor v1"`
- Parsed: `distance: <mm> mm` → `DistanceReading { mm, cm, timestamp }`
- Events emitted: `desk:distance`, `desk:device-connected`, `desk:device-lost`, `desk:sensor-error`

### 2.2 SQLite (`desk.db`)
Path: `{app_data_dir}/desk.db`. Table `sessions`:
```
id, started_at, ended_at, state, duration_seconds,
sitting_seconds, standing_seconds, position_changes,
session_limit_secs, date_local
```
Queries: `db_queries::get_today_summary`, `load_today_totals`.

### 2.3 events.log
Path: `{app_data_dir}/logs/YYYY-MM-DD/events.log`. Format `HH:MM:SS TYPE details`.
Event types: `STATE`, `DEVICE`, `ALERT`, `RESET`, `CREDIT`, `START`, `NOTIF`, `AUTOSTART`.
Retention: 7 days (cleanup_old_logs).

### 2.4 Minute snapshots
Path: `{app_data_dir}/logs/YYYY-MM-DD/HH-MM.json`. One per minute, ~500 B.
Fields: `ts, state, sitting/standing/break_seconds, session_limit_secs, stand_limit_secs, desk_height_cm, position_changes, sitting_seconds_total, idle_secs, away_bout_secs, continuous_computer_secs, max_continuous_computer_secs, longest_computer_session_secs, current_session_secs, standing_session_secs, connected, port, version, metrics[]`.
Retention: 7 days.

### 2.5 Profiles (JSON, hot-reload)
Path: `{app_data_dir}/profiles/{ergonomic|communication}/*.json`.
- **Ergonomic**: limits, scoring, kpi thresholds, break credit multiplier
- **Communication**: escalation steps, snooze cooldowns, channels, messages
Built-ins: default / aggressive / gentle / silent / demo (comm); standard / strict / relaxed / demo (ergo).

### 2.6 Store (tauri-plugin-store)
Keys: `sitting_mm`, `standing_mm`, `desk_thickness_mm`, `persisted_session_state`, `reset_after`, active profile IDs.

### 2.7 System signals
- Idle: `activity.rs` (`GetLastInputInfo`); threshold 60s for Away
- Continuous computer time: tracked in `session.rs`, reset after Away ≥ 300s
- Focused window: NOT tracked (reserved for smartwatch)

### 2.8 Remote display (WebSocket)
Server on `:3390`. Message types: `snapshot`, `desk:state-changed`, `desk:device-connected/lost`, `desk:daily-reset`, `heartbeat`. Capacity 64 messages.

## 3. Derived signals

### State machine
States: `Sitting | Standing | Walking | Away`. Debounce: 5 consecutive readings.
Inputs: desk height (mm), idle_secs, sleep gap (>300s → break credit + rewind).

### Break credit (ADR 008)
- `<break_min_secs` → none
- `[break_min_secs, 2×break_min_secs)` → partial (0.5× multiplier)
- `≥ 2×break_min_secs` → full (1.0× multiplier)
- Reduction: `sitting_secs -= break_secs × multiplier` (default 2.0)

### Scoring
- `+pts_standing_per_min` per min standing
- `−pts_sitting_per_min` per min sitting (penalty)
- `+pts_session_bonus` when standing session ≥ `standing_target_secs`
- Reset: midnight OR day break (≥ 6h)

### Thresholds / alerts
- Sit limit escalation (steps 1..N from comm profile), snooze [0,5,15,30..] min
- Stand limit escalation (`standing_max_secs` 5400s default)
- Screen break nudge (`max_continuous_computer_secs` 3600s default)
- PostureBalance: `sitting_seconds_total ≥ 6h AND sitting > standing × 2`
- Day break credit (`day_break_min_secs` 21600s): reset notif flags + daily_score

## 4. Storage & retention

| Data | Where | Retention |
|------|-------|-----------|
| Current session | in-memory | until restart |
| Calibration, profiles | tauri-store | persistent |
| Persisted session state | tauri-store | until daily reset |
| Sessions history | `desk.db` | no limit |
| Snapshots | `logs/.../HH-MM.json` | 7 days |
| events.log | `logs/.../events.log` | 7 days |
| Backups | `backups/` | manual user-triggered |

## 5. IPC surface

### Commands (existing — see `commands.rs`, `commands_config.rs`, `commands_profiles.rs`, `commands_backup.rs`, `commands_share.rs`)
- Session: `list_ports`, `start_auto_connect`, `stop_reading`, `get_session_state`, `get_dashboard_state`, `get_connected_port`, `get_today_summary`, `inject_reading` (debug), `trigger_test_notification`
- Settings: `get_settings`, `save_settings`, `calibrate`, `set_session_limit`, `set_stand_limit`, `get_overlay_state`
- Profiles: `list_*_profiles`, `get_active_profiles`, `switch_*_profile`, `open_profile_in_editor`, `duplicate_profile`
- Backup: `backup_database`, `restore_database`
- Share: `share_stats`

### Events emitted (existing — see `serial.rs`, `serial_periodic.rs`, `tray.rs`)
`desk:distance`, `desk:device-connected`, `desk:device-lost`, `desk:device-missing`, `desk:sensor-error`, `desk:state-changed`, `desk:daily-reset`, `desk:show-widget`, `desk:show-settings`, `desk:popup-theme`.

## 6. Gaps — what an analyst dashboard should surface

1. **Minute snapshots** — captured but never visualized (height timeline, idle progression, max continuous computer session trends)
2. **Break credit detail** — partial vs full counts, daily credit applied, missed opportunities (<60s)
3. **Sensor connection history** — port changes, disconnect frequency, auto-reconnect success rate
4. **Daily score trajectory** — per-hour breakdown, impact of each standing session, today vs 7d avg
5. **State machine edge cases** — sleep/suspend gap distributions, debounce failures, smoothing effects
6. **Notification policy** — escalation step over time, cooldown timeline, suppressed vs delivered
7. **KPI history** — standing % trajectory, position change rate per hour, longest session growth
8. **Profile switch impact** — before/after metrics on switch, profile efficacy comparison
9. **Computer activity patterns** — away bout distribution, idle ↔ state transition correlation, nudge effectiveness
10. **Error events** — sensor error frequency, recovery rate, port enum failures

These are the candidate views for the Explorer tab.
