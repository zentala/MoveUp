---
formatVersion: 1
id: E014-T05
title: Declare the installed application supervision contract
epic: E014
status: pending
points: 3
importance: High
dependencies: [E014-T11]
---

# Installed application configuration

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

Acceptance: generated configuration passes the accepted PM3 schema in an isolated
home; candidate version and application identity must match; missing sensor
remains healthy enough for supervision. Existing PM3 services are untouched.
Return file paths, command exit codes and the PM3 source revision tested.

Allowed scope: repository pm3 configuration/template, scripts for local setup,
adapter configuration tests and deployment documentation. No live registration,
daemon replacement, Scheduled Task changes or Run-key removal.
