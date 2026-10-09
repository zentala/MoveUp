# ADR 024: Application lifecycle diagnostics and explicit readiness

- **Status**: accepted
- **Date**: 2026-10-09
- **Related**: [ADR 015](015-pure-ergo-engine.md), [ADR 018](018-pm3-app-ownership-split.md),
  [ADR 019](019-release-store-layout.md)

## Decision

The application owns lifecycle evidence and a typed readiness contract outside
the ergonomic engine. PM3 supervision, restart policy, Quit suspension, release
selection and autostart cutover remain separate tasks. The dated intent amendment
below records the subsequent Quit and development adapter decision.

### Runtime ownership and events

`lib::run` installs diagnostics before dotenv, plugins or application state.
The single-instance plugin initializes first. Application setup then claims
`{app_data_dir}/lifecycle.owner` with an exclusive OS file lock, retained for
the process lifetime. This second guard handles activation races in the existing
plugin (a mutex can exist before its message window). A competing claimant
cannot read or replace the owner's run record. Normal secondary activation only
focuses the existing window and logs `secondary_activation` in the owner.

Only after ownership, `app_lifecycle_store` reads the previous record and writes
`lifecycle.json` through a sibling temporary file, sync and atomic rename.
The record contains a UUID run ID, numeric PID, version, debug/release mode,
optional bounded `PM3_RUN_ID`, outcome, Quit intent and flush result.
An absent record is `previous_absent`. A recorded `clean_exit` is
`previous_clean_exit`; an explicitly recorded `failed` outcome is
`previous_failed`. Corrupt/unreadable evidence and all other outcomes are
`previous_interrupted_unknown`; they never identify a crash cause.
Quit intent alone is not a clean exit.

`AppLifecycle` uses the existing `EventLogger::log` and daily
`HH:MM:SS LIFECYCLE run=... pid=... version=... build_mode=... stage=...` format.
It records started, previous outcome, tray/persistence readiness, ready,
secondary activation, explicit tray Quit, failures and actual Tauri Exit.
Essential DB initialization reuses `commands::ensure_initialized` during
setup, after configuration restoration, rather than waiting for frontend IPC.
The tray flag is set only after successful native tray construction.

Tauri Exit retains the existing persisted-engine-state save and in-progress
session insertion. Each leg reports saved, failed, not_needed, or a specific
skipped reason (missing store/DB or unavailable lock). A clean process exit
does not imply that either flush succeeded. Observed failure remains failed.
Neither the session engine nor its persistence format changes.

### Earliest failure and panic sink

A per-invocation `moveup-startup-<pid>-<uuid>.log` in the OS temporary directory
is independent of application paths and locks. Each write opens in append mode;
there is no truncating create, shared writer mutex, logger call, panic payload,
backtrace, environment dump or arbitrary error text in the panic path.
Only generated identity and bounded event codes are written. All IO is best effort;
failed diagnostics do not crash an otherwise working app.

The hook marks a sticky atomic failure, records panic and calls the previous
hook. It does not catch-and-continue, suppress unwind/abort, or claim to observe
external termination. It never tries to update the shared run file while
application/persistence locks might be held. A killed or panicking run without
Exit is therefore interrupted/unknown on next launch; the independent panic log
is additional evidence, not a guessed exit cause. OS temporary-file retention
applies to the early sink; normal EventLogger retention is unchanged.

### Dedicated readiness endpoint

`GET /display/readiness` uses a dedicated axum listener in the same application
process, bound only to IPv4 loopback. Its default port is 3391, configurable by
`DESK_READINESS_PORT` (integer 1–65535). Invalid values and bind failures are
explicit failures: there is no fallback port. Failure is logged and prevents
readiness while the desktop remains available for diagnosis.

Start the listener independently of `remote_lan_enabled` and `DESK_REMOTE_PORT`.
Remove readiness from the LAN router so process/run metadata is not exposed
there. Preserve `/display/api`, WebSocket, health and voice routes and the LAN
setting semantics. Optional LAN failure and sensor absence are not essential
readiness failures. This adds no process, private watchdog or engine change.

The 2026-10-09 supervision amendment replaces the earlier shared-LAN listener
design and ADR 018/019's display endpoint as supervision evidence: an optional
display cannot reliably prove that the essential application is running.

Example response (all fields are flat):

```json
{
  "app_id": "io.zntl.desk",
  "version": "0.6.0",
  "pid": 1234,
  "run_id": "81c9c6f0-5b23-482f-902d-a8c645818e7b",
  "supervisor_run_id": "pm3-example",
  "build_mode": "release",
  "ready": true,
  "state": "degraded",
  "reason": "sensor_missing",
  "tray_initialized": true,
  "persistence_initialized": true,
  "setup_complete": true,
  "remote_listening": true,
  "sensor_present": false,
  "session_store_available": false,
  "quit_requested": false
}
```

Readiness requires successful tray construction, essential SQLite initialization
and completed setup, with neither failure nor Quit intent. `persistence_initialized`
means the existing SQLite open/schema path succeeded and the connection remains
present, not that every future write is guaranteed to succeed.
The optional session-store flag reports actual availability; this change neither
invents a new store path nor claims that the existing optional store was loaded.

The reader uses atomic lifecycle evidence and nonblocking `try_lock` on the live
DB and sensor connection. DB contention preserves the last known initialization
flag without waiting; it cannot promote an uninitialized or failed app to ready.
An acquired DB lock with no connection, or a poisoned DB mutex, clears the
response's persistence flag and readiness. Sensor absence or contention yields
ready+degraded, reason `sensor_missing`, once
essential prerequisites hold. It never calls the session aggregate, takes its
locks or calls an outbound provider. Fixed schema and bounded identity keep
responses small. HTTP availability never sets the tray flag.

States are lowercase `initializing`, `ready`, `degraded`, `failed`.
Failure is sticky across recomputation. Missing sensor is restart-ineligible.
Quit clears readiness before `app.exit(0)`.

`health_probe::validate_readiness` deserializes the typed schema, caps input at
4096 bytes, checks app/version and optional expected PID/run/supervisor IDs,
requires ready and consistent essential flags, state and reason. The optional
`remote_listening` field remains diagnostic and is not a readiness prerequisite.
Arbitrary JSON and stale or contradictory flags cannot pass. Existing
`probe`, `HttpDisplayApi` and `/display/api` callers keep their old behavior;
new supervision adapters must explicitly call the typed validator.

### PM3 preparation boundary

Generate an explicit opt-in configuration with absolute installed paths,
structured candidate argv, the same readiness port in child environment and
probe URL, two misses spaced 60 seconds apart, and stand-down enabled.
Service health requires exactly `app_id: io.zntl.desk` and `ready: true`.
Candidate health additionally pins `version`. Owned-child evidence binds
`pid` and `supervisor_run_id`; the separate `run_id` is an application diagnostic
UUID. A healthy foreign dev process without PM3 identity permits stand-down,
never owned-child promotion.

The 2026-10-09 generator correction removes the custom PM3 source/VM loader.
The application owns the candidate JSON projection: preparation takes its absolute
path unchanged, never regenerates or copies candidates, and points the config at
that same path so later application publications remain visible.

Preparation writes only a draft config with `boot: false`. It flushes a private
temporary file in the output directory and publishes it with an exclusive hard
link, giving atomic visibility without overwriting a racing writer. Filesystems
without hard-link support fail explicitly; only invocation-owned temporary
artifacts are cleaned, and no partial destination is left on a pre-publication
failure. No unsafe rename/write fallback is used.

Optional offline validation uses an explicit absolute Node executable and the
stable `<PM3_HOME>/releases/launch.mjs` entrypoint with structured
`candidates validate FILE --config FILE --service desk --json` arguments.
It has no shell, uses a hidden window and bounded time/output, and never imports
PM3 internals. Missing selection yields `pending-not-validated`; invalid,
unsupported or failed validation prevents publication. Even a valid offline
report is not owner acceptance: the PM3 owner must certify supervision, instance
binding and independent suspension holds before integration. Installation,
registration, promotion callback and login cutover remain separate work; the
Run bridge stays intact.
See [preparation and evidence](../../docs/PM3_SUPERVISION.md).

## Verification boundary

Sibling library tests exercise lifecycle correlation, atomic replacement
failures, previous failed/interrupted/corrupt records, secondary ownership, sticky
failure, truthful flush results and bounded readiness under contended locks.
The integration harness starts actual child processes for pre-setup failure,
panic before/after ownership, secondary activation, forced termination and clean
exit. Every fixture uses temporary files and in-memory databases. The existing
EventLogger implementation is exercised without modification.

No full GUI run against production data, PM3 restart or real login acceptance
is implied by these fixtures. Those remain attended integration gates.

## Amendment — 2026-10-09: Confirm an owned PM3 hold before Quit or development replacement

- Status: accepted implementation of the E014-T06 handoff
- Dependency: PM3 supervised application v1, amendments 1–5, frozen source
  `5be48d773f1138301718858a965b8f4b038aa578`

### Intent adapter decision

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

### Intent validation and limits

Injected transports and synthetic process operations cover acknowledgement,
unknown outcomes, ownership, alternate ports, startup failure and cleanup.
No live process, production pipe, daemon, registry or ergonomic state is changed.
Real stand-down over two poll cycles and login ownership remain runtime gates.
