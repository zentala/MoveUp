# ADR 018: PM3 owns supervision, the app owns what "good" means

- **Status**: accepted
- **Date**: 2026-09-06
- **Epic**: E014 (task E014-T09, decisions D1, D2, D4, D7)

## Context

Two failures have no answer today, and both are about a process nobody watches.

`scripts/tauri-dev.ps1` kills the installed `desk.exe` before every dev build,
because Windows cannot overwrite a running executable. Nothing brings it back.
A dev session ends and the app is gone until the next logon.

Separately, a build that installs but never serves — a broken frontend bundle,
a panicking command handler, a wrong `DESK_REMOTE_DIST` — has no fallback. The
only recovery is to notice, find an older installer, and reinstall by hand.

The pieces that would answer this are split across two owners, and the split is
not obvious. A supervision loop (poll, restart, back off, demote after repeated
failure) is generic; it works the same for MoveUp and for anything else on this
machine. Deciding *which builds exist* and *what counts as a healthy one* is app
knowledge that no process manager can guess.

There is also a live constraint. The dev build and the installed build share the
bundle id `io.zntl.desk`, therefore share `tauri-plugin-single-instance`
(`src-tauri/src/lib.rs:136`). A supervisor that matches on executable path would
respawn the installed build on every poll during a dev session, and every spawn
would exit immediately against the singleton lock.

## Decision

**PM3 owns the supervision loop. MoveUp owns the release store and the
definition of health.** Four rules follow.

**Ownership (D1).** PM3 polls, restarts, applies the two-miss rule, stands down,
demotes and promotes. MoveUp keeps the release store, writes the
`last-known-good` marker, emits the ordered candidate list, exposes the health
contract, and honours the `--minimized` launch flag. PM3 consumes; it never
decides which build is good.

**One daemonizer, not two (D2, revised 2026-09-05 by Paweł).** The first draft of
E014's plan kept `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\MoveUp` as
the login path alongside PM3. That is reversed: the app must not register itself
in the registry at all. `ensure_autostart`
(`src-tauri/src/setup_helpers.rs:104`) stops enabling autostart and the `Run`
value is deleted — but **only in the same task that verifies PM3 starts at
logon**, never before. PM3's daemon currently has no ONLOGON Scheduled Task on
mATX, contrary to its own ADR 016; it is revived by the `pm3-doctor` watchdog
every five minutes. Removing the key first would mean the app starts up to five
minutes late after every login. The key is a temporary bridge with a named exit.

**Occupancy is detected by the health check answering, not by a Windows API
(D4).** `setup_remote_display` runs unconditionally at
`src-tauri/src/setup_helpers.rs:67`, outside any `debug_assertions` gate, so the
dev build and the installed build both serve `DESK_REMOTE_PORT` (default 3390)
and both answer `GET /display/api`. Whoever holds the singleton is by definition
the process answering that port. PM3 therefore treats a passing health check as
"the service is up" even when the answering process is not the child it spawned.
No `FindWindowW`, no `OpenMutexW`, no new Windows interop.

**MoveUp's half ships first (D7).** Waves 1-2 (release store, marker, probe,
candidate list) are independently useful and block nothing. Waves 3-4 wait on
PM3's candidate-list feature, tracked in that repo's backlog.

## Alternatives

**A private PowerShell watchdog plus a Scheduled Task.** Smaller, ships without
touching another repo, depends on nobody's roadmap. Rejected: it puts a second
process supervisor on a machine whose convention is that PM3 owns long-lived
processes, and its rollback logic would be private to this one app — every
future app would rewrite the same watchdog. Paweł settled this on 2026-09-05:
PM3 is the only daemonizer.

**Detect the singleton holder through the named mutex.** On Windows the plugin
holds a named mutex and a message-only window whose class is `<bundle id>-sic`
(`tauri-plugin-single-instance-2.4.3/src/platform_impl/windows.rs:65-105`), so
PM3 *could* identify the holder directly. Rejected as the default: it is
Windows-only interop that buys nothing the health check does not already give.
It stays as the named fallback if the port signal turns out to be unreliable.

**In-app self-supervision.** Rejected outright, not as a trade-off. A process
that has died cannot restart itself, and a build that fails to serve cannot
judge itself from the inside. The judge must be a separate, still-alive process.

## Consequences

- Every future app supervised by PM3 on this machine inherits candidate
  demotion and rollback, rather than reimplementing it. That is the reason the
  more expensive option won.
- E014 now depends on another repo's roadmap. Waves 3-4 stay visibly blocked in
  `HANDOFF.md` instead of quietly stalling, and two items in
  `int://mATX.lan/C:/code/pm3-mcp/.plan/BACKLOG.md` are the gate.
- The `Run` key survives longer than the decision that removes it. Anyone
  reading D2 and finding the key still present is looking at the bridge, not at
  a missed task; E014-T10 removes it and is blocked on PM3's ONLOGON task.
- The D4 signal has an assumption with a known break: a dev build started with a
  non-default `DESK_REMOTE_PORT` answers on a port PM3 is not probing, so PM3
  would see the service as down and spawn against the singleton. E014-T06
  records whether that is routine; if it is, the mutex probe above is the
  fallback.
- The health contract (`GET /display/api` on `DESK_REMOTE_PORT`) is now a public
  interface, not an implementation detail. Changing its path, port default or
  response shape breaks supervision, and there is no compiler error for it.

## References

- [E014 PLAN.md](../../.plan/epics/E014-2026-09-05-supervised-release-rollback/PLAN.md)
  §Decisions and ADRs (D1, D2, D4, D7)
- [ADR 019 — release store layout and retention](019-release-store-layout.md) —
  the app's side of this boundary
- [ADR 001 — remote display web kiosk](001-remote-display-web-kiosk.md) — where
  `/display/api` and `DESK_REMOTE_PORT` come from
- `src-tauri/src/lib.rs:136` — `tauri-plugin-single-instance` wiring
- `src-tauri/src/setup_helpers.rs:67,104` — `setup_remote_display`,
  `ensure_autostart`
- `int://mATX.lan/C:/code/pm3-mcp/.plan/BACKLOG.md` — the PM3 items Waves 3-4
  wait on
