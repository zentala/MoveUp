# E002 — UX Integrity: Fix What the User Sees

## Problem

The app is **confusing for its own developer**. Multiple UI elements show wrong data,
misleading labels, or nothing at all. The state machine has a fundamental gap (Away
is unreachable). Two notification systems fire independently. The user cannot understand
what UI elements mean without asking the AI that built them.

This is not a feature epic — it's a **trust and correctness** epic. Every fix here
makes the app do what it already claims to do.

## Root Causes (discovered 2026-03-24)

1. **Away state unreachable** — `session_reading.rs` returns `Sitting` when desk is low,
   even if user left the room. `Away` exists in enum but only as initial state. Causes:
   sitting timer counts while user is gone, alerts fire at empty desk, screen time KPI
   never resets, timeline never shows breaks.

2. **Two notification systems, zero coordination** — native toasts (6 conditions scattered
   across `serial_periodic.rs`) + custom WinAPI popup (alert escalation). No central router.
   `notification_backend` config field is dead code.

3. **Popup ignores Standing state** — OneBarTimer always showed sitting metrics. Fixed in
   this session but exposed: no tests prevented the regression, overlay and popup use
   different data sources and color palettes.

4. **UI labels don't explain themselves** — KPI "Session" means "longest continuous screen
   time" but reads as "sitting session". "Standing" shows dash with no explanation. User
   must ask the developer tool what its own UI means.

## What was already fixed (2026-03-24, this session)

- [x] Standing timer in popup: shows `breakSecs / standLimitSecs` when Standing
- [x] Popup ProgressBar: gold bar for Standing, sitting bar for Sitting
- [x] Overlay ProgressBar in App.tsx: same logic
- [x] `standLimitSecs` flows: DTO → useDesk → useWidgetData → WidgetProps → widgets
- [x] Default dev mode: `pnpm tauri:dev` = live (not demo)
- [x] KPI badge tooltips: hover explains each metric + thresholds
- [x] Rule: `.claude/rules/ux-flow-sync.md` — UX-FLOW.md must stay current
- [x] Test: standing big number uses `breakSecs`

## What still needs fixing

### A. State Machine (correctness — blocks everything)

| # | Issue | Impact |
|---|-------|--------|
| A1 | Away unreachable: desk low + inactive = still Sitting | Sitting timer runs while user is gone, false alerts |
| A2 | Walking misnomer: we don't know user walks | Misleading state name in UI and code |
| A3 | Away sessions not saved to DB | Timeline has no Away records, totals wrong |
| A4 | `away_bout_secs` is dead code | 5-min position change logic never fires |
| A5 | `continuous_computer_secs` never resets | Screen time KPI always red after 75 min |
| A6 | No tests for: low+inactive, Away accumulation, screen time reset | Can break again silently |
| A7 | Notifications fire during Away: "time to stand" while user is gone | App nags empty desk |
| A8 | Away should start a NEW session, not extend the old one | Returning = fresh start, not "you were sitting 49 min" |

### B. UI Sync & Clarity (user-facing)

| # | Issue | Impact |
|---|-------|--------|
| B1 | KPI "Session" → rename to "Screen time" + eye icon 👁 | User thinks it's sitting session or app uptime |
| B2 | KPI "Standing" shows dash for 30 min | User sees useless badge for half the morning |
| B3 | Overlay/popup use different color palettes | Green/yellow/red thresholds differ between bars |
| B4 | Standing lap: no 2-layer gold visual | Bar resets to 0 on lap, no visual of completed lap |
| B5 | Timeline doesn't show Away (gray blocks) | User can't see when they were away from desk |
| B6 | Self-descriptive UI audit needed | Elements lack explanation, colors unexplained |
| B7 | Timeline only shows completed sessions, not live | User doesn't see current state in timeline until next transition |
| B8 | KPI strip has no section header / time range label | User doesn't know stats = "today" or "24h" or what |
| B9 | KPI badges need icons: eye (screen), arrows (changes), clock (breaks), person (standing) | Text-only badges are hard to scan |
| B10 | No time range selector for stats (today / 7d / 30d) | Can't see trends, only today |
| B11 | Header "desk 72cm" and state label "sitting" are separate, redundant | Two lines say related things apart; height too prominent, state buried lower |
| B12 | Two progress bars in popup (overlay + inline) — redundant, chaotic | User sees two bars showing the same thing |
| B13 | Visual hierarchy wrong: timeline above timer, timer buried at bottom | Most important info (timer) should be at top, least important (timeline) at bottom |
| B14 | State colors not consistent: standing = gold bar but label is white | Standing label, dot, bar, timeline, background should ALL be gold |

### C. Notification System (architectural)

| # | Issue | Impact |
|---|-------|--------|
| C1 | 6 toast conditions scattered, no central routing | Can't switch backend, can't A/B test |
| C2 | `notification_backend` config is dead code | Setting exists but does nothing |
| C3 | Custom popup looks like Win32 2003 dialog | Ugly, inconsistent with app's dark theme |

### D. Developer Experience

| # | Issue | Impact |
|---|-------|--------|
| D1 | Process guard lost (bash→cross-env migration) | Can't auto-kill old desk.exe before rebuild |
| D2 | UX-FLOW.md outdated (pre-E001) | Docs say one thing, code does another |
| D3 | No regression tests for popup standing timer | Same bug can return in next refactor |

## Scope

**In scope:** A1-A6, B1-B6, C1-C2, D1-D3
**Deferred:** C3 (popup redesign to WebviewWindow), cross-platform activity detection

## Acceptance Criteria

1. `inactive ≥ 60s` → Away state, regardless of desk height
2. Away sessions in DB, visible as gray blocks in timeline
3. Screen time KPI resets after 5 min Away (verifiable with test)
4. Every KPI badge: tooltip on hover explaining what it measures
5. Popup and overlay show the same metric per state (same progress, same color semantics)
6. All notification conditions route through one service
7. UX-FLOW.md matches running code
8. Developer can `pnpm tauri:dev` and see live data, `pnpm tauri:dev:demo` for demo
9. Process guard works (auto-kill old instance)
10. 15+ new tests covering Away flow, standing timer, screen time reset
