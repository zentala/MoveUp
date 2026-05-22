---
created: 2026-03-20
status: completed
title: Session Alerts & Snooze
---
# E004 — Session Alerts & Snooze

## Goal

Implement a progressive alert escalation system that nudges the user to take breaks when sitting too long. The system must be non-intrusive, configurable, and respect user agency through a snooze mechanism with deescalating frequency.

## Scope

- AlertManager: pure state machine (no WinAPI, no UI) that emits `Vec<AlertAction>` for `tray_controller.rs` to execute
- Progressive escalation: Idle → Stage1 (bar pulses) → Stage2 (popup) → Snoozed
- Standing at any stage resets everything back to Idle
- Bar behavior per state:
  - **Under limit**: green bar, growing (normal)
  - **At limit (Stage1)**: red bar, PULSING (`set_variant(2)`)
  - **Stage2**: red bar, PULSING + popup window visible
  - **Snoozed**: red bar, SOLID (passive reminder — "I heard your dismiss")
  - **Snooze expired → Stage1**: red bar, PULSING again
  - **Standing → Idle**: bar hidden, everything resets

## Key Decisions

- AlertManager is a pure state machine returning `Vec<AlertAction>` — no WinAPI calls, no UI dependencies
- CEO decision: Stage1 (bar pulse) and Stage2 (popup) shipped together — bar pulse alone too subtle
- AlertPopup uses WinAPI thread with `AtomicBool` dismiss flag (not Tauri WebviewWindow) to overlay fullscreen apps
- Popup position: right-bottom corner near system tray, always-on-top, non-modal
- Time source: `Instant` (monotonic), `stage_entered_at` field
- Safety: if progress drops below 1.0, also dismiss popup (missed event mitigation)

## Snooze Logic

- Deescalating cooldown — more dismisses = longer intervals (app backs off):
  - Dismiss #1 → 5 min
  - Dismiss #2 → 15 min
  - Dismiss #3 → 30 min
  - Dismiss #4+ → 60 min (capped)
  - Standing → reset `snooze_index` to 0
- Tone shift at dismiss #3 — if nagging doesn't work, stop nagging. Inspire instead:
  - Dismiss 1-2: neutral ("Time for a stretch!", "Your body needs a break")
  - Dismiss 3+: positive ("Even 2 min standing helps blood flow", "Quick stand = fresh mind")
  - Messages are configurable arrays, not hardcoded

## Prerequisites

- T-OVR-010 (split overlay_renderer.rs) must complete first — can't add alert integration to a 1000+ line file
- `set_variant()` method on OverlayRenderer must exist

## Acceptance Criteria

- AlertManager transitions Idle→Stage1→Stage2 correctly
- Bar pulses at session limit (Stage1), popup appears after +2min (Stage2)
- Standing resets everything to Idle
- Dismiss enters Snoozed with deescalating intervals
- Tone shifts from neutral to positive at dismiss #3
- All tests pass (~23 total: 15 for alert manager + 8 for snooze)
