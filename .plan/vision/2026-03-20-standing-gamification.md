# Vision: Standing Gamification & Points System

**Date:** 2026-03-20
**Status:** Vision — partially implemented (T022 — standing progress bar)

---

## Core Idea

Make standing **rewarding**, not just "not bad". The sitting bar nags the user when they've been sitting too long. The standing bar **celebrates** standing. Two modes, two emotional tones.

---

## Standing Progress Bar (gold)

When user stands up, the red/invisible sitting bar transforms into a **gold progress bar** that fills from 0 → 15 min (default standing target).

- Bar starts empty (left edge) when user stands
- Fills gold as standing time accumulates
- At 15 min: bar is full — a "second lap" appears on the left and starts filling (another 15-min cycle starts)
- Visual: the "lap 2" segment is a slightly brighter/deeper gold to indicate it's bonus territory
- Default standing target: **15 min** (configurable in settings)
- Bar hidden (or resets) when user sits down

---

## Points System

A running score reflecting posture balance.

### Earning / Losing Points

| Action | Points |
|--------|--------|
| 1 minute standing | +1 |
| Complete standing session (≥ target duration, e.g. 15 min) | +5 bonus |
| 15 min standing total | +20 (15×1 + 5 bonus) |
| Each additional 15 min standing lap | +20 again |
| 1 minute sitting | −0.5 |

### Goal
Points should balance out with "good" posture behavior. Standing 15 min compensates ~30 min of sitting.

### Configuration (in Settings)
- `standing_target_min: 15` — duration for a "complete session"
- `points_per_standing_min: 1.0`
- `points_per_complete_session: 5`
- `points_per_sitting_min: -0.5`

---

## UI Concepts

### Tray tooltip
Show daily points: `↕ 72cm  🏆 +38 pts`

### Floating window
- "Today's score: +38 pts"
- Standing streak indicator

### Standing progress bar (implemented in T022)
- Top-of-screen bar, gold color, fills left→right over 15 min
- At 100%: lap animation (brief flash), second lap starts from left

---

## Future Ideas

- **Streak tracking**: "3 days in a row hitting standing target"
- **Daily/weekly score history**: sparkline in floating window
- **Adaptive nudges**: if score consistently negative, increase alert intensity
- **Milestones**: "First 100 pts standing today!"
- **Leaderboard**: future (if multi-device)

---

## Related Tasks

- `T022` — Implement standing progress bar (gold, 0→15min) — **immediate**
- `T015` (done) — Snooze logic
- `T019` — Success notifications + gamification (deferred to BACKLOG)
