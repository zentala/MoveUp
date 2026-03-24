# E008 Journal

## Session 2026-03-24 09:00–10:10

- **Goal**: Investigate UX problems reported by user, plan E008 epic
- **Done**:
  - Deep investigation: activity tracking, notification systems, state machine, overlay bar, KPI strip
  - Fixed standing timer regression (b63c050) — E001-T03 had overwritten fix from 4e8d011
  - Fixed popup/overlay ProgressBar: state-aware (standing=gold, sitting=green→red)
  - Added standLimitSecs to full TS data flow
  - Changed default dev mode to live (pnpm tauri:dev = live, not demo)
  - Added KPI badge hover tooltips
  - Created mockup gallery + 9 scenarios + /popup-mockup skill
  - Created E008 PLAN.md (13 tasks) + ORCHESTRATOR.md (4 waves)
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
- **Improvements logged**: 0 (all captured in E008 PLAN.md)
- **Next**: E008-T01 (UX-FLOW.md update), then T02 (Away state fix)

## Session 2026-03-24 10:30–12:00

- **Goal**: Implement all 13 E008 tasks + add versioning rule
- **Done**:
  - Added versioning rule (`.claude/rules/versioning.md`): each epic = version bump
  - Tagged `v0.1.0` (E001) and `v0.2.0` (E008)
  - T01: UX-FLOW.md synced with E001 + target state machine (cd3095b)
  - T02: Away state fix — `!active → Away` regardless of desk height (c305f08)
  - T04: 17 Away flow tests (b17a923)
  - T03: Live current session block in timeline (81d89af, merged)
  - T05: KPI rename + Unicode icons: `↕ Standing`, `⇄ Changes`, `☕ Breaks`, `👁 Screen` (a557be6, merged)
  - T06: Color unification — `colors.ts` + CSS vars matched to Rust `colors.rs` (72f46f0, merged)
  - T11: Process guard — rewired package.json to use `tauri-dev.sh`, removed `cross-env` (4da0ee9, merged)
  - T12: 8 regression tests for standing timer (d16b22e, merged)
  - T07: 2-layer gold visual for standing laps (152d7ce, merged)
  - T09: NotificationService — centralized 6 toasts, wired `notification_backend` config (0408700, merged)
  - T08: Popup redesign — timer first, state dot in header, removed redundant bar (b6fd863, merged)
  - T10: 28 NotificationService tests (86ef5b7, merged)
  - T13: Final UX-FLOW.md verification — fixed 3 discrepancies (7d3332c)
  - Split oversized test file (273→152+113 lines) (52d4106)
- **Decisions**:
  - Versioning: `0.MAJOR.0` per epic, `0.MAJOR.MINOR` for hotfixes
  - Walking grouped with Away in `accumulate_ongoing` (both = not at computer)
  - Notification suppression: early return in `check_notification_conditions` for Away/Walking
  - `cross-env` removed — bash script handles env vars
- **Findings this session**: 1
  - `notification_service_tests.rs` exceeded 250-line limit (273 lines) — split immediately
- **Improvements logged**: 0
- **Test totals**: 339 Rust + 169 TS = 508 tests, all green
- **Next**: E008 complete. Consider E003 or backlog items (SQLite time-series, timeline full window, notification A/B testing)
