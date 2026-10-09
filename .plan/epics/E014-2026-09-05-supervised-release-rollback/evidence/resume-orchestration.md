# Supervised recovery execution evidence

## Lifecycle source

The initial attempt failed its write-set and runtime integration checks. It is
retained as failed; a later correction does not replace that evidence. A bounded
attended recovery completed lifecycle wiring in an isolated worker branch.

Corrected source passed the orchestration source gate and a fresh four-test
child-process integration run. Independent review then confirmed two defects:
database lock contention incorrectly cleared readiness, and an explicitly
failed prior run was classified as an unknown interruption. See
[review rounds](../review-rounds.md) for correction and acceptance status.
No live application change or login acceptance is implied by source tests.

Execution limitations retained: the first worktree commit attempt failed with
exit 128 because the Git index was protected. Source was preserved and a fresh
compatible execution profile completed the commit. Formatting failed with exit
1 because rustfmt was unavailable; no global toolchain was installed. A later
log redirection outside the worktree was refused by checkout validation; logs
were instead kept in the worktree's ignored build-output directory, without
changing the guard. Detailed operational records remain in ignored local evidence.

## Baseline checks

| Check | Observed result | Done |
|---|---|---|
| Frontend typecheck | Exit 0 | Yes |
| Frontend lint | Exit 0, 16 pre-existing warnings | Yes |
| Rust library baseline | 815 tests, exit 0 | Yes |
| First frontend coverage run | Exit 1: three timeouts, 467 passed | Failed |
| Focused single-worker retry | 23 tests, exit 0 | Yes |
| Full single-worker coverage retry | 57 files, 470 tests, exit 0 | Yes |

The retry retained the original failure and changed neither assertions nor
timeout settings. Coverage: statements 89.70%, branches 80.57%, functions
85.45%, lines 91.76%. These are baseline checks, not acceptance of later work.

## Integration boundaries

The process manager's generic recovery dependency is being implemented in its
own repository. Source review, isolated daemon tests and live release are
separate gates. MoveUp still needs independent loopback readiness, the verified
supervision adapter, installed release integration and runtime/login acceptance.
The current application and direct-login bridge remain in place until the
replacement passes the required checks. No production data migration or failure
injection is authorized by the fixture results.
