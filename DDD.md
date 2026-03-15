# Domain Definitions — Desk App

## Core Concepts

**Session**
A continuous period of sitting. Starts when desk transitions to sitting height + user is active. Ends when user stands, walks, or goes away. Stored in SQLite.

**Break**
A continuous period NOT sitting (STANDING or WALKING). Duration determines effect on session counter:
- < 5 min → no effect
- 5–9 min → session counter -= 20 min
- ≥ 10 min → session counter reset to 0

**Desk Height**
Physical height of the desk surface in cm, derived from: `sensor_distance_to_floor - desk_thickness`. Calibrated per-installation.

**Sitting Height**
Desk height range considered "sitting position" (e.g. 68–80 cm).

**Standing Height**
Desk height range considered "standing/walking position" (e.g. 100–125 cm).

**State**
Current user state. One of: `SITTING`, `STANDING`, `WALKING`, `AWAY`.
- `STANDING` vs `WALKING` = same desk height, distinguished by keyboard/mouse activity.
- `AWAY` = no keyboard/mouse activity regardless of desk height.

**Activity**
Presence of keyboard or mouse events within the last N seconds. Used to distinguish STANDING from WALKING, and to detect AWAY.

**Session Limit**
Configurable maximum sitting duration before user is notified (default: 40 min).

**Progress Bar**
Thin overlay at top of primary monitor. Fills green→red over one session limit duration. Resets on break.

**Device**
Seeed XIAO ESP32-C3 with VL53L1X sensor. Identified by firmware string `DEVICE: zntl-desk-sensor v1` on serial connect.
