---
updated: 2026-03-24T10:10:00Z
active_epic: E002
active_epic_path: .plan/epics/E002-2026-03-24-ux-integrity
current_wave: 0
---

## Status
- E000 (maintenance) — **done**
- E001 (Timer UX + KPI Dashboard + MetricEngine) — **DONE** (all 11 tasks merged)
- E002 (UX Integrity) — **PLANNED** (13 tasks, 4 waves, 0 started)

## E002 Pre-work (done this session, committed b63c050)
- Fixed standing timer regression (E001-T03 had overwritten fix 4e8d011)
- Fixed popup/overlay ProgressBar to be state-aware (standing = gold)
- Added standLimitSecs to full data flow
- Changed default dev mode to live (not demo)
- Added KPI badge tooltips
- Created mockup gallery (/#/mockup) with 9 scenarios
- Created /popup-mockup skill + ux-design-flow + ux-flow-sync rules
- Created E002 PLAN.md + ORCHESTRATOR.md
- Updated BACKLOG with 6 new items

## Next Steps
1. E002-T01: Update UX-FLOW.md (sync with E001 + target state machine)
2. E002-T02: Away state machine fix (inactive → Away regardless of desk)
3. Visual review with /popup-mockup before implementing layout changes
