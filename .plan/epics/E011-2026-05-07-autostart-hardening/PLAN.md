# E011 — Autostart Hardening + Quality Gates

**Status:** active
**Started:** 2026-05-07
**Linked from:** [.plan/BACKLOG.md](../../BACKLOG.md), [.plan/STATE.md](../../STATE.md)

## What

Make the desk app's autostart self-healing, ensure events.log reliability, and add quality gates to prevent the regression class that broke today's release build.

## Why

On 2026-05-07 the app failed to autostart with the system. Investigation surfaced four problems:

1. Autostart registry pointed to a stale debug build (`target/debug/desk.exe`) — registered once during `pnpm tauri:dev`, never refreshed because `ensure_autostart()` only acts when `is_enabled()=false`.
2. Release build refused to compile because `OneBarTimeline.test.tsx` was missing two required `WidgetProps` fields. Precommit didn't catch it — desk app has no `tsc --noEmit` gate (only tray app does).
3. `events.log` is not being written for today even though the app starts. `EventLogger::log()` swallows all errors into `log::warn!()` so we have zero visibility into the failure.
4. Popup window flashes onscreen at autostart instead of starting silent in tray.

Together these mean the user has to manually fix the registry every time they switch between dev and release, build failures hit late (release time, not commit time), and we can't tell whether the app ever ran on a given day. Hardening these surfaces removes a recurring papercut.

## Scope

5 tasks in 3 waves. Worktree per task. Parallel dispatch within a wave.

| Task | Wave | Title |
|------|------|-------|
| E011-T01 | 1 | Autostart self-heal + dev guard + log to events |
| E011-T02 | 1 | EventLogger investigation + write reliability |
| E011-T03 | 1 | Commit pending OneBarTimeline.test.tsx fix |
| E011-T04 | 2 | Precommit gate: tsc --noEmit on desk app |
| E011-T05 | 3 | Autostart `--minimized` + popup hidden on autostart |

## Acceptance criteria

- [ ] After `cargo clean && pnpm tauri:dev` → autostart registry is NOT touched (dev guard works)
- [ ] After installing release → autostart registry points to release exe; if it pointed elsewhere, it's been re-registered (self-heal works)
- [ ] After every app start, `events.log` contains both `START vX.Y.Z` and `AUTOSTART {enabled|verified|stale_path|skipped} ...` lines
- [ ] Integration test proves `EventLogger::log()` actually writes to disk under realistic conditions
- [ ] Committing TS file with type error to `apps/desk/` is blocked by precommit
- [ ] System reboot → desk app starts in tray with no popup flash
- [ ] All existing tests still pass (590+ TS, 420+ Rust)

## Test strategy

**Unit (Rust):**
- `paths_equal()` — case insensitivity, quote trimming, slash normalization
- `--minimized` arg parser — present, absent, mid-list

**Integration (Rust):**
- `EventLogger` writes to a real `tempdir`, file appears, content matches

**Manual smoke (in each task file):**
- Autostart self-heal: deliberately break registry, run exe, verify it heals
- events.log: clear log dir, run exe, verify file appears with `START` line
- Precommit gate: try to commit deliberate type error, expect block
- Minimized startup: run with `--minimized`, expect tray-only

**End-to-end (manual):**
- Reboot system, log in, wait 30s, verify `events.log` has fresh `START` + `AUTOSTART` entries and no popup flashed

## Out of scope

- UI "Verify autostart" button in Settings → BACKLOG
- Code signing the release exe → separate epic (cert procurement)
- Migrating EventLogger to JSON Lines → IMPROVEMENTS
- Changing `app_data_dir` location → no reason to touch

## Files referenced

- [ORCHESTRATOR.md](./ORCHESTRATOR.md) — execution order + waves
- [JOURNAL.md](./JOURNAL.md) — live findings + session summaries
- [IMPROVEMENTS.md](./IMPROVEMENTS.md) — open TODOs surfaced during this epic
- Task specs: [tasks/](./tasks/)
- Investigation: [../../investigations/2026-05-07-events-log-not-written.md](../../investigations/2026-05-07-events-log-not-written.md) (created by T02)
- Original plan file: `~/.claude/plans/wszystko-jiggly-wall.md`
