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

Read all five PM3 contract amendments. Status readback confirms suspension
only when the named service has a supervision.suspension.holds entry matching
this request's intentId. Missing service or supervision, incomplete/malformed
JSON and transport errors are UNKNOWN, never proof that no hold exists. Retry
the same intent or read status within a bounded deadline; never Quit on UNKNOWN.
Use structured process arguments with no shell interpolation and hide helper
console windows. Do not import PM3 internal source modules into MoveUp.

The frozen consumer CLI uses an explicit absolute Node executable and the stable
PM3_HOME/releases/launch.mjs launcher. Read back with `supervision <service>
--json`, not the general status command: this dedicated command never starts a
daemon or registers a project. It returns exit 0 with `{state:"known", name,
supervision}` or exit 3 with `{state:"unknown", name, reason, detail}`. Check the
service name and matching hold; an exit code alone is insufficient. Suspend
success has no `ok` field; require its acknowledged service, intent and scope.
Tolerate additional status fields, but an explicitly unknown supervision mode
cannot authorize Quit even if a previously stored matching hold is visible:
PM3 may be unable to establish whether that hold applies to the current login.
The accepted contract is frozen at PM3 source
5be48d773f1138301718858a965b8f4b038aa578 (independent final round 3 ACCEPT).
T05 public-launcher interoperability passed against that exact release.
For isolated Windows fixtures, inherit every supplied fixture environment key,
including PM3_TEST_IPC_PATH, PM3_TEST_RUN_DIR and PM3_TEST_PIDFILE_PATH;
PM3_HOME alone does not isolate the production named pipe. Do not start a daemon
or use the production pipe during this task.

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

## Implementation handoff

Dispatch begins only after T05 is integrated and the PM3 owner supplies accepted
source evidence. The coordinator supplies read-only PM3_SOURCE and
PM3_SUPERVISION_CONTRACT references at runtime; never publish their private paths.
Keep the process invocation and response classification independently testable,
separate from tray/Tauri glue. Define explicit supervision configuration; partial
or invalid configured mode cannot silently become an ordinary unsupervised Quit.
Use the accepted public launcher with the configured PM3_HOME in its child
environment. Bound elapsed time and output size, and keep UI event handling
responsive. Duplicate Quit clicks must not create an unbounded set of holds.

Preserve the existing flush path after acknowledgement. On failure the app stays
running and gives a visible, actionable error through existing UI/notification
mechanisms; do not introduce a new visual design or falsely record a clean exit.
Record the adapter decision in ADR024 before implementation and update PROJECT.xml.

The development launcher must not kill all processes sharing the executable name.
Inspect and preserve process identity, stop only the identified instance this
invocation is replacing, and keep release restoration and hold ownership inside
the same try/finally scope. A failed suspend must precede and prevent replacement.
Tests use injected operations and synthetic processes only, including alternate
ports, nested holds, failed dev startup and lost acknowledgement. Provide a
self-contained `scripts/test-tauri-dev.ps1` runner without installing a new test
framework; it must return a nonzero exit for a failed assertion.

Run focused Rust adapter tests, the complete Rust library suite and the developer
launcher runner. Do not run the real tauri-dev launcher or alter the installed
application during verification. Keep logs in the worktree's ignored target
directory; report exact counts/exits, a frozen commit and any pending runtime gates.
