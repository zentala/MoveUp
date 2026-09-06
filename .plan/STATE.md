---
updated: 2026-09-06T02:30:52
active_epic: none (E011 and E012 code complete; E011 close-out ceremony pending, see E016 in reports/2026-09-06-pelny-przeglad-architektury-i-release.md)
planning_epic: E015
planning_epic_path: .plan/epics/E015-2026-09-06-engine-single-truth
current_wave:
  - E015 wave 1 planned, not dispatched (handoff written 2026-09-06)
---

## Status
- **2026-09-06 full review** — engine, backend, frontend, release, plans:
  [reports/2026-09-06-pelny-przeglad-architektury-i-release.md](reports/2026-09-06-pelny-przeglad-architektury-i-release.md).
  Root cause of the "counter resets on standing" complaint found (popup reads
  `current_session_secs`); fix planned as E015. Roadmap E016→E015→E017→E018/E019→E020.
- **E013 (signed Tauri release and PM3 deployment)** — planning; extends the
  deferred E003-T07 release-automation task. No implementation task is active.
- **E011 (autostart hardening)** — done, code-complete 2026-05-07, ceremony closed
  via E016. All five tasks (T01–T05) are `[x]` in [DONE.md](./DONE.md). See
  [PLAN.md](./epics/E011-2026-05-07-autostart-hardening/PLAN.md).
- E000 (maintenance) — open (permanent)
- E001–E009 — partially done (see ORCHESTRATOR.md per epic)
- E010 (Marketing Launch) — 19/22 tasks DONE (3 human tasks remain)

## E011 progress — complete

All five tasks are done and recorded in [DONE.md](./DONE.md); the epic's
close-out ceremony (HISTORY entry, IMPRO triage) is handled by E016-T03.

- ✅ T01 — autostart self-heal + dev guard + events log (commit `2c95911`)
- ✅ T02 — EventLogger investigation + write reliability
- ✅ T03 — fix WidgetProps test (commit `95305b7`)
- ✅ T04 — precommit tsc gate (commit `2dcf49c`)
- ✅ T05 — `--minimized` autostart (commit `bd7df6`)
- ✅ Epic scaffold (commit `bfddf63`)
- ✅ Hot fix outside epic: registry rewritten to release path, fresh `desk.exe` built

## Version
- Current: `v0.5.0` (tag v0.5.0; STATE previously said 0.3.0 — corrected 2026-09-06)

## Test Totals
- Rust: 420 tests
- TypeScript: 170 tests
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

Reconciliation of the 3-vs-10 count: these 3 are E010's own numbered tasks.
The "Collect from User — blocking launch" checklist in root
[`BACKLOG.md:266-280`](../BACKLOG.md) lists 10 items — the same media work
split finer, plus 7 additional business/infra items (Stripe account,
Plausible account, Google Search Console, OG cover image, real usage stats
export) that are outstanding but are not counted as E010 tasks.
TODO: repoint this link at `.plan/BACKLOG.md` once E016-T02 merges that
section out of the root file.

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
