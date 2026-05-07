# HANDOFF 2026-05-07 — E011 autostart hardening

**Linked from:** [.plan/STATE.md](../STATE.md), [.plan/epics/E011/PLAN.md](../epics/E011-2026-05-07-autostart-hardening/PLAN.md)

## Context for next session

User reported autostart didn't work today. Diagnosis surfaced 4 issues. Plan E011 created and approved. Hot fix already applied outside the epic; durable fixes pending.

## What's already done

- Registry `HKCU\…\Run\SmartDesk` rewritten from `target/debug/desk.exe` → `"target/release/desk.exe"`
- Fresh release built at `C:\code\zntl-tray\target\release\desk.exe` (07.05.2026 02:55:58, 13.83 MB)
- Manually verified app runs (overlay rendered, process stable)
- Epic E011 scaffolded with 5 task specs (commit `bfddf63`)
- T03 done — committed `OneBarTimeline.test.tsx` field fix (commit `95305b7`)

## Where to resume

Wave 1, parallel dispatch. Read these in order:
1. `~/.claude/plans/wszystko-jiggly-wall.md` — full plan with code skeletons
2. `.plan/epics/E011-2026-05-07-autostart-hardening/PLAN.md` — what + why
3. `.plan/epics/E011-2026-05-07-autostart-hardening/ORCHESTRATOR.md` — wave order
4. Task specs in `.plan/epics/E011-2026-05-07-autostart-hardening/tasks/`

## Suggested next session entry point

```
> resume E011 — set up worktrees for T01 and T02, dispatch parallel agents
```

Or just `/gsd:resume-work` if using gsd plugin.

## Outstanding observations (don't lose)

1. **events.log missing for 2026-05-07** — both Roaming and Local AppData log dirs have no entry for today, even though the app started multiple times during this session. T02 owns the investigation. Hypotheses listed in `tasks/E011-T02-event-logger-fix.md`.

2. **Popup window appearance during autostart** is uncertain — `tauri.conf.json` sets `"visible": false` on main window but in conversation we observed `MainWindowTitle: 'zntl Overlay'` (overlay shown, popup not visible). Need to confirm whether reboot test actually reproduces the popup flash or if `--minimized` (T05) is preventive only.

3. **`winreg = "0.52"`** is the suggested version for T01; verify it's still current at implementation time (`cargo search winreg`).

4. **Precommit gap:** today's bug (missing `WidgetProps` fields not caught) was specifically because desk app has no `tsc --noEmit` precommit. T04 is the durable fix. Until T04 lands, the same class of bug can recur.

5. **CLAUDE.md was modified at session start** (per git status) by something outside this conversation — `M CLAUDE.md` showed before any work. Not investigated; left as-is. Verify before next session.

## Out of scope reminders

- Code signing the release exe = $200-500/yr cert, separate epic
- `EventLogger` → JSON Lines migration → IMPROVEMENTS at epic close
- Settings UI "Verify autostart" button → BACKLOG
