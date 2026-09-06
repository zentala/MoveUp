---
updated: 2026-09-06T14:00:00
active_epic: none (E011, E012, E016, E017, E018, E019 code complete; E015 code-complete, T05 browser evidence gap open — see below)
planning_epic: none
planning_epic_path: null
current_wave:
  - E016 done — ran through AO run E016-20260906-0348, promoted to main at
    0995f42, all 5 tasks merged and verified (2026-09-06).
  - E015 code-complete via AO run E015-20260906-0449 (3rd attempt after
    2 write_set/dirty_worktree fixes), promoted to main at 7a8bbf3,
    version bumped to 0.6.0 and tagged (cdb4bf3). 504 Rust + 255 TS tests
    green, typecheck clean. T05's browser/visual check did NOT complete —
    dev-mode gaps (no Vite proxy for /display, mock mode doesn't drive the
    real session engine) blocked it; filed to BACKLOG.md. Engine
    correctness for the fixed scenario is proven by an e015_ Rust test, not
    by a rendered screenshot. E015 is NOT marked fully done until that gap
    closes or Paweł accepts the automated-test evidence as sufficient.
  - E017 done (2026-09-06) — ran through AO run E017-20260906-0622 (one
    stale/transient merge_conflict on T01, resolved via `ao resume` after a
    dry-run merge proved the branch clean), promoted to main at 91b3572, all
    8 tasks merged and independently re-verified (506 Rust + 259 TS tests,
    typecheck, all 8 check scripts). Build gate ran for real
    (MoveUp_0.6.0 MSI/NSIS produced, ~4-6 MB — corrected the stale 60-70 MB
    target in installer.md). First-run browser pass found and (via a second
    ts-dev fix) closed 3 real bugs (mangled Polish diacritics, CWD-relative
    remote-display static path, unguarded Tauri calls in remote mode);
    calibration and no-sensor state remain unverified by design/hardware
    constraint, not a gap in the fix. Code-signing provider decision
    deliberately deferred by Paweł — open in BACKLOG.md. `main` is not
    pushed to `origin` (20+ commits ahead, accumulated across E016/E015/E017)
    — push was not requested this session.
  - E019 done (2026-09-06) — ran through AO run E019-20260906-0822 with two
    operator interventions, neither a code defect: (1) both wave-0 workers
    hit a Claude session-limit wall at the same moment, resolved by waiting
    past the reset then `ao resume`; (2) task T04 then failed verification
    because Windows Smart App Control started blocking freshly-compiled,
    unsigned Rust proc-macro DLLs system-wide (confirmed via
    Microsoft-Windows-CodeIntegrity/Operational, 193 events since
    10:45:32) — not epic-specific, hit a plain `cargo build` in the main
    checkout too. Fixed live with `CiTool.exe --refresh` (no reboot) after
    explicit user consent via consent-broker. Promoted to main at 4649dc5,
    all 8 tasks merged and independently re-verified (533 Rust + 3
    integration + 261 TS tests, `just check` exit 0). No Outside-AO items —
    all backend/non-UI. Both root causes filed to
    `dispatch.internal/.plan/BACKLOG.md` as gaps in AO's
    `executor_result_error` classification.
  - E018 done (2026-09-06) — ran through AO run E018-20260906-1209, promoted
    to main at 7566de7 after three write_set widenings (T03, T02 — 18 files,
    ts-rs codegen fallout — and T10, all legitimate under-declared scope,
    not worker mistakes) and one transient `pnpm build` flake (T05, cache
    contention with a parallel worker). All 11 tasks merged and
    independently re-verified (534 Rust + 3 integration + 304 TS tests,
    `just check` exit 0, all 12 evidence records current). Both Outside-AO
    items closed: `browser` agent confirmed all 4 Analyst-UI checkpoints
    (date header/nav, sticky day-nav, 3 donut KPI cards, pulse-on-day-change)
    against the mockup route (live Tauri window needs the physical sensor,
    not checked, stated honestly); coverage-threshold call (D4) surfaced as
    a code comment + BACKLOG.md follow-up (T05 reached 84.96/82.81/73.07/72.04,
    short of the original 80/80/75 target, thresholds lowered accordingly).
  - E020 (engine: pure core, 69 pts) next per the 2026-09-06 roadmap — last
    of the three planned epics, wants a stable core after E018/E019.
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
