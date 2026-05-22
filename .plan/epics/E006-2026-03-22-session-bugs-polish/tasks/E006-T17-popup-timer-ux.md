---
id: E006-T17
epic: E006
status: completed
notes: Absorbed into E007 (NotificationService epic)
created: 2026-03-23
original_id: T045
title: Popup Timer UX Redesign
---
# T045: Popup Timer UX Redesign

**Note:** This task was originally scoped for E006 but was absorbed into E007 due to
overlapping scope with the notification service redesign.

## Problem

The popup (OneBarWidget) displays confusing, redundant information:
- Big numbers show remaining time instead of elapsed
- "Standing for 3 min" text duplicates the timer
- Coach message also says "7 min left" — same number repeated in three places
- Timer doesn't visually reset when state changes
- Away state uses green color (should be gray)
- Progress bar inside popup only appears during sitting

## Solution (4 core changes)

1. Big numbers: `elapsed / total` (not `remaining / total`)
2. Unified ProgressBar component (DRY)
3. Away state = gray color
4. Remove redundant "State for Xm" text

Plus 5 delight animations (fade, pulse, shimmer, desaturation, tooltip).

## Status

Absorbed into E007 — see E007 tasks for implementation details.
