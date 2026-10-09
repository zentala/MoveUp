# ADR 024: Application lifecycle diagnostics and explicit readiness

- **Status**: accepted
- **Date**: 2026-10-09
- **Related**: [ADR 015](015-pure-ergo-engine.md), [ADR 018](018-pm3-app-ownership-split.md),
  [ADR 019](019-release-store-layout.md)

## Decision

The application owns lifecycle evidence and a typed readiness contract outside
the ergonomic engine. PM3 supervision, restart policy, Quit suspension, release
selection and autostart cutover remain separate tasks.

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
Missing/corrupt/unreadable previous evidence or any outcome without
`clean_exit` is `interrupted_unknown`; it never identifies a crash cause.
An absent record is `previous_absent`. Quit intent alone is not a clean exit.

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

`GET /display/readiness` is a separate axum route on the existing LAN server.
`/display/api`, its aggregate, and the public legacy health probe remain unchanged.
The LAN enable switch, bind address and port behavior are preserved. Disabled LAN
or a failed bind means this endpoint is unavailable, not that another process
may be inferred ready. Listener state is set only after successful bind and
cleared when serving ends.

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
DB and sensor connection. DB contention conservatively reports not-ready; sensor
absence or contention yields ready+degraded, reason `sensor_missing`, once
essential prerequisites hold. It never calls the session aggregate, takes its
locks or calls an outbound provider. Fixed schema and bounded identity keep
responses small. HTTP availability never sets the tray flag.

States are lowercase `initializing`, `ready`, `degraded`, `failed`.
Failure is sticky across recomputation. Missing sensor is restart-ineligible.
Quit clears readiness before `app.exit(0)`.

`health_probe::validate_readiness` deserializes the typed schema, caps input at
4096 bytes, checks app/version and optional expected PID/run/supervisor IDs,
requires ready/listening and consistent essential flags, state and reason.
Arbitrary JSON and stale or contradictory flags cannot pass. Existing
`probe`, `HttpDisplayApi` and `/display/api` callers keep their old behavior;
new supervision adapters must explicitly call the typed validator.

## Verification boundary

Sibling library tests exercise lifecycle correlation, atomic replacement
failures, previous interrupted/corrupt records, secondary ownership, sticky
failure, truthful flush results and bounded readiness under contended locks.
The integration harness starts actual child processes for pre-setup failure,
panic before/after ownership, secondary activation, forced termination and clean
exit. Every fixture uses temporary files and in-memory databases. The existing
EventLogger implementation is exercised without modification.

No full GUI run against production data, PM3 restart or real login acceptance
is implied by these fixtures. Those remain attended integration gates.
