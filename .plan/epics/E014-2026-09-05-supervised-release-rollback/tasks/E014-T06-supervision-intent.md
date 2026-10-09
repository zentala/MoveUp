---
formatVersion: 1
id: E014-T06
title: Respect Quit and development sessions under supervision
epic: E014
status: pending
points: 3
importance: High
dependencies: [E014-T05, E014-T11]
---

# Explicit stop intent and development coexistence

Read ../RESUME.md, ADR 018 and the accepted PM3 suspend/resume contract first.
Pair this task with T05 where one worker needs the same adapter context; preserve
separate acceptance and the existing epic total.

Before a supervised application's intentional Quit, obtain a durable PM3 suspend
acknowledgement, then run the existing flush/exit path. Bound the operation; do
not block the UI thread. A missing/lost acknowledgement is an unknown outcome,
not proof of no suspension: reconcile through status. An unsupported or failed
suspend must produce a visible actionable error and must not claim successful
session suspension. Unsupervised bridge mode retains ordinary Quit behavior.

Read all three PM3 contract amendments. Status readback confirms suspension
only when the named service has a supervision.suspension.holds entry matching
this request's intentId. Missing service or supervision, incomplete/malformed
JSON and transport errors are UNKNOWN, never proof that no hold exists. Retry
the same intent or read status within a bounded deadline; never Quit on UNKNOWN.
Use structured process arguments with no shell interpolation and hide helper
console windows. Do not import PM3 internal source modules into MoveUp.

The PowerShell development launcher must explicitly suspend supervision before
replacing an installed process, including when DESK_REMOTE_PORT is changed. Use
try/finally to restore only the supervisor intent this invocation changed. Never
resume a pre-existing user suspension. Handle nested launchers conservatively;
do not infer ownership from a reason string alone. Preserve the existing bridge
relaunch behavior before PM3 cutover and do not introduce a second recovery loop.

Acceptance: isolated tests for acknowledged Quit, timeout/lost acknowledgement,
previous suspension, dev launch failure, finally cleanup, alternate port, and
ordinary unsupervised use. Real stand-down over two 60-second poll cycles belongs
to T08. No user process is stopped while developing or unit-testing this task.

Allowed scope: new small supervision adapter and tests, tray/lib glue,
scripts/tauri-dev.ps1 and its tests, documentation. No ergonomic engine, production
data, live daemon or registry changes. Read DESIGN.md before any visual change.
