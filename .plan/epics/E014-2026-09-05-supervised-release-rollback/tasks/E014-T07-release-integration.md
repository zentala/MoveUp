---
formatVersion: 1
id: E014-T07
title: Connect installed releases to candidate recovery and promotion
epic: E014
status: pending
points: 8
importance: High
dependencies: [E014-T05, E014-T06, E014-T11]
---

# Installed release integration

Read ../RESUME.md, ADR 018/019, the existing release_store/candidate_list/
last_known_good/health_probe modules and the accepted PM3 candidate contract.
Use the existing modules rather than maintaining a second release store.

Provide real application-owned install/stage, candidate projection and promotion
entrypoints. Maintenance commands must dispatch before Tauri's singleton/window
startup and cannot overwrite the active run record. Preserve the legacy newline
candidate list and publish a versioned JSON projection for PM3 atomically. Paths
and versions are validated, contained in the explicit release root, and never
interpreted as shell input. Reject symlink/junction escapes and incomplete builds.
Publish a candidate only after its complete artifact is staged successfully;
preserve the old candidate file and known-good marker on failure.
The existing release server resolves `dist/` beside its executable through
remote_server::default_dist_path. Stage and validate those frontend assets with
the executable; a copied desk.exe alone is not a complete installed release.
Test rejection of missing assets and independence from the launch directory.

Promotion must bind to the observed candidate and owned process/run evidence,
after at least 30 seconds alive with matching typed readiness. A different
instance of the same version is insufficient. MoveUp owns marker writes and
retention, PM3 owns polling, demotion and callback dispatch. Keep three releases
while protecting the known-good and running artifacts. A legacy 0.6.0 binary
without typed readiness must not be silently certified as the new-contract
fallback. No data downgrade or destructive database migration is in scope.

Update the existing version policy consistently for the new installable patch.
Never build over or replace the currently running executable. Expose clear
failure/status output for missing candidates, missing marker and callback failure.

Acceptance: tests exercising the actual entrypoints against temporary directories
for install failure, candidate ordering/deduplication, path escape, retention,
wrong identity/process, callback failure and successful promotion. Run repository
required checks and a release build in an isolated target. The full recovery
fixture through PM3's daemon belongs to T08.

Allowed scope: release modules and tests, small maintenance CLI and main/lib glue,
install/setup scripts, pm3 template, package/version manifests and corresponding
lock files, ADR/deployment docs. No production data, user process termination,
Run-key removal or PM3 source changes from this repository.
