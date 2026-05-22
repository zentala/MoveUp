---
id: E004-T03
epic: E004
status: completed
created: 2026-03-20
completed: 2026-03-20
original_id: T015
title: Snooze with deescalating frequency + tone shift
---
# E004-T03: Snooze with deescalating frequency + tone shift

**Depends on:** T013 (AlertManager.dismiss() and Snoozed stage exist)

## Deescalating cooldown
```
snooze_durations: [5, 15, 30, 60]  // minutes
Dismiss #1 -> 5 min   ("just checking")
Dismiss #2 -> 15 min  ("ok, later")
Dismiss #3 -> 30 min  ("I hear you")
Dismiss #4+ -> 60 min ("last reminder")
Standing -> reset snooze_index to 0
```

## Tone shift at dismiss #3
- Dismiss 1-2: neutral ("Time for a stretch!", "Your body needs a break")
- Dismiss 3+: positive ("Even 2 min standing helps blood flow", "Quick stand = fresh mind")
- Messages are configurable array, not hardcoded

## Bar during snooze
```
UNDER LIMIT:     green bar, growing
AT LIMIT:        red bar, PULSING
SNOOZED:         red bar, SOLID (passive reminder)
SNOOZE EXPIRED:  red bar, PULSING again
STANDING:        bar hidden, everything resets
```

## Implementation
- Extends `AlertManager` with `snooze_index: usize`, `snoozed_until: Option<Instant>`, `snooze_durations: Vec<Duration>`, message arrays
- `dismiss()` returns `Vec<AlertAction>` (StopPulse + DismissPopup)
- `popup_message()` selects neutral/positive based on `snooze_index >= tone_shift_threshold`
- `progress < 1.0` while Snoozed -> cancel snooze -> Idle (problem resolved)
- Snooze expiry -> Stage1 (not Stage2 — user sees pulse first, then popup after +2min)

## Tests (~8)
- Dismiss #1-4 snooze durations and message tone
- Standing while Snoozed -> Idle, snooze_index=0
- Progress < 1.0 while Snoozed -> cancel snooze
- Bar variant: solid red during Snoozed, pulsing after expiry
