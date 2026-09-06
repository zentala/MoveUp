# E011 — Journal

**Linked from:** [PLAN.md](./PLAN.md)

## Session 2026-05-07 03:50 — epic-bootstrap

- **Goal:** scaffold E011 + commit pending test fix (T03) + dispatch wave 1
- **Trigger:** user reported that autostart didn't work today; diagnosis surfaced 4 underlying issues
- **Pre-plan diagnostics done in conversation (not committed work, just findings):**
  - Registry `Run\SmartDesk` was pointing at `C:\code\zntl-tray\target\debug\desk.exe` (stale dev path). Manually rewrote to release path during diagnosis.
  - Release build failed twice on missing `WidgetProps` fields in `OneBarTimeline.test.tsx`. Edited file in place (added `idleSecs: 0`, `continuousComputerSecs: 0`) — not yet committed.
  - Built fresh release: `desk.exe` 13.83 MB, 2026-05-07 02:55:58.
  - Verified app runs (overlay rendered, process stable) but `events.log` was never written for today in either Roaming or Local AppData. Root cause unknown — silent error swallow in `EventLogger::log()`.
- **Decisions:** unified all 4 issues + the precommit gap into single epic E011 instead of E000-maintenance scatter. User confirmed.
- **Next:** commit T03, create worktrees for T01/T02, dispatch parallel.

## Session 2026-05-07 03:55 — handoff

- **Outcome:** epic scaffolded, T03 done (`95305b7`), hot fix applied (registry + release build)
- **Decision:** stop here for the night — wave 1 dispatch deferred to next session
- **Handoff:** [`../../handoffs/HANDOFF-2026-05-07-e011-autostart.md`](../../handoffs/HANDOFF-2026-05-07-e011-autostart.md)
- **Next:** worktrees for T01+T02, parallel agents, then T04, T05

## Findings

(append `## Finding YYYY-MM-DD HH:MM — <slug>` entries during work)

## Session 2026-09-06 (auto — session ended without done.)
- **Note**: Session ended without `done.` command. No journal was written.
- **State at exit**: see STATE.md for last known state
- **Action needed**: next session should review what happened and write proper journal
