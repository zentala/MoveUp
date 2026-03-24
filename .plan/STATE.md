---
updated: 2026-03-24T18:00:00Z
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
- E008 (UX Integrity, 2026-03-24) — **DONE** (v0.2.0, 508 tests)

## Version
- Current: `v0.2.0` (tagged)
- Rule: each new epic bumps `0.MAJOR.0`

## Test Totals
- Rust: 339 tests
- TypeScript: 169 tests
- Total: 508

## Migration
- 2026-03-24: Full PM migration completed (.claude/→.plan/epics/, .arch/)
- Old tasks, journals, plans, reports migrated to retroactive epics E001-E006
- E001/E002 renumbered to E007/E008 (chronological ordering)

## Known Gaps
- **HourlyBreakTracker not wired** — struct written + 8 tests, but `#[allow(dead_code)]`; metric shows placeholder data from position_changes. Need to integrate into session loop.
- **`height_readings` DB table** — schema exists, never populated. Could serve as time-series storage.

## Next Steps
1. Wire HourlyBreakTracker into session loop (small task, E000-maintenance)
2. Or: pick next epic — candidates: SQLite time-series, timeline full window, extended height stabilization
3. Or: run app live, dogfood changes, collect feedback
