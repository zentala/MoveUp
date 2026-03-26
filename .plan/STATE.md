---
updated: 2026-03-26T12:00:00Z
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
- E009 (Remote Display — Web Kiosk, 2026-03-25) — **DONE** (v0.3.0)

## Version
- Current: `v0.3.0`
- Rule: each new epic bumps `0.MAJOR.0`

## Test Totals
- Rust: 345 tests
- TypeScript: 189 tests
- Total: 534

## New This Session
- Away time excluded from standing in DB queries (3 files, shared helper)
- DeskState centralized classification (from_db_str, is_desk_position, is_standing_like)
- Overlay default changed to Live (was Demo in debug)
- `/ergo-review` skill for motivation analytics
- Progressive break credit + analytics process saved to memory

## Next Steps
1. Dogfood remote display on real phone — visual verification
2. Manual e2e scenarios (reconnect, sensor disconnect, multiple clients)
3. Gamification research report (BACKLOG task)
4. Notification flag persistence fix (BACKLOG P2)
5. Pick next epic
