# Prepare MoveUp supervision

MoveUp exposes flat application readiness on a dedicated loopback listener at
`127.0.0.1:3391/display/readiness`. Set `DESK_READINESS_PORT` to an integer from
1 to 65535 to choose another port; set the same value for every installed/dev
instance and PM3. Invalid values and occupied ports are reported as failures,
never retried on a silently selected port. The optional LAN display remains on
`DESK_REMOTE_PORT` (default 3390) and obeys `remote_lan_enabled` independently.
Its routes no longer expose readiness process/run metadata.

Tray, persistence and setup must be initialized, with no observed failure or
Quit request. A disconnected sensor is `ready: true`, `state: degraded`.
Unavailable LAN display does not make the application unready. Readiness
does not read remote health providers or the display aggregate.

## Preparation only

The generator writes a new configuration and candidate document into an explicitly
provided directory. It never loads the PM3 CLI, registers a service, starts an
application, changes Scheduled Tasks or removes the Run-key bridge. `boot: false`
keeps this preparation out of automatic boot restore. Do not install it until the
PM3 owner accepts its implementation and the separately owned live cutover passes.

Requirements: Node with this checkout's installed TypeScript dev dependency;
installed candidate executables; the ordered newline-separated candidate list
from MoveUp's release store; an existing, isolated output directory; and the
owner-provided PM3 source with its dependencies available.

```powershell
node scripts/prepare-pm3.mjs --install-root C:/example/MoveUp `
  --ordered-candidates C:/example/MoveUp/releases/candidates `
  --output-dir C:/example/prepared-supervision `
  --pm3-source C:/example/pm3-source --port 3391
```

All paths must be absolute. The example directories are placeholders: supply
resolved paths for the intended installation. Executables are checked and
resolved without executing them. The input order is preserved, never recomputed.
The generator refuses existing output files, so it cannot overwrite a service
configuration. `PM3_SOURCE` may replace `--pm3-source`.

Output is `pm3.yaml` (JSON, valid YAML) plus `candidates.json`. The service name is
`moveup/desk`. It uses structured argv (`desk.exe`, `--minimized`), absolute cwd,
no `.env` discovery, two misses 60 seconds apart, and stand-down. Service-level
health matches `{ app_id: "io.zntl.desk", ready: true }`; candidate-level health
adds its exact `version`. The PID key is `pid`, and the supervision identity key
is `supervisor_run_id`, which echoes validated `PM3_RUN_ID`. The app's `run_id`
remains its independent diagnostic UUID. A foreign healthy dev instance may
satisfy service health, but never proves ownership or permits promotion.

## Minimum PM3 capability and pending gates

PM3 must implement supervised candidate-file v1, bounded typed HTTP health,
stand-down, miss spacing, instance PID/run binding, and independent suspension
holds (the two amendments to its frozen supervision contract). Its current
package version alone does not identify that capability.

The preparation tool evaluates the supplied source schema and candidate parser
in memory, including the workspace runtime-release schema. It records source
hashes, refuses a schema that drops required supervision fields, and reports an
actionable error if source or dependencies are missing. It does not try an old
CLI as a fallback. This checks the application adapter, not the live daemon.

External gates remain **pending**: PM3 owner certification (including suspension
holds and runtime promotion), Quit/dev intent integration, release maintenance
and promotion callback, and supervised logon cutover. This task intentionally
omits the promotion callback: no last-known-good marker is changed by preparation.
No public domain is registered for this process-metadata-only local endpoint.

## Verification evidence — 2026-10-09

PM3 source checkout HEAD inspected before and after the final configuration
fixture: `c4bb47dff144a78d1baf9d46691e42323ffb5696`. Earlier inspection saw
`e735e1f241cf72320bf2607b537ee3c283e767b9`; the owner's checkout advanced during
this task, so validation was repeated. The source hashes below were unchanged.
HEAD identifies the checkout, while the hashes identify the evaluated files;
no clean-tree or owner-acceptance claim is made.
Source was evaluated directly, not from potentially stale compiled CLI output.
The fixture checks foreign-dev vs owned-child identity using PM3's actual
`isOwnedInstance`, typed comparison and candidate-merge functions.

- `node scripts/pm3-config.test.mjs`: 5 passed, exit 0; synthetic installation,
  isolated configuration directory, fixture cleanup, no process launch.
- Earlier `node --test scripts/pm3-config.test.mjs`: failed, exit 1 (`spawn EPERM`
  in the sandbox's test-worker launch). Direct execution above runs the same
  `node:test` cases without a child runner.
- Initial source-loader check failed, exit 1: PM3's runtime-release package has
  import-only exports. The loader now evaluates that workspace's source as well;
  the source-validation fixture above passes.

Source SHA-256 evidence:

| PM3 file | SHA-256 |
| --- | --- |
| `src/config-schema.ts` | `dbc811dd6a4bd482744c5c575163c93ad231430eedefa3cb57f3b586e1866423` |
| `src/supervised/candidate-file.ts` | `9023071d7be42ed94cf380c77ccd0eb2af38a31891dc10f96c0271185ff44d1d` |
| `src/health-json.ts` | `93b02c801cd1bd323d548bc66f11cbd16c2100513cea703ae44a3f17de3ad2f7` |
| `src/supervised/controller-state.ts` | `d831eaafd42a1a71e9578d9f69d884ff89f7a1e5bd34b89c184421b3c9ebba8e` |

- `cargo test --offline --lib readiness --manifest-path src-tauri/Cargo.toml`:
  12 passed, 826 filtered out, exit 0. Covers loopback binding, LAN disabled,
  occupied LAN port, occupied readiness port, unchanged display API, and no
  readiness metadata on the LAN router.
- `cargo test --offline --lib --manifest-path src-tauri/Cargo.toml`:
  838 passed, 0 failed/ignored, exit 0. Full output is local ignored build output
  at `src-tauri/target/e014-t05-library-tests.log`. Four pre-existing unused-variable
  warnings in `session_tests_timers.rs` remain outside this task.
- `PROJECT.xml` parsed successfully; `git diff --check` returned exit 0.

Changed files: this document; `PROJECT.xml`;
`.arch/ADR/024-application-lifecycle-readiness.md`;
`scripts/{pm3-config.mjs,pm3-config.test.mjs,pm3-source-validation.mjs,prepare-pm3.mjs}`;
`src-tauri/src/{readiness_server.rs,readiness_server_tests.rs,health_probe_readiness.rs,lib.rs,remote_server.rs,runtime_readiness_tests.rs,setup_helpers.rs}`.
Generated dependencies, task references and installed services are unchanged.
Independent source verification remains the coordinator's acceptance step.

Design: [ADR 024](../.arch/ADR/024-application-lifecycle-readiness.md).
