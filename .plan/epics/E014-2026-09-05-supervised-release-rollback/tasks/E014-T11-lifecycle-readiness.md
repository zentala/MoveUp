---
formatVersion: 1
id: E014-T11
title: Lifecycle diagnostics and explicit application readiness
epic: E014
status: completed
priority: high
effort: large
points: 8
importance: High
dependencies: []
tags: [task, reliability, E014]
created_at: 2026-10-09
commit: "feat(reliability): record lifecycle and explicit readiness [E014-T11]"
---
<!-- pm3-plan-completion: E014-T11 e8033f6af399ce451f7669413a3db4eda780b463973c746f67690e88722f3b0c -->

# E014-T11: Lifecycle diagnostics and explicit readiness

Read ../RESUME.md, root CLAUDE.md and .arch/ADR/015-pure-ergo-engine.md first.
This is one coherent 8-point infrastructure task. No PM3 implementation or live
cutover is required here. No user process may be stopped or autostart modified.

## Scope

Implement a small Rust lifecycle module outside the ergonomic engine. Wire
bootstrap/setup/run/exit/tray to it, using the existing EventLogger format.
Use a per-run identifier with PID/version/build mode and bounded stage names.
Record started, ready, explicit Quit request, clean exit and observed failure.
Keep existing session flush semantics; report skipped/failed persistence honestly.
Persist minimal run outcome atomically, after singleton ownership is established,
so a second-instance activation cannot overwrite a running instance's record.
Missing clean-exit evidence means interrupted/unknown, never a guessed cause.
Early setup failures and Rust panics need a best-effort minimal diagnostic sink
that does not lock application state, poison it, panic recursively or log secrets.
No general catch-and-continue after panic. Failed diagnostics may not crash an
otherwise working app. No claim that a panic hook catches external termination.

Add an additive readiness surface to the existing local display server with a
documented schema: app identity, version, run identity, process mode, initializing/
ready/degraded/failed and initialization of the tray and essential persistence.
The existing /display/api shape and readers must remain compatible. Keep the
health-probe module's existing public callers working; add typed validation for
new supervision adapters. Being able to return arbitrary JSON alone cannot pass
the new readiness verdict. Sensor absence is degraded and restart-ineligible.
Do not perform outbound health-provider calls to answer readiness. No frontend
redesign; expose only truthful available readiness evidence.

## Files and boundaries

Allowed: src-tauri/src/app_lifecycle*.rs, src-tauri/src/startup_diagnostics*.rs,
src-tauri/src/runtime_readiness*.rs, src-tauri/src/main.rs, lib.rs,
setup_helpers.rs, tray.rs, remote_server.rs, health_probe*.rs, event_logger.rs,
.arch/ARCHITECTURE.md and .arch/ADR/024-application-lifecycle-readiness.md.
Sibling test modules and fixture binaries under src-tauri/tests/ are allowed.
Do not edit SessionManager, session_*.rs, COM logic, profiles, live app data,
global config, PM3, dependency versions, generated files or plan documents.
Use existing dependencies. Keep new modules below 250 lines with sibling tests.

## Verification

Meaningful Rust tests for lifecycle transitions/correlation, normal exit,
interrupted previous run, unwritable diagnostics, secondary-instance protection,
early failure/panic in isolated child process, readiness before/after tray and
DB initialization, identity validation, and missing sensor without restart.
Run cargo test --manifest-path src-tauri/Cargo.toml --lib and any added integration
tests. Compile release configuration too if cfg gates differ. No test may use
real profile/DB/Run keys. Do not build over the running target/release/desk.exe.
Return changed files, test counts/exit codes, known failures and scope gaps.
Do not self-certify independent review or update task completion state.
