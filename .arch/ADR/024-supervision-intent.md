# ADR 024: Confirm an owned PM3 hold before Quit or development replacement

- Status: accepted implementation of the E014-T06 handoff
- Date: 2026-10-09
- Dependency: PM3 supervised application v1, amendments 1–5, frozen source
  `5be48d773f1138301718858a965b8f4b038aa578`

## Decision

Use the public `PM3_HOME/releases/launch.mjs` through an explicit absolute Node
executable, structured arguments, inherited environment and hidden helper windows.
Never import PM3 implementation modules. The adapter keeps configuration,
response classification and bounded process transport separate from Tauri.

`DESK_SUPERVISION=pm3` opts in; a supplied `PM3_SERVICE` or
`DESK_PM3_SERVICE` also selects PM3. `DESK_SUPERVISION=bridge` explicitly selects
ordinary Quit (including dev children). PM3 requires a qualified service name
and an absolute `DESK_PM3_NODE` (or `PM3_DAEMON_NODE`). Home is
`DESK_PM3_HOME`, then `PM3_HOME`, then `%USERPROFILE%/.pm3`. Partial configured
mode fails closed. Launcher and executable must exist before invoking them.

Quit uses one stable UUID per process and an in-flight guard. Within eight
seconds it requests a login-scoped hold, reconciles lost acknowledgement through
`supervision SERVICE --json`, and retries the same intent. Only a structurally
valid acknowledgement or a known-mode matching hold permits the existing
lifecycle/flush/exit path. Unknown inner mode never authorizes exit. Failure
keeps the application running and uses its existing notification mechanism.

Development gets a fresh intent per invocation. Suspension precedes any process
replacement, irrespective of the display port. Finally releases only that
intent, including when acknowledgement was lost; never uses `--all`.
Only a single recognized executable with matching PID, path and start time may
be stopped. Ambiguous or unknown instances are refused. In bridge mode the
exact release replaced is restored; under PM3 only PM3 resumes recovery, without
an additional launcher recovery loop. Nested or pre-existing holds survive.

## Validation and limits

Injected transports and synthetic process operations cover acknowledgement,
unknown outcomes, ownership, alternate ports, startup failure and cleanup.
No live process, production pipe, daemon, registry or ergonomic state is changed.
Real stand-down over two poll cycles and login ownership remain runtime gates.
