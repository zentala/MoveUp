---
id: E006
created: 2026-03-22
status: done
title: Session Bugs & Polish
---
# E006 — Session Bugs & Polish

## Goal

Reactive polish sprint (not pre-planned). Fix bugs discovered during real-world use of the desk app after E005 landed: session persistence, sensor race conditions, UX label issues, design review findings, and general polish.

## Period

2026-03-22 to 2026-03-23

## Scope

- Session 1 (2026-03-22 morning): Critical bugs — insert_session never called, sensor race condition, "CONNECTING" label, standing timer showing sitting value, UX-FLOW.md creation
- Session 2 (2026-03-22 afternoon): Design audit — 7 CSS findings fixed, window positioning, tray click behavior, memory leak in tray icon
- Session 3 (2026-03-23): Parallel tasks — settings tabbed layout, HeightStabilizer module, popup closes on blur, "No sessions yet" serde fix, session timer unit tests

## Key Findings

- `insert_session()` was never called from production code — `save_session_state()` created DB rows without `ended_at`, invisible to all queries
- Sensor sends ~10 readings/s (not 1/s), readings jump +/-27mm — needs stabilization (T037)
- `desk:device-connected` event fired before React listener mounted — race condition
- `SessionRow` JSON field names mismatched TS `SessionEntry` (serde rename fix)
- `Box::leak` in `generate_tray_icon` caused 4KB/s memory leak
