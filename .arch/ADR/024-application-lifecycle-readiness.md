# ADR 024: Application lifecycle diagnostics and explicit readiness

- **Status**: accepted
- **Date**: 2026-10-09
- **Epic**: E014 (supervised release and rollback) — task T11
- **Related**: [ADR 015](015-pure-ergo-engine.md) (engine purity and ownership),
  [ADR 018](018-supervised-startup.md) (startup recovery),
  [ADR 019](019-supervised-runtime.md) (runtime restart bounds)

## Context

When MoveUp crashes or fails to start, PM3 needs to know:
1. **What stage did it reach?** (setup, DB, serial, ready)
2. **Did it crash, or did the user quit?** (decide whether to restart)
3. **Is it actually ready?** (validate before promotion)

The existing session engine is pure and stateless about initialization. The tray
and serial connections are independent. Early failures (panic, DB corruption)
have no recovery path — they just disappear into the event log.

Three gaps needed closing:

- **No per-run identifier.** Multiple restarts blend together in logs.
- **No readiness surface.** PM3 must infer readiness from HTTP polls to
  `/display/api`, which may return stale data if the app dies before flushing.
- **No startup failure diagnostics.** A panic during setup leaves no trace,
  so PM3 cannot distinguish a crash loop from a slow startup.

## Decision

Implement three independent modules outside the session engine:

### 1. `app_lifecycle.rs` — Per-run tracking

Tracks lifecycle stages and quit intent. Records:
- **RunId**: PID + version + build_mode + startup_timestamp, stable across one run.
- **LifecycleStage**: Starting → Initializing → {TrayReady, DatabaseReady,
  SerialReady, RemoteReady} → Ready, or → QuitRequested or → Failed.
- **Quit intent flag**: Distinguishes explicit user quit from crash.
- **Failure reason**: Optional message if the app failed.

Owned by the coordinator and updated at key setup milestones. Persistence
happens after singleton ownership is established, so a second-instance
activation cannot overwrite the first instance's record.

### 2. `startup_diagnostics.rs` — Early failure capture

Writes to a minimal diagnostic log (`startup_diagnostics_<pid>.log`) if
normal logging fails. Designed to never panic or poison state:

- File creation is optional; absence of a log is itself a diagnostic.
- Writers lock per-attempt; a deadlock returns silently.
- Panic hook integration: `record_panic(message, backtrace)` captures the trace.

Called early in `main()` before Tauri setup. Stays dormant if setup succeeds;
PM3 ignores it when the app starts normally.

### 3. `runtime_readiness.rs` — Explicit readiness status

Validates readiness state independent of health sources. Exposed through
`/display/api` as an additive field (preserves existing API shape).

```json
{
  "readiness": {
    "app_id": "io.zntl.desk",
    "version": "0.6.0",
    "build_mode": "release",
    "state": "ready|initializing|degraded|failed",
    "reason": "sensor_missing|database_not_ready|...",
    "tray_initialized": true,
    "database_initialized": true,
    "serial_scanned": true,
    "remote_listening": true
  }
}
```

**Key rules:**
- **Sensor absence is degraded, not failed.** User can still interact without
  the device.
- **No outbound health calls.** Readiness answers from local state only.
- **compute_state(sensor_present)** updates the verdict based on
  initialization flags.

## Rationale

### Why three separate modules?

1. **Lifecycle** is owned by the coordinator and wired at app startup. It
   tracks initialization progression.
2. **Diagnostics** must be available even if logging fails, so it's
   independent and minimal.
3. **Readiness** is a query, not a store. It computes state from
   initialization flags, answering PM3's "is it ready yet?" question.

### Why per-run ID?

Multiple crash-restart cycles would otherwise be indistinguishable in logs.
PID + version + timestamp lets PM3 correlate events with a specific run and
detect restart loops (same version restarting within seconds).

### Why explicit quit flag?

`process.exit(0)` via `app.exit()` leaves Tauri to flush the session, but
there's no signal that this was intentional. Without the flag, PM3 sees a
zero exit code and cannot distinguish between:
- User quit via tray menu → don't restart.
- App reached Ready and exited normally → okay to restart.
- Crash that happened to exit 0 (unlikely but possible) → probably should
  restart.

The flag makes it explicit: `quit_requested=true` means "don't restart this
session."

### Why sensor absence is degraded?

A missing serial device is recoverable — the user can plug it in. An
uninitialized database is not. So:
- **Initializing**: Required subsystems (tray, DB, remote) are still starting.
- **Degraded**: All required subsystems ready, but sensor is missing.
- **Ready**: All required + sensor present.
- **Failed**: Some required subsystem failed (DB corruption, panic during
  startup).

## Implementation

### Wiring in setup_helpers.rs

```rust
let lifecycle = Arc::new(AppLifecycle::new(APP_VERSION, build_mode()));
lifecycle.advance_stage(LifecycleStage::Initializing);
app.manage(lifecycle.clone());
// After each major step:
lifecycle.advance_stage(LifecycleStage::TrayReady);
lifecycle.advance_stage(LifecycleStage::DatabaseReady);
// ...
lifecycle.advance_stage(LifecycleStage::Ready);
```

### Wiring in remote_server.rs

Add readiness to the `/display/api` response:

```rust
let readiness = ReadinessStatus::new(APP_ID, APP_VERSION, build_mode());
readiness.tray_initialized = /* from lifecycle */;
readiness.database_initialized = /* from lifecycle */;
readiness.remote_listening = /* true at this point */;
readiness.compute_state(/* sensor_present */);

json!({ ..., readiness })
```

### Event logging

Lifecycle events go through the existing `event_logger`:

```rust
event_logger.log(&format!("LIFECYCLE {} started", lifecycle.run_id()));
event_logger.log("LIFECYCLE ready");
event_logger.log("LIFECYCLE quit_requested"); // vs. clean_exit
```

## Constraints

- No changes to the session engine (`session_*.rs`).
- No dependencies on external modules except `event_logger`, `snapshot_logger`.
- Modules stay under 250 lines each, with sibling test modules.
- Readiness does not outbound-call health sources; it reports local state only.
- Startup diagnostics never panic, even if file creation fails.

## Verification

1. **Unit tests**: Lifecycle stage progression, quit flag, failure recording.
2. **Startup diagnostics tests**: File creation, panic capture, failure-to-create
   gracefully handled.
3. **Readiness tests**: State computation, sensor-absent degradation,
   serialization.
4. **Integration**: Tray initialization marks stage, DB open marks stage,
   lifecycle.ready() unblocks PM3 polling.

---

This work is task E014-T11 of the supervised release and rollback epic. PM3
contract changes (listening for readiness, detecting quit intent) are in E014
tasks T05/T06 and beyond.
