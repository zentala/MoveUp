# Supervised recovery source verification

## Lifecycle round 1 — 2026-10-09 (correction required)

- Source: `26d4618588de3714e8642d38c2e8295ce88e3b8d`, compared with `9c9057e`.
- Reviewer: a fresh independent Claude session, read-only acceptance and source
  review. Implementation and commit-finishing sessions have exited.
- Blocking findings: two, both source-confirmed by the independent verifier.
  F1 (Medium, 2 points): DB mutex contention incorrectly clears readiness
  (app_lifecycle_runtime.rs:33); preserve initialized state on contention and
  distinguish a genuinely absent connection. F2 (Medium, 2 points): an explicit
  failed prior outcome is classified as interrupted/unknown
  (app_lifecycle_store.rs:35); retain a separate previous_failed classification.
- Corrections dispatched only for F1/F2; neither is closed yet. The automated
  reviewer stop hook also retained both as conditions before merge.
- F3 (Medium, 3 points) belongs to the next integration scope: supervision health
  must remain available when LAN display is disabled and must be loopback-only.
  The coordinator will update the configuration/integration task accordingly.
- Remaining low-priority observations are recorded separately for scoped triage;
  they are not silently treated as implemented. Live desktop/login behavior
  remains NOT_CHECKED and belongs to later runtime acceptance.
- Coordinator verification: AO gate-only verification PASS, cargo exit 0;
  fresh child-process integration test command exit 0, four tests passed.
- Review scope: actual lifecycle wiring, durable outcome/singleton ordering,
  early failure and panic diagnostics, explicit bounded readiness, existing API
  compatibility, truthful persistence/tray evidence, sensor degradation.
- Retained failures: the first scaffold omitted required runtime integration;
  its write-set gate failed. Later git-add exit 128 reflected a protected index;
  source was preserved and a fresh compatible client committed it. rustfmt exit
  1 remains an unavailable-tool limitation, with no toolchain installation.
- No merge, task closure, live application change or login acceptance yet.

## Lifecycle round 2 — 2026-10-09 (source accepted)

- Frozen source: `0f06e4bef0a0f814b6f1f1e4f3313223a158337e`.
- Independent reviewer inspected the five-file correction and the author logs.
  F1 PASS: contention preserves known initialization, while absent/poisoned DB
  and failed/uninitialized lifecycle remain unready. F2 PASS: an explicit prior
  failure yields PreviousRun::Failed and a correlated previous_failed event;
  unfinished, corrupt and Quit-intent-only records remain interrupted/unknown.
- Confirmed blocking findings remaining: zero. Disputed findings: zero.
  F3 is explicitly scheduled in T05; optional observations remain in backlog.
- Author checks: 12 focused tests, 833 library tests and 4 child-process tests,
  all exit 0. Coordinator AO verification re-ran the library gate at this HEAD,
  exit 0, and separately re-ran all four child-process tests, exit 0.
- Live GUI, actual user log and login acceptance remain NOT_CHECKED. This
  accepts lifecycle source for integration, not deployment or epic completion.
- Retained refusal: a test-log write to system Temp failed checkout validation;
  the tests had not started. Logging to the same worktree's ignored target
  directory succeeded without changing the guard or approval controls.

## Readiness/configuration round 1 — 2026-10-09 (correction required)

- Frozen source: e41d2f6ea6c94a4e2142bbba74169bd50816ac08, base21aa490.
- Fresh independent read-only source reviewer accepted the dedicated loopback
  listener, LAN independence, typed identity and degraded sensor semantics.
- Three mandatory generator findings were independently confirmed:
  F1 High/5: ordinary configuration imports a custom TypeScript/VM loader of
  PM3 internals; use the public offline CLI boundary, keeping unavailable
  validation pending. F2 High/3: a second candidate projection competes with
  T07; consume the app-owned JSON path instead. F3 Medium/3: failure during
  two-file publication leaves a partial result; publish the sole config safely
  after F2, preserving input and existing output and testing failure/retry.
- Corrective scope contains those generator findings only. Rust source remains
  unchanged and accepted for this round; no source merge or PM3 acceptance yet.
- Existing evidence: author838 Rust library/12 focused/5 configuration tests
  passed. AO cargo gate exit0; initial write-set gate failed on filename
  mismatches. Manifest scope was reconciled to actual legitimate files, then
  gate-only verification passed. That scope repair did not certify behavior.
- Retained failures: Node child test runner spawnEPERM, initial source loader
  failure, and initial write-set gate. Later passing checks do not erase them.
- Live supervision, GUI and login remain NOT_CHECKED.

## Readiness/configuration round 2 — 2026-10-09 (one correction remains)

- Frozen source: 1befee770ff94484ff08937ebf4a507fff360647.
- Independent read-only source review accepted F1 (public offline CLI adapter)
  and F2 (unchanged app-owned candidate document). Atomic publication and
  concurrent-writer protection also passed source review.
- Remaining confirmed finding: F3 Medium/2. Cleanup after successful hard-link
  publication can throw, report preparation refused and suppress the published
  result even though the complete configuration exists. Separate publication
  and cleanup outcomes; preserve primary pre-publication errors. Add injected
  unlink/rmdir failure coverage without deleting an already-published output.
- Author and coordinator independently ran generator 13/13 and adapter 18/18,
  exit 0. Rust is unchanged. AO gate projection reports cargo exit 0, while its
  aggregate command still exits 1 from retained earlier execution attention;
  that nonzero command result remains recorded, not relabelled as success.
- Bounded correction is followed by the third/final review. PM3 owner source
  acceptance and real offline CLI interoperability remain pending. No merge,
  live change or desktop/login acceptance occurred.

## Readiness/configuration round 3 — 2026-10-09 (source accepted)

- Frozen source: 76f4d5e3044f942a0aa04028e81ae50f84723dbf.
- Independent read-only verifier accepted the remaining cleanup correction.
  Successful publication returns its actual result plus cleanup warnings;
  failed publication retains the original exception and reports secondary
  cleanup failures separately. Neither branch deletes the final output.
- Six new injected failure cases cover unlink/rmdir after success, alongside
  a conflicting publication, and alongside offline validation refusal. They
  assert unchanged candidate bytes, preserved foreign files and original
  error identity. Confirmed mandatory findings remaining: zero; disputed: zero.
- Author checks: generator 19/19 and adapter 18/18, exit 0. Coordinator reran
  generator 19/19, exit 0. Adapter and Rust source did not change. Final AO
  gate projection records cargo exit 0; aggregate verify still exits 1 from
  previous execution attention, retained explicitly pending normal integration.
- PM3 owner acceptance and actual public-launcher interoperability remain
  separate pending gates before merge. Source acceptance is not live runtime
  or login verification.

## Readiness dependency acceptance and integration — 2026-10-09

- PM3 owner source 5be48d773f1138301718858a965b8f4b038aa578 passed the final
  independent third review and owner's full gate (exit 0). Contract amendment 5
  records durable retry refusal, unknown hold applicability and eligible start
  intent. No live release is implied.
- MoveUp source 76f4d5e passed actual public-launcher offline interoperability:
  valid config accepted; app-owned candidate bytes preserved; existing output
  refused and preserved; missing executable refused before publication. Exit 0.
  A repeated run applied every fixture environment key, including a dedicated
  Windows named pipe and run/pid paths; exit 0. Neither run started a daemon.
- Normal AO merge produced 5ec6249. Its aggregate exit 2 retained cleanup's
  unknown/degraded state; the recorded merge verdict is merged, not a failed
  source gate. AO promote exited 0, integrated at b8f0c3d and closed T05 at
  fa628af. Cleanup remains pending; historical failures above remain retained.
- T06 may proceed. Runtime recovery, GUI and real-login acceptance remain open.
- Integrated main library check: 838 passed, zero failed, exit 0. No source
  changed during that check; later plan-only edits do not replace its scope.

## Intent round 1 — 2026-10-09 (three corrections required)

- Frozen source: 85e59e1d1f3719b782a2821234d3c7f368d6ca6c, base 800e2d4.
- Independent source verifier confirmed F1 High/3: ordinary Rust/PowerShell
  tests require PM3 Node environment despite Node being on PATH. Negative
  reproductions failed with exit 101 (one Rust test) and exit 1 (PowerShell
  after 32 assertions). Explicit Node runs passed four transport tests and
  41 assertions; those passes do not replace the failures.
- F2 Medium/2: duplicate ADR024 instead of amending the canonical lifecycle/
  readiness decision. F3 Low/1: five new module-registration lines fail
  git diff --check (exit 2). All three are confirmed; no disputed findings.
- Author: 15 focused/853 full Rust tests and 41 PowerShell assertions, exit 0.
  AO cargo and pwsh gates also exited 0. Aggregate verification FAILED its
  write-set check. Legitimate dev helper/fixture paths were added to the
  coordinator manifest; the duplicate ADR remains a required source correction.
- Independent PowerShell 5.1 forwarding proof passed after two reviewer probe
  errors (exit 1, then 64), retained as reviewer harness failures. Source was
  unchanged. Profile initialization's unrelated gitconfig refusal is retained;
  subsequent commands used login:false. No approval controls changed.
- Correct only F1/F2/F3, then freeze and admit round 2. Real GUI/exit/flush,
  supervision cycles and login are NOT_CHECKED here. No integration yet.
