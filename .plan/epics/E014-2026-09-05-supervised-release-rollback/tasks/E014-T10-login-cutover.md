---
formatVersion: 1
id: E014-T10
title: Transfer login startup to verified PM3 ownership
epic: E014
status: pending
points: 3
importance: High
dependencies: [E014-T08]
---

# Login ownership cutover

Read ../RESUME.md and ADR 018. This is an attended final integration gate, not
an unattended worker permission to restart Windows or replace the live daemon.

Prepare source that stops automatic Run-key rewriting after successful PM3
ownership transfer. Make the existing autostart setting truthful: it must reflect
and control the selected owner, and remain disabled persistently when requested.
Preserve the registry bridge until the PM3 login path has actual evidence.

Before a live change, show the exact reviewed MoveUp and PM3 revisions, installed
paths, current startup configuration, passed checks, known release holds and
recovery procedure. Resolve PM3's owned release gates through its owner. Arrange
the disruptive logoff/login with the operator only when the result is reviewable.

Acceptance: actual Windows login starts the intended installed version with one
process and a usable tray; intentional Quit stays stopped for that login and
starts again at next login; the direct Run value is removed only after predecessor
proof and is not recreated on manual launch. Verify the autostart off choice.
Retain registry/task/runtime evidence. Never mark this complete from source tests
or Scheduled Task presence alone. Unperformed login remains pending.

The login evidence must identify PM3 as the launching owner. One process/tray
while the old Run entry can also launch the app is insufficient: singleton or
stand-down could hide which startup path actually worked. Preserve an exact
backup and a tested rollback procedure for the bridge; prepare a recoverable
temporary disable for the operator-arranged ownership test, and delete the Run
value only after PM3-only login proof. Do not perform this disable during T08 or
while merely preparing the cutover. Confirm supervision readiness, actual
installed artifact and recorded process/run identity after the test login.

## Existing per-service boot contract

The PM3 owner inspected source `9608643`: the app-owned configuration's `boot`
flag is read during daemon boot. A false flag prevents restore at the next login,
even for a previously running service; same-login adoption of an existing child
is separate. Never call `unstartup`, which removes shared PM3 startup.

The existing public apply flow is `reload <project>` from the explicitly selected
application configuration directory. Re-enabling a changed service additionally
requires explicit `start` to establish desired-service bookkeeping; reload alone
does not do that. PM3 source 5be48d7, independently accepted in final round 3,
now records eligible desired intent after a successful supervised start,
including an already-live owned child; amendment 5 defines this transition.
Consumer acceptance must still exercise the actual isolated daemon path.
Preserve login suspension holds and never use `resume --all`
merely to toggle the preference. Test OFF -> ON, a service first started with
boot false, and an existing Quit hold through the actual isolated daemon path.

There is currently no machine-readable effective-boot field in PM3 status or
describe, and reload prints text. Read the preference from the app-owned file;
do not present that as observed daemon convergence. Bound launcher calls, report
apply failures truthfully and preserve recoverable configuration on failure.
Resolve any remaining unsupported transition before claiming the checkbox works;
do not silently add a private PM3 source import or a competing startup owner.

## PM3 owner deployment prerequisite — 2026-10-09

Source main/origin at 5be48d7 is accepted. The owner's final dependency handoff
in pm3-mcp's private plan, E050/reports/MOVEUP-PUBLIC-LAUNCHER-HANDOFF.md,
separately retains live holds: E047 restart ownership, E048 GitNexus admission,
E042 released-daemon cutover, E044 owner/Job control and E045-T04 attended domain
acceptance. E045-T06 is complete and must not be repeated as an open hold.
Installed pointer remains e138cabb3ed6. Re-read current owner evidence before
cutover; a successful isolated fixture or ordinary source push does not clear
these holds. Complete independent MoveUp source/fixture work first and report
the exact remaining deployment dependency without replacing the live daemon.
