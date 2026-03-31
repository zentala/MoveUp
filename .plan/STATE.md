---
updated: 2026-03-31T12:00:00Z
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
- Rust: 418 tests
- TypeScript: 170 tests
- Total: 588

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

## Recent: Notification Anti-Spam (2026-03-30)
- Removed duplicate notifications (toast + popup) — visual-only escalation at +5 min
- Added escalating silence (0→5m→15m→30m→silence), configurable per profile
- Cleaned 3 stale autostart registry entries
- ADR 010 — notification escalating silence philosophy

## Recent: Timeline Skin System (2026-03-30)
- 3 switchable skins: Semantic (default), Amber, Clinical
- CSS var-driven (`--tl-*`), skin class on widget container
- Settings dropdown in More tab, persisted in AppConfig
- Fixed walking state inconsistency (OneBar mapped walking→standing)
- 2 commits, 14 files changed

## Recent: Session State Persistence (2026-03-30)
- Notification flags, break credit, daily_score persisted to tauri-plugin-store
- Survives restarts within same day (date-guarded)
- Store cleared on daily reset
- 10 new tests (round-trip, serde, daily-reset interaction)
- 1 commit, 8 files changed (2 new + 6 modified)

## Recent: Unified Work Cycle (2026-03-31)

- **Screen break nudge**: toast when standing + computer_time > 60 min
- **Activity status**: "Active" / "Idle Xm Ys" in StateIndicator
- **Configurable**: `max_continuous_computer_secs` (60 min), `computer_break_reset_secs` (5 min)
- **ADR 011**: unified sit-stand-walk cycle (no separate screen timer)
- **Research reports**: screen time + eye health, sit-stand-walk cycle
- **KPI label**: "Changes" → "Posture" (counts desk moves + away returns)
- 16 commits, 54 files changed, +651/-104 lines (source only)

## Next Steps
1. Dogfood unified work cycle with real sensor data
2. Split position_changes into desk_changes + posture_changes (P3)
3. Write screen time article for landing page (P2)
4. E010 human tasks (photos, video, GIF)
5. Deploy landing page + waitlist endpoint
