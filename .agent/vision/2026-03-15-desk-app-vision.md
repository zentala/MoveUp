# Desk App — Vision

**Date**: 2026-03-15
**Status**: Defined, not yet implemented

---

## Purpose

Ergonomics tracker for a sit/stand desk. Measures desk height via VL53L1X ToF laser sensor (mounted under the desk, pointing down to the floor), determines whether the user is sitting or standing, tracks session durations, and nudges the user to take breaks.

## Target User

Single user (zentala) working at a desk with a motorized sit/stand setup.

---

## Hardware

- **Sensor**: VL53L1X mounted **under the desk pointing down** to the floor
- **MCU**: Seeed XIAO ESP32-C3, connected via USB (COM3)
- **Height formula**: `desk_height = sensor_distance_to_floor - desk_thickness`
- **State detection**: compare measured height against calibrated thresholds:
  - `sitting_height` ≈ ~75 cm
  - `standing_height` ≈ ~115 cm
  - Threshold between states: midpoint, e.g. 95 cm

---

## User States

| State | Meaning |
|-------|---------|
| `SITTING` | Desk at sitting height, user at computer |
| `STANDING` | Desk at standing height (counts as a break) |
| `WALKING` | Desk at standing height, user away from computer |
| `AWAY` | No keyboard/mouse activity, state unknown |

**STANDING and WALKING are interchangeable as "break"** — both reset session timer.

---

## UI

### System Tray (always visible)
- Text label: `↕ 72 cm` or `↕ STANDING`
- Color: neutral when OK, yellow/red when session overdue

### Floating Window (click on tray icon)
- Current desk height
- Current state (sitting / standing)
- Today's totals: time sitting, time standing
- Current session duration
- Session history for today

### Top-of-Screen Progress Bar (overlay window)
- Thin bar at the very top of primary monitor
- **Green → Yellow → Red** as sitting session progresses toward limit (default: 40 min)
- Disappears / resets when user stands/walks
- At session limit (40 min): popup notification — "You've been sitting for 40 minutes. Time to stand or take a break."

---

## Session Logic

### Session Timer
- Counts continuous sitting time
- Resets or reduces when user takes a break (stands / walks)

### Break Rules (default, configurable later)
| Break duration | Effect |
|----------------|--------|
| < 5 min | No effect |
| 5–9 min | Subtract 20 min from session counter |
| ≥ 10 min | Reset session counter to 0 |

### Activity Detection
- Track keyboard and mouse events to determine if user is at computer
- If no activity for X minutes → mark as `AWAY`
- Computer sleep/lock → also `AWAY`

### Desk State Detection
- Poll sensor every ~2 seconds
- Debounce state changes (require 5s stable reading before switching state)
- Combine desk height + activity to determine full state

---

## Data Storage (SQLite)

Tables:
- `sessions` — start/end, state (sitting/standing), duration
- `height_readings` — timestamped raw mm values (sampled, not every reading)
- `daily_summary` — per-day totals

---

## Device Auto-Detection

Firmware will send an identification string on connect:
```
DEVICE: zntl-desk-sensor v1
```
App scans all COM ports, sends a ping or waits for this string, then auto-connects without user needing to select a port.

---

## Configuration (future)
- Session duration limit (default: 40 min)
- Break thresholds (default: 5 min / 10 min)
- Sitting/standing height thresholds (calibration)
- Progress bar color scheme
- Notification style

---

## Out of Scope (for now)
- Mobile app
- Cloud sync
- Multi-user
- Multiple monitors for progress bar
