# Supervised recovery continuation — 2026-10-09

## Accepted outcome

The operator asked to finish the existing supervision project as proposed:
lifecycle and failure diagnostics, explicit application readiness, PM3-owned
restart respecting Quit/development, installed release selection and rollback,
and verified login startup. This document changes implementation scope, not
the product vision in `../../vision/2026-03-15-desk-app-vision.md`.

◉ MoveUp remains available after an unexpected process exit, explains observed
failures, respects an intentional stop, and starts at login through one owner.
⑄ Existing E014 (43 points) plus E014-T11 (8 points) and T05's 3-to-5-point
re-estimate for independent loopback readiness, full total 53 points.
⑆ Scoped MoveUp changes, isolated test fixtures, attended integration and
deployment; PM3 changes run in its own repository and Herdr workspace.
⑀ Read repository CLAUDE.md, ADR 018/019, engine boundary ADR 015 and this file
before implementation. Read DESIGN.md before changing any visual surface.
⋈ Coordinator supplies reviewed commits; operator accepts visible behavior.
⁅ All acceptance below needs actual source/runtime evidence. A missing test,
unknown result or unperformed login remains incomplete; keep the goal active. ⁆

## Verified starting evidence

- Main checkout clean at `11b5c20`; release running is still 0.6.0 from the
  build output directory. Its October 8 log proves startup, not cause of exit.
- `release_store`, `last_known_good`, `candidate_list`, `health_probe` and
  tests exist but are not wired into the actual startup/install path.
- `ensure_autostart` enables/rewrites the Run value on release launch; the
  settings checkbox therefore is not an authoritative persistent off switch.
- `Quit` invokes `app.exit(0)`; Tauri Exit flushes the session but records no
  explicit lifecycle outcome. Existing log/profile failures must stay nonfatal.
- The `pm3d` Scheduled Task now exists, is Interactive, Running, and has a
  LogonTrigger. This removes an old absence claim, not the live-login gate.
- PM3 candidate/stand-down contract is being checked on current source in a
  task-owned Herdr tab in its existing workspace. No live daemon changes yet.

## Delivery order and reservations

Checkpoint: T11 source is integrated at `9c552e3`; independent review round2
accepted it, and integrated library (833) / child-process (4) tests passed.
Real desktop/login evidence in the acceptance matrix remains outstanding.
T05 source at 76f4d5e passed independent round 3 and public-launcher validation
against accepted PM3 source 5be48d7 with all fixture isolation keys applied.
AO integrated it at b8f0c3d and completed its source task at fa628af. T06 is next;
real supervision, desktop behavior and login remain separate acceptance gates.

1. T11 (8 points, High): MoveUp lifecycle diagnostics and readiness, independent
   of the PM3 contract. Preserve current Run registration until cutover.
2. PM3 owner implements/verifies missing generic dependency in its own plan.
3. T05/T06 (5 + 3 points, High): independent loopback readiness, actual config,
   start/stop intent and dev coexistence based on verified PM3 APIs. Readiness
   must work when LAN display is disabled; process/run metadata stays off the
   LAN router. Resolve alternate-port dev explicitly.
4. T07 (8 points, High): connect install/release store/probe/candidates/marker to
   real commands and PM3 callbacks. Keep last-known-good and data compatibility.
5. T08/T10 (5 + 3 points, High): fixture rollback and real desktop checks;
   retire direct Run registration only after PM3 login ownership is proven.

Route is attended AO (total 53) from Herdr; every run has explicit manifest,
ledger, isolated worktree and independent review. Initial PM3 reconnaissance
was read-only; its owner now coordinates the generic dependency implementation
in that repository. Source acceptance and live release are separate gates.
Preserve other agents and the running desk process.

The coordinator owns plan edits. Workers own only their task write sets.
Do not change the ergonomic state engine, public relay or personal health data.
Ordinary runtime persistence is allowed; no destructive database migration,
restore, downgrade of incompatible data, or test on the production database.

## Acceptance evidence matrix

| Requirement | Required evidence | Done |
|---|---|---|
| Start/ready/stop diagnostics correlate one run | Unit tests and real local log | No |
| Panic/early setup failure remains diagnosable | Isolated child-process fault fixture | No |
| Forced exit is not mislabelled as known crash cause | Interrupted-run fixture + observer evidence | No |
| Readiness checks app identity/version/tray initialization | Contract tests; backend liveness alone insufficient | No |
| Missing sensor is degraded, no restart loop | Device-absent fixture | No |
| Recovery bounded and delays obey two-miss rule | PM3 tests and isolated runtime evidence | No |
| Quit stays stopped for session and permits next login | Intent-protocol tests and desktop acceptance | No |
| Dev singleton is not fought, alternate port explicit | At least two real poll cycles + alternate-port fixture | No |
| Broken newest version falls back and stays demoted | Synthetic release fixture through real PM3 path | No |
| Marker and three-version retention preserve fallback | Existing + integration tests | No |
| Installed version starts through PM3 at login | Actual login evidence with one process and tray | No |
| Direct Run registration removed at safe cutover | Registry + code checks after predecessor gate | No |
| Integrated build/review/push and durable task closure | Source review, required gates, owner remote SHAs | No |

No unannounced reboot/logoff: prepare the concrete cutover first and arrange
the disruptive final login verification with the operator when ready.
