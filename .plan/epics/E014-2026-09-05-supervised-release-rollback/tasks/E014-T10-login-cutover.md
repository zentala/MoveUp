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
