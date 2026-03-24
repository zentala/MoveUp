---
updated: 2026-03-25T00:30:00Z
active_epic: none
active_epic_path: null
current_wave: null
---

## Status
- E000 (maintenance) — open (permanent)
- E001 (App Foundation, 2026-03-15) — **DONE**
- E002 (Overlay Progress Bar, 2026-03-16) — **DONE**
- E003 (Installer & Distribution, 2026-03-16) — **DONE**
- E004 (Session Alerts & Snooze, 2026-03-20) — **DONE**
- E005 (UX Communication + Widgets, 2026-03-21) — **DONE**
- E006 (Session Bugs & Polish, 2026-03-22) — **DONE**
- E007 (KPI Dashboard + Timer UX, 2026-03-23) — **DONE** (v0.1.0)
- E008 (UX Integrity, 2026-03-24) — **DONE** (v0.2.0)

## Version
- Current: `v0.2.0` (tagged)
- Rule: each new epic bumps `0.MAJOR.0`

## Test Totals
- Rust: 327 tests
- TypeScript: 169 tests
- Total: 496

## New This Session
- `/desk-debug` skill for snapshot/event log analysis
- `sitting_seconds_total` field — raw KPI accumulator
- Metrics now included in minute snapshots
- position_changes seeded from DB on restart
- Gamification philosophy documented in memory
- BACKLOG: gamification research + scoring redesign tasks

## Next Steps
1. Dogfood app — verify standing_pct, position_changes, metrics in snapshots
2. Gamification research report (BACKLOG task)
3. Notification flag persistence fix (BACKLOG P2)
4. Pick next epic
