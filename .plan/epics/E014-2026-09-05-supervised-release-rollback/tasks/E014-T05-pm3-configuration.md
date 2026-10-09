---
formatVersion: 1
id: E014-T05
title: Provide independent local readiness and the supervision configuration
epic: E014
status: completed
points: 5
importance: High
dependencies: [E014-T11]
---
<!-- pm3-plan-completion: E014-T05 15d9f38994a84ecbecc20aa255bc4c091b3af93ee232e2aaf132a7c981f87027 -->

# Installed application configuration

Re-estimated from 3 to 5 points after review finding F3: the current readiness
route shares an optional LAN server, so disabling remote display would cause
false supervision failures. The full epic is now 53 points; its AO route is unchanged.

Read ../RESUME.md and ADR 018/019 before implementation. This task serves the
existing always-available tracker vision. PM3 source acceptance is a prerequisite
for integrating its adapter, not for implementing the independent local listener.
Preparation against the owner's frozen interface is permitted in isolation; an
example schema or unshipped CLI is not a working runtime dependency. Record the
exact source revision validated and keep dependency acceptance explicitly pending.

Generate an explicit PM3 service configuration using its accepted supervised
candidate-file contract, typed readiness, two misses spaced 60 seconds, and
stand-down. Use resolved absolute installed paths, structured argv, and the
dedicated readiness route delivered by T11. Do not depend on outbound providers
or the current working directory. Keep installation opt-in until live cutover.
Document the minimum PM3 capability and fail with an actionable error if missing.
Do not silently start an old CLI against unsupported configuration.

Use the actual flat readiness contract: service-level expectJson is
`{ app_id: "io.zntl.desk", ready: true }`; each candidate additionally pins
`version`. Configure `instance.pidField: pid` and
`instance.runIdField: supervisor_run_id`: the app's separate `run_id` is its
own diagnostic UUID, while `supervisor_run_id` echoes the validated PM3_RUN_ID.
The foreign dev instance may pass service health without a supervisor identity,
but must never satisfy owned-child promotion. Test that distinction explicitly.

Before implementation, record the selected design in ADR024: readiness uses a
dedicated loopback-only listener in the same application process, independent of
remote_lan_enabled and the optional LAN display server. Preserve the existing
/display/api and LAN setting semantics. Remove the new readiness route from the
LAN-facing router so process/run metadata is not newly exposed there. Keep the
/display/readiness path on the local listener. Choose and document an explicit
configurable port and pass the same value to PM3; no silent port fallback.
No additional process, private watchdog or ergonomic engine change is permitted.

Acceptance: generated configuration passes the accepted PM3 schema in an isolated
home; candidate version and application identity must match; missing sensor
remains healthy enough for supervision. Test LAN disabled, busy LAN port, local
readiness bind failure, loopback binding, and unchanged existing display routes.
Optional display failure cannot make the essential application unready. Existing
PM3 services are untouched.
Return file paths, command exit codes and the PM3 source revision tested.

Allowed scope: repository pm3 configuration/template, scripts for local setup,
readiness runtime and server wiring, setup_helpers.rs, focused readiness/server
tests, ADR024 and deployment documentation. No live registration,
daemon replacement, Scheduled Task changes or Run-key removal.
Update PROJECT.xml for added modules/tests. Preserve the existing Run bridge.

## Implementation handoff

The coordinator supplies PM3_SOURCE and PM3_SUPERVISION_CONTRACT environment
variables for read-only source/contract inspection. The contract includes both
amendments (instance identity and independent suspension holds). Do not write
that repository or publish its private plan. The PM3 owner has not yet certified
its implementation; record actual validation separately from that pending gate.
Implement T05 only; T06 owns Quit/dev intent and T07 owns release maintenance.
No installation, process termination, service registration or machine changes.

Use a dedicated readiness_server module, preserving the existing app lifecycle
and pure engine boundaries. Keep implementation files under the repository's
line limits and reuse existing axum/tokio dependencies. Logs and scratch test
output belong in the worktree's ignored target directory. Test fixtures use
temporary directories and isolated local ports, never the installed application.

Run focused readiness/server tests and the complete Rust library suite. A
configuration fixture must verify exact readiness keys and process/run binding,
and validate against PM3's actual source when its standalone validator is usable.
An unavailable PM3 validation step is a named pending gate, not success.
Return changed files, exact exits/counts and remaining external acceptance.
