---
updated: 2026-03-30T18:00:00Z
active_epic: E000
active_epic_path: .plan/epics/E000-maintenance
current_wave: n/a
---

## Status
- E000 (maintenance) — open (permanent)
- E001–E009 — partially done (see ORCHESTRATOR.md per epic)
- E010 (Marketing Launch) — 19/22 tasks DONE (3 human tasks remain)

## Version
- Current: `v0.3.0`

## Test Totals
- Rust: 393 tests
- TypeScript: 197 tests
- Total: 590

## Recent: Communication Architecture (2026-03-30)

Major refactor completed in E000-maintenance:
- **CommunicationPolicy** replaces AlertManager as central signal decision engine
- **Two profile types**: ergonomic (limits/scoring/KPI) + communication (escalation/channels/patterns)
- **7 built-in profiles**: default, aggressive, gentle, silent, demo + standard, strict, relaxed
- **Tray blink engine**: configurable blink patterns with 50ms timer thread
- **Color system**: green/gold removed, unified 4-color dictionary
- **Break credit**: proportional system (configurable multiplier in ergonomic profile)
- **Sleep gap fix**: machine sleep now applies break credit
- **25 commits**, 73 files changed, +4100/-1600 lines

## E010 Progress (unchanged)
- 19/22 tasks completed
- 3 remaining (human action):
  - T02: Collect real usage data (screenshots, photos)
  - T14: Marketing videos (2-3)
  - T18: Record 15-second hero GIF/video

## Next Steps
1. Dogfood day break credit + communication architecture
2. Fix timeline readability (BACKLOG — colors indistinguishable)
3. Commit unstaged communication policy fixes (duplicate notifications, spam)
4. E010 human tasks (photos, video, GIF)
5. Deploy landing page + waitlist endpoint
