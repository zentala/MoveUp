# Floating Window Redesign — Design Document

_Date: 2026-03-21 | Source: brainstorming session_
_Mockups: `apps/desk/.superpowers/brainstorm/14480-1774063099/`_

## Problem

Current floating window is a broken tracker dashboard:
- "stop" button is a dev command (not user-facing)
- "changes" counter doesn't update in real-time (60s poll from DB)
- "standing" = 0 (active session not counted until ended)
- "break" label confusing (shows time not-sitting, but labeled "break")
- `sessions[]` fetched from backend but never rendered
- No visual hierarchy — current state and today's totals equally weighted
- No coaching — purely informational

## Design Decisions

### 1. Widget architecture (T031)
Popup content is a pluggable widget. Core provides data, widget handles presentation.
User can switch between widget styles via settings.

### 2. One bar, one meaning (T032)
Progress bar = limit usage. Fills when sitting, drains when standing/away.
Same visual, same direction, always the same meaning: less = better.

### 3. Three states, three temperatures
- **Sitting:** red, escalates from calm→warm→hot→burning
- **Standing:** green, triumphant
- **Away:** gray, neutral (legitimate break, same as standing)

### 4. Timeline as hero
Proportional blocks showing today's rhythm. Hover for details.
Ghost rhythm lines show ideal change frequency.

### 5. Coach speaks in one sentence
Not statistics — actions. "Wstań w ciągu 5 min." Not "70% limitu, 5 zmian, +32 pkt."

### 6. Quick glance (5 seconds)
Only 3 elements: timeline + timer + coach.
Everything else removed or folded into these three.

### 7. limitRemaining is the hero number
Big number shows how much limit is left — not how long you've been sitting.
When standing: number decreases as you "pay off" the sitting debt.

## States

| State | Detection | Timeline color | Temperature | Bar behavior |
|-------|-----------|---------------|-------------|-------------|
| Sitting | desk low + keyboard active | red | calm→burning | fills up |
| Standing | desk high + keyboard active | green | green | drains down |
| Away | no keyboard/mouse > 60s | gray | gray | drains down |

## Break credit rules

| Break duration | Effect |
|---|---|
| < 5 min | No effect |
| 5–14 min | Partial reset (subtract 20 min from limit used) |
| ≥ 15 min | Full reset (limit back to 0) |

Standing and Away count equally as breaks.
