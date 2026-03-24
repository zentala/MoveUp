# E002 Journal

## Session 2026-03-24 09:00–10:10

- **Goal**: Investigate UX problems reported by user, plan E002 epic
- **Done**:
  - Deep investigation: activity tracking, notification systems, state machine, overlay bar, KPI strip
  - Fixed standing timer regression (b63c050) — E001-T03 had overwritten fix from 4e8d011
  - Fixed popup/overlay ProgressBar: state-aware (standing=gold, sitting=green→red)
  - Added standLimitSecs to full TS data flow
  - Changed default dev mode to live (pnpm tauri:dev = live, not demo)
  - Added KPI badge hover tooltips
  - Created mockup gallery + 9 scenarios + /popup-mockup skill
  - Created E002 PLAN.md (13 tasks) + ORCHESTRATOR.md (4 waves)
  - Updated BACKLOG with 6 new items
  - Created rules: ux-design-flow.md, ux-flow-sync.md
- **Decisions**:
  - Away state: inactive → Away regardless of desk height (Walking = future/smartwatch)
  - Mockup-first flow: scenarios → visual review → approval → implement → verify
  - One progress bar per context (popup inline + screen overlay, not two in popup)
  - State colors consistent everywhere (gold=standing, gray=away, etc.)
  - KPI "Session" → "Screen time" with eye icon (T05)
  - Layout redesign: timer top, KPI middle, timeline bottom (T08)
- **Findings this session**: 8
  - Away state unreachable (Sitting when desk low + inactive)
  - Walking misnomer (don't know user walks)
  - Two notification systems without coordination
  - E001-T03 regression: destroyed standing timer fix from day before
  - KPI "Session" label confusing (means screen time, reads as sitting session)
  - Timeline only shows completed sessions (no live block)
  - Two progress bars in popup (overlay + inline = redundant)
  - Overlay bar defaults to demo in dev mode (not obvious)
- **Improvements logged**: 0 (all captured in E002 PLAN.md)
- **Next**: E002-T01 (UX-FLOW.md update), then T02 (Away state fix)
