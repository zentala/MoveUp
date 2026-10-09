# Installed release maintenance

## Correction verification continuation (2026-10-09)

Manual activation requires `DESK_PM3_CONFIG` pointing to the final generated
installation `pm3.yaml`; preparation carries it in the service environment.
Activation validates the generated JSON/YAML, explicit service/Node/home and
installation paths before invoking the selected public launcher. It supplies
`PM3_CONFIG` and a deterministic child working directory. Generic PM3 start
output is text; MoveUp wraps it in JSON with command completion and
`readiness:not-checked`, preserving held/no-candidate output without falsely
claiming a running application. Nonzero and unknown outcomes remain explicit.

Candidate publication attempts to restore prior compatibility bytes after
JSON publication failure. If restore write, rename or removal fails, the error
retains both publication and restoration evidence and reports uncertain state.
This is not a cross-file power-loss transaction.

The historical failed checks below remain retained. A later successful check
does not erase their exit codes. The correction work also updates PROJECT.xml
under the coordinator's explicit expanded write scope.

MoveUp 0.6.1 maintains complete immutable builds. Maintenance commands run before
GUI, singleton ownership, diagnostics and live database access. All commands
emit JSON; failures emit JSON on stderr and exit nonzero. Nothing here removes
an autostart registry value or changes production data.

Build only after the repository test gates pass, using a private target rather
than `src-tauri/target/release` in the main checkout. The work item uses
`src-tauri/target/e014-t07-isolated`; its intended release executable is
`src-tauri/target/e014-t07-isolated/release/desk.exe`. Never build over the
installed or running executable. Prepare a separate artifact directory containing
the new executable and the frontend build's complete `dist/` tree beside it.

```powershell
scripts/stage-release.ps1 -ArtifactDirectory C:/example/build-0.6.1 `
  -ReleaseRoot C:/example/MoveUp/releases -RunDirectory C:/example/app-data
```

The wrapper refuses legacy binary version resources, asks the new binary for
`--release describe`, writes `release.json` into build output, and calls
`--release stage --root ABS --source ABS --version 0.6.1 --run-directory ABS`.
`RunDirectory` must be the existing Tauri app-data directory containing
`lifecycle.owner` and `lifecycle.json`, not the session database file.
The output reports the exact artifact and `cutover:false`. Existing immutable
versions, missing HTML/assets, path traversal, symlinks/junctions and competing
writers are refused. Legacy binaries are never silently listed or certified.

The stage command protects the locked active run and marker and retains three
releases. Unknown/legacy artifacts are also protected. If that prevents safe
retention, it refuses installation rather than deleting a possible running
artifact. Only the application owns marker writes and retention.

```powershell
desk.exe --release status --root C:/example/MoveUp/releases
desk.exe --release candidates --root C:/example/MoveUp/releases
```

Status distinguishes absent/empty candidate files and an absent marker.
Candidate publication preserves the old newline list and writes candidate-file
v1 JSON for PM3. The marker stays absent until identity-checked promotion.

To prepare installed supervision, select an absolute Node executable and PM3
home and provide the run directory:

```powershell
node scripts/prepare-pm3.mjs --install-root C:/example/MoveUp `
  --candidates-file C:/example/MoveUp/releases/candidates.json `
  --output-dir C:/example/supervision --pm3-node C:/example/node.exe `
  --pm3-home C:/example/pm3-home --run-directory C:/example/app-data
```

This publishes a stable copy of `release-callback.mjs` beside `pm3.yaml` and
validates the full configuration through the selected public launcher. Move or
delete neither after registering that configuration. No PM3 source checkout is
loaded. Preparation does not register a service or claim runtime acceptance.
Omitting the run-directory option preserves the earlier draft-only mode.

PM3 owns the thirty-second observation window and gives the callback ten seconds.
The callback dispatches on `PM3_CANDIDATE_ID` rather than the newest binary. The
application checks owned/observed PID, supervisor run identity, local probe URL,
locked lifecycle record and one fresh bounded typed readiness response. It
accepts a degraded sensor when essential initialization is ready, and refuses
unknown, foreign, exited or quitting instances. Callback errors preserve the
marker and return nonzero. This is revalidation, not a second promotion timer.
The callback argv also carries the selected PM3 home and readiness port. Its Node
process reinstates the candidate's supervision settings explicitly: PM3 callback
execution inherits daemon environment, which cannot substitute for service env.

Manual activation uses `desk.exe --release activate` with the same
`DESK_SUPERVISION=pm3`, `DESK_PM3_SERVICE=moveup/desk`, `DESK_PM3_NODE` and
`DESK_PM3_HOME` settings carried in installed `pm3.yaml`. It calls the selected
public launcher with `start moveup/desk` and wraps its text response in JSON;
it does not bypass suspension
holds with a second app launcher. Missing or malformed configured mode fails.

Validation comprises the full Rust library suite, real maintenance-process and
callback tests (`cargo test --test release_maintenance`), Node PM3 preparation
and callback tests, frontend coverage, TypeScript and lint, then an isolated
release build. The recovery fixture through the PM3 daemon and attended login
verification remain separate gates; no live cutover is performed here.

See [ADR 019](../.arch/ADR/019-release-store-layout.md) for publication/retention
limits and [ADR 024](../.arch/ADR/024-application-lifecycle-readiness.md) for readiness.

## Worktree validation — 2026-10-09

Implementation version: 0.6.1 in npm, Cargo, Cargo lock and Tauri manifests.
Required acceptance remains **blocked**, not passed. Changes remain uncommitted
for orchestrator collection; generated task/handoff inputs were not modified.

| Check | Observed result | Exit |
| --- | --- | --- |
| Offline Cargo check with isolated target | Passed | 0 |
| Full offline Rust library suite, final source | 863 passed | 0 |
| Actual pre-GUI executable promotion test | Passed; active run record unchanged | 0 for the test |
| Actual Node-to-retained-executable callback | Failed: `spawnSync desk.exe EPERM`; passing the mock dispatcher does not replace this gate | 101 for integration suite |
| PM3 configuration/preparation tests | 21 passed | 0 |
| Public PM3 validation adapter tests | 18 passed | 0 |
| Dispatcher argument/error tests | 4 passed, including explicit callback environment | 0 |
| TypeScript `tsc --noEmit` | Passed | 0 |
| ESLint | Passed with 16 existing warnings | 0 |
| Frontend coverage via package script | Failed: pnpm attempted automatic dependency reconciliation and refused module removal without a TTY | 1 |
| Direct installed Vitest coverage command | Failed: esbuild process spawn returned `EPERM` | 1 |
| Isolated release build | Not run: repository pre-build test gate remains failed | Not checked |
| CRLF-aware diff check and script syntax checks | Passed | 0 |

The initial source-overlap rejection assertion failed (Cargo exit 101); the
explicit containment check was moved before complete-tree validation and the
full library suite then passed. Earlier integration failures remain recorded:
the first callback fixture timed out without reaching readiness, and the bounded
diagnostic rerun exposed Node's `spawnSync ... EPERM` directly. They are failures,
not missing evidence. No permission bypass was attempted and no dependency
reinstall completed.

Ignored local check logs live under `src-tauri/target/e014-t07-*.log`, including
`e014-t07-library-accepted.log`, `e014-t07-callback-accepted.log`,
`e014-t07-pm3-config-final.log`, `e014-t07-pm3-validation.log`,
`e014-t07-dispatcher.log`, `e014-t07-coverage.log` and
`e014-t07-coverage-direct.log`. The executable exercised by the maintenance test
is `src-tauri/target/e014-t07-isolated/debug/desk.exe`, a debug test artifact,
not an installable release. No 0.6.1 release artifact was certified or staged live.

Next acceptance action: run the real callback test and frontend coverage in an
authorized execution environment that permits the required Node child processes;
after those pass, build the frontend and 0.6.1 executable in the isolated target,
then obtain independent source verification. The recovery-daemon and attended
login fixtures remain outside this work item. Before integration, the coordinator
also needs to reconcile the new modules/tests in `PROJECT.xml`; that file is
outside this work item's declared release/CLI/script/manifest/document write set.
