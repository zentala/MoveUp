---
formatVersion: 1
id: E014-T05
title: Provide independent local readiness and the supervision configuration
epic: E014
status: pending
points: 5
importance: High
dependencies: [E014-T11]
---

# Installed application configuration

Re-estimated from 3 to 5 points after review finding F3: the current readiness
route shares an optional LAN server, so disabling remote display would cause
false supervision failures. The full epic is now 53 points; its AO route is unchanged.

Read ../RESUME.md and ADR 018/019 before implementation. This task serves the
existing always-available tracker vision. PM3 source acceptance is a prerequisite;
an example schema or unshipped CLI is not a working runtime dependency.

Generate an explicit PM3 service configuration using its accepted supervised
candidate-file contract, typed readiness, two misses spaced 60 seconds, and
stand-down. Use resolved absolute installed paths, structured argv, and the
dedicated readiness route delivered by T11. Do not depend on outbound providers
or the current working directory. Keep installation opt-in until live cutover.
Document the minimum PM3 capability and fail with an actionable error if missing.
Do not silently start an old CLI against unsupported configuration.

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
