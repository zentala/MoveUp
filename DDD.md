# DDD.md — Domain Glossary for Desk App

## User States

- **DeskState** — Enum: `Sitting`, `Standing`, `Walking`, `Away`. Detected via laser sensor height + keyboard/mouse activity.
- **Sitting** — Desk below midpoint threshold; user at keyboard.
- **Standing** — Desk above midpoint threshold; user active (keyboard/mouse input detected).
- **Walking** — Desk above midpoint threshold; user inactive (no input). Counts as a break.
- **Away** — No sensor data or extended inactivity. Counts as a break.

## Session Management

- **SessionManager** — Rust state machine owning all session logic. Processes sensor readings, tracks durations, applies break credits.
- **SessionState** — Internal mutable state: current DeskState, accumulated seconds, timestamps, score.
- **SessionStateDto** — Serialized snapshot of SessionState sent to the frontend via IPC.
- **CompletedSession** — A finished sitting segment with start/end timestamps and duration.

## Break Credit

- **Break credit** — Sitting-time reduction earned by standing. Applied on Standing-to-Sitting transition.
- **BreakCredit::None** — Break < 5 min. No credit.
- **BreakCredit::Partial** — Break 5-9 min. Subtracts 20 min from accumulated sitting time.
- **BreakCredit::Full** — Break >= 10 min. Resets sitting time to zero.

## Limits and Targets

- **Sitting limit** (`sit_limit_mins`) — Max continuous sitting before alert. Default 45 min.
- **Standing target** (`standing_target_mins`) — Daily standing goal. Completing one cycle = one lap.
- **Stand max** (`stand_limit_mins`) — Max continuous standing before "sit down" reminder.

## Scoring

- **Lap** — One completed `standing_target_mins` cycle of standing. Awards bonus points.
- **Daily score** — Points accumulated today: 1 pt/min standing + lap bonuses + position change bonuses.
- **Position change** — A Sitting<->Standing transition. Each earns bonus points.

## Calibration

- **sitting_mm** — Sensor distance (mm) when desk is in sitting position.
- **standing_mm** — Sensor distance (mm) when desk is in standing position.
- **desk_thickness_cm** — Desk surface thickness subtracted from sensor reading.
- **Midpoint** — `(sitting_height_cm + standing_height_cm) / 2`. Threshold between Sitting and Standing.

## Debounce

- **DEBOUNCE_COUNT** — Number of consecutive consistent sensor readings (5) required before a state transition.

## UI / Widget System

- **Widget** — A pluggable React component rendering desk data. Receives `WidgetProps`.
- **WidgetProps** — Pure presentation contract: connected, state, timers, score, limits.
- **Widget registry** — Map of widget IDs to components. Active widget selected via store.
- **Overlay** — Native WinAPI progress bar at top of screen showing sitting session progress.
- **DataSource** — Overlay data mode: `Demo` (cycling animation), `Live` (real sensor), `Mock` (simulated).
- **Overlay variant** — Visual style: `0` solid color, `1` gradient, `2` pulsing.

## Hardware

- **Device** — Seeed XIAO ESP32-C3 with VL53L1X sensor. Identified by `DEVICE: zntl-desk-sensor v1` on serial connect.
- **Activity** — Presence of keyboard/mouse events within last N seconds. Distinguishes Standing from Walking.

## Notifications

- **Inactivity** — No position change for 60 min.
- **Posture balance** — Sitting > 2x standing duration.
- **Praise halfway** — Standing reached 50% of daily target.
- **Standing target reached** — Standing met full daily target.
