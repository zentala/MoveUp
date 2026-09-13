# AO branches in MoveUp — inventory before cleanup (2026-09-14)

## TLDR

113 `ao/*` branches left by Agent Orchestrator runs E014-E022 (2026-09-06/07). 84 were fully merged into `main` and were deleted with `git branch -d` on 2026-09-14. 29 are not merged: every one is a failed or retried attempt of a task whose other attempt did land on `main` (28 worker branches), plus the integration branch of the abandoned E015 run `0425`. They are kept. Restore any deleted branch with `git branch <name> <sha>` from the table below.

## Deleted — merged into main (84)

| Branch | SHA | Last commit |
|---|---|---|
| `ao/integration/E014-20260906-2012` | `88d7c20` | Merge branch 'ao/worker/E014-T04/E014-20260906-2012-0' into ao/integra |
| `ao/integration/E015-20260906-0408` | `bada223` | chore(state): refresh STATE.md timestamp |
| `ao/integration/E015-20260906-0449` | `e4da068` | Merge branch 'ao/worker/E015-T04/E015-20260906-0449-0' into ao/integra |
| `ao/integration/E016-20260906-0348` | `0c4714c` | Merge branch 'ao/worker/E016-T03/E016-20260906-0348-0' into ao/integra |
| `ao/integration/E017-20260906-0622` | `9a25332` | Merge branch 'ao/worker/E017-T08/E017-20260906-0622-0' into ao/integra |
| `ao/integration/E018-20260906-1209` | `3acc1d8` | Merge branch 'ao/worker/E018-T11/E018-20260906-1209-0' into ao/integra |
| `ao/integration/E019-20260906-0822` | `f9c6b1c` | Merge branch 'ao/worker/E019-T07/E019-20260906-0822-0' into ao/integra |
| `ao/integration/E020-20260906-1418` | `20768eb` | Merge branch 'ao/worker/E020-T08/E020-20260906-1418-0' into ao/integra |
| `ao/integration/e021-run1` | `5c3aa9f` | Merge branch 'ao/worker/E021-T10/e021-run1-0' into ao/integration/e021 |
| `ao/integration/e022-run1` | `6c08f6d` | Merge branch 'ao/worker/E022-T14/e022-run1-0' into ao/integration/e022 |
| `ao/worker/E014-T01/E014-20260906-2012-0` | `6395aa2` | feat(E014-T01): add versioned release store with retention |
| `ao/worker/E014-T02/E014-20260906-2012-0` | `6e0d6b1` | feat(E014-T02): add last-known-good marker with probe semantics |
| `ao/worker/E014-T03/E014-20260906-2012-0` | `43d71e4` | feat(E014-T03): add health-good probe for release promotion |
| `ao/worker/E014-T04/E014-20260906-2012-0` | `d272e3d` | feat(E014-T04): write the ordered rollback candidate list |
| `ao/worker/E014-T09/E014-20260906-2012-1` | `3e44070` | docs(E014): add ADR 018/019 and wire them into ARCHITECTURE.md |
| `ao/worker/E015-T01/E015-20260906-0449-0` | `d75a02f` | refactor(session): delete the second sitting counter |
| `ao/worker/E015-T02/E015-20260906-0449-0` | `e33a286` | refactor(ui): read the credited counter, add a DTO drift test |
| `ao/worker/E015-T03/E015-20260906-0449-0` | `c9765ff` | feat(session): persist break credit, fix posture balance |
| `ao/worker/E015-T04/E015-20260906-0449-0` | `dbea176` | docs(session): one credited counter in ADR 008, UX-FLOW, ARCH |
| `ao/worker/E016-T01/E016-20260906-0348-0` | `a8da138` | docs(plan): make STATE.md body match its frontmatter |
| `ao/worker/E016-T02/E016-20260906-0348-0` | `5805aa8` | docs(plan): consolidate root backlog files into .plan/BACKLOG.md |
| `ao/worker/E016-T03/E016-20260906-0348-0` | `8ab96b5` | docs(plan): close E011 ceremony, mark E003-T07 and E013 superseded |
| `ao/worker/E016-T04/E016-20260906-0348-0` | `58a92c2` | chore(repo): untrack coverage and test-performance-report |
| `ao/worker/E016-T05/E016-20260906-0348-0` | `b244908` | chore(ao): add .giter.yaml and root justfile |
| `ao/worker/E017-T01/E017-20260906-0622-1` | `529a76c` | docs: rewrite user docs for MoveUp naming and real paths |
| `ao/worker/E017-T02/E017-20260906-0622-0` | `1901a98` | docs(updates): describe the real manual update path |
| `ao/worker/E017-T03/E017-20260906-0622-0` | `9375366` | docs(privacy): rewrite for MoveUp and disclose Google Fit |
| `ao/worker/E017-T04/E017-20260906-0622-0` | `af8d2f5` | chore(desk): add MIT license and fix crate metadata |
| `ao/worker/E017-T05/E017-20260906-0622-0` | `17b2842` | ci(E017-T05): run test workflow from repo root |
| `ao/worker/E017-T06/E017-20260906-0622-0` | `593f8b7` | docs(firmware): add flashing guide and cable warning |
| `ao/worker/E017-T07/E017-20260906-0622-0` | `40f5485` | chore(perf): refresh perf baseline and add E017-T07 check |
| `ao/worker/E017-T08/E017-20260906-0622-0` | `ae43ceb` | chore(e017): add aggregate check and record MIT licence decision |
| `ao/worker/E018-T01/E018-20260906-1209-0` | `477a8e7` | refactor(desk): share one desk-state reducer across both transports |
| `ao/worker/E018-T02/E018-20260906-1209-2` | `6059092` | docs(desk): record ADR 017 — ts-rs for the Rust-to-TypeScript codege |
| `ao/worker/E018-T03/E018-20260906-1209-1` | `693c7f3` | refactor(desk): delete dead components and overlay build entry |
| `ao/worker/E018-T04/E018-20260906-1209-0` | `da7b664` | chore(desk): install and configure ESLint flat config |
| `ao/worker/E018-T05/E018-20260906-1209-1` | `ee4b3e4` | build(desk): enforce the coverage gate in build; finish the justfile |
| `ao/worker/E018-T06/E018-20260906-1209-0` | `0bac1a1` | refactor(desk): consolidate local format helpers into utils/format |
| `ao/worker/E018-T07/E018-20260906-1209-0` | `200be93` | docs(desk): correct overlay dev-launcher references to tauri-dev.ps1 |
| `ao/worker/E018-T08/E018-20260906-1209-0` | `b11064c` | docs(desk): sync UX-FLOW with Steps widget and timeline-to-Analyst cli |
| `ao/worker/E018-T09/E018-20260906-1209-0` | `98b6bb2` | feat(analyst): move date title to top, pin day-nav to bottom |
| `ao/worker/E018-T10/E018-20260906-1209-1` | `3eaeeae` | feat(desk): replace analyst KPI bars with recharts donuts |
| `ao/worker/E018-T11/E018-20260906-1209-0` | `0a1e346` | feat(desk): pulse KPI donuts when the selected day changes |
| `ao/worker/E019-T01/E019-20260906-0822-0` | `65f342b` | docs(backlog): mark E015/E016 done, they were stuck at [ ] |
| `ao/worker/E019-T01/E019-20260906-0822-1` | `576b8f2` | fix(desk): make commands.rs mutex locks poison-safe |
| `ao/worker/E019-T02/E019-20260906-0822-0` | `f6520c9` | refactor(desk): one composition point for today's totals |
| `ao/worker/E019-T03/E019-20260906-0822-0` | `94e5eae` | refactor(desk): name desk events with shared constants |
| `ao/worker/E019-T04/E019-20260906-0822-1` | `f37f1e2` | fix(desk): warn instead of panic when the event log dir is unusable |
| `ao/worker/E019-T05/E019-20260906-0822-0` | `08ab0ed` | refactor(desk): split google fit modules under the file-size cap |
| `ao/worker/E019-T06/E019-20260906-0822-0` | `596e92a` | refactor(desk): split serial periodic steps out of the orchestrators |
| `ao/worker/E019-T07/E019-20260906-0822-0` | `d23fb5f` | refactor(desk): share one remote display state derivation |
| `ao/worker/E019-T08/E019-20260906-0822-0` | `65f342b` | docs(backlog): mark E015/E016 done, they were stuck at [ ] |
| `ao/worker/E019-T08/E019-20260906-0822-1` | `8cb5e7b` | docs(desk): correct TrayController decision-logic claim |
| `ao/worker/E020-T01/E020-20260906-1418-0` | `b5beb0a` | docs(plan): close E018 — record run, widenings, browser pass, next s |
| `ao/worker/E020-T01/E020-20260906-1418-2` | `4934b30` | refactor(desk): inject the clock into the session engine |
| `ao/worker/E020-T02/E020-20260906-1418-2` | `f5ba9f8` | refactor(desk): move ergonomic limits out of session state |
| `ao/worker/E020-T03/E020-20260906-1418-0` | `4e5f471` | refactor(desk): split day break reset out of break credit |
| `ao/worker/E020-T04/E020-20260906-1418-0` | `825d15f` | refactor(desk): one versioned engine persistence snapshot |
| `ao/worker/E020-T05/E020-20260906-1418-1` | `5d1d3ba` | refactor(desk): engine owns policy-facing derived fields |
| `ao/worker/E020-T06/E020-20260906-1418-0` | `3a491b0` | test(desk): DTO-level scenario table for the session engine |
| `ao/worker/E020-T07/E020-20260906-1418-1` | `7734a21` | docs(desk): record the pure session engine as ADR 015 |
| `ao/worker/E020-T08/E020-20260906-1418-0` | `1f98831` | test(desk): record E020 full-suite evidence |
| `ao/worker/E021-T02/e021-run1-1` | `88af059` | feat(desk): add a source-agnostic health inlet with a Google Fit adapt |
| `ao/worker/E021-T03/e021-run1-1` | `ba84564` | feat(desk): add authenticated LAN health push inlet |
| `ao/worker/E021-T04/e021-run1-0` | `c3c2f3e` | feat(desk): show health from any source on both transports |
| `ao/worker/E021-T05/e021-run1-0` | `cbb4ebd` | feat(desk): add VoiceCapture dictation panel to the phone display |
| `ao/worker/E021-T06/e021-run1-2` | `225c612` | feat(desk): accept dictated voice notes from the phone |
| `ao/worker/E021-T07/e021-run1-0` | `f6dd5b1` | feat(desk): mirror sit-limit alerts to an ntfy/generic webhook |
| `ao/worker/E021-T08/e021-run1-2` | `e803c1c` | feat(desk): add a BYOK OpenRouter voice AI reply client |
| `ao/worker/E021-T09/e021-run1-0` | `0e9e0d6` | docs(E021): record the health inlet and voice path in ADRs 020/021 |
| `ao/worker/E021-T10/e021-run1-0` | `0ae3a10` | test(E021): record evidence for the epic's ten checks |
| `ao/worker/E022-T01/e022-run1-0` | `7166098` | feat(E022-T01): add the remote protocol contract |
| `ao/worker/E022-T02/e022-run1-0` | `3240810` | feat(E022-T02): add the relay Worker core |
| `ao/worker/E022-T03/e022-run1-1` | `ded0281` | feat(E022-T03): add relay credentials, pairing and REST |
| `ao/worker/E022-T04/e022-run1-0` | `7e3b67d` | feat(E022-T04): route relay commands to the desk and back |
| `ao/worker/E022-T05/e022-run1-0` | `3776ff3` | feat(E022-T05): add the desktop relay client |
| `ao/worker/E022-T06/e022-run1-1` | `e0be2df` | feat(E022-T06): add desk relay credentials, pairing and IPC |
| `ao/worker/E022-T07/e022-run1-1` | `cafda06` | feat(E022-T07): execute remote viewer commands on the desk |
| `ao/worker/E022-T08/e022-run1-0` | `898c7c6` | feat(E022-T08): put the phone display behind a transport |
| `ao/worker/E022-T09/e022-run1-0` | `59873d8` | feat(E022-T09): add phone pairing and remote controls |
| `ao/worker/E022-T10/e022-run1-0` | `2d216a1` | feat(E022-T10): add Settings Remote section with pairing and devices |
| `ao/worker/E022-T11/e022-run1-0` | `1e1b5c8` | feat(E022-T11): wrap LAN display messages in the v1 envelope |
| `ao/worker/E022-T12/e022-run1-1` | `dce19a6` | feat(E022-T12): wire relay recipes, dev proxy and local e2e |
| `ao/worker/E022-T14/e022-run1-0` | `cd6aea8` | docs(E022-T14): ADR 022/023 and the relay documentation |

## Kept — not merged (29)

| Branch | SHA | Date | Diff vs main | Why not merged |
|---|---|---|---|---|
| `ao/integration/E015-20260906-0425` | `294c7ad` | 2026-09-06 | 16 files changed, 127 insertions(+), 63 deletions(-) | integration branch of abandoned run; E015 promoted from run 0449 |
| `ao/worker/E014-T09/E014-20260906-2012-0` | `4aed56f` | 2026-09-06 | 3 files changed, 260 insertions(+) | superseded: another attempt of E014-T09  merged |
| `ao/worker/E015-T01/E015-20260906-0408-0` | `cd0851a` | 2026-09-06 | 21 files changed, 197 insertions(+), 112 deletions(-) | superseded: another attempt of E015-T01  merged |
| `ao/worker/E015-T01/E015-20260906-0425-0` | `b74c39b` | 2026-09-06 | 16 files changed, 127 insertions(+), 63 deletions(-) | superseded: another attempt of E015-T01  merged |
| `ao/worker/E015-T02/E015-20260906-0425-0` | `71094be` | 2026-09-06 | 38 files changed, 335 insertions(+), 116 deletions(-) | superseded: another attempt of E015-T02  merged |
| `ao/worker/E015-T03/E015-20260906-0408-0` | `c75c1cd` | 2026-09-06 | 10 files changed, 312 insertions(+), 13 deletions(-) | superseded: another attempt of E015-T03  merged |
| `ao/worker/E015-T03/E015-20260906-0425-0` | `ea13f07` | 2026-09-06 | 23 files changed, 430 insertions(+), 74 deletions(-) | superseded: another attempt of E015-T03  merged |
| `ao/worker/E017-T01/E017-20260906-0622-0` | `3ba151e` | 2026-09-06 | 6 files changed, 369 insertions(+), 247 deletions(-) | superseded: another attempt of E017-T01  merged |
| `ao/worker/E018-T02/E018-20260906-1209-0` | `b0a9d79` | 2026-09-06 | 44 files changed, 1571 insertions(+), 1279 deletions(-) | superseded: another attempt of E018-T02  merged |
| `ao/worker/E018-T02/E018-20260906-1209-1` | `31996df` | 2026-09-06 | 36 files changed, 2556 insertions(+), 2419 deletions(-) | superseded: another attempt of E018-T02  merged |
| `ao/worker/E018-T03/E018-20260906-1209-0` | `663ce98` | 2026-09-06 | 13 files changed, 54 insertions(+), 984 deletions(-) | superseded: another attempt of E018-T03  merged |
| `ao/worker/E018-T05/E018-20260906-1209-0` | `804603c` | 2026-09-06 | 4 files changed, 60 insertions(+), 10 deletions(-) | superseded: another attempt of E018-T05  merged |
| `ao/worker/E018-T10/E018-20260906-1209-0` | `8d0c49d` | 2026-09-06 | 10 files changed, 1015 insertions(+), 15 deletions(-) | superseded: another attempt of E018-T10  merged |
| `ao/worker/E019-T04/E019-20260906-0822-0` | `ce828b3` | 2026-09-06 | 2 files changed, 131 insertions(+), 77 deletions(-) | superseded: another attempt of E019-T04  merged |
| `ao/worker/E020-T01/E020-20260906-1418-1` | `645c94c` | 2026-09-06 | 18 files changed, 688 insertions(+), 202 deletions(-) | superseded: another attempt of E020-T01  merged |
| `ao/worker/E020-T02/E020-20260906-1418-0` | `d3c3704` | 2026-09-06 | 13 files changed, 222 insertions(+), 50 deletions(-) | superseded: another attempt of E020-T02  merged |
| `ao/worker/E020-T02/E020-20260906-1418-1` | `1253247` | 2026-09-06 | 14 files changed, 219 insertions(+), 58 deletions(-) | superseded: another attempt of E020-T02  merged |
| `ao/worker/E020-T05/E020-20260906-1418-0` | `67cab2d` | 2026-09-06 | 4 files changed, 152 insertions(+), 119 deletions(-) | superseded: another attempt of E020-T05  merged |
| `ao/worker/E020-T07/E020-20260906-1418-0` | `b4ad9f3` | 2026-09-06 | 8 files changed, 504 insertions(+), 16 deletions(-) | superseded: another attempt of E020-T07  merged |
| `ao/worker/E021-T02/e021-run1-0` | `621ed59` | 2026-09-07 | 19 files changed, 883 insertions(+), 147 deletions(-) | superseded: another attempt of E021-T02  merged |
| `ao/worker/E021-T03/e021-run1-0` | `d1b47fa` | 2026-09-07 | 16 files changed, 968 insertions(+), 29 deletions(-) | superseded: another attempt of E021-T03  merged |
| `ao/worker/E021-T06/e021-run1-0` | `cd73dcc` | 2026-09-07 | 21 files changed, 1766 insertions(+), 221 deletions(-) | superseded: another attempt of E021-T06  merged |
| `ao/worker/E021-T06/e021-run1-1` | `c017107` | 2026-09-07 | 23 files changed, 1525 insertions(+), 6 deletions(-) | superseded: another attempt of E021-T06  merged |
| `ao/worker/E021-T08/e021-run1-0` | `db43d4e` | 2026-09-07 | 8 files changed, 456 insertions(+), 1 deletion(-) | superseded: another attempt of E021-T08  merged |
| `ao/worker/E021-T08/e021-run1-1` | `470fce9` | 2026-09-07 | 8 files changed, 490 insertions(+), 1 deletion(-) | superseded: another attempt of E021-T08  merged |
| `ao/worker/E022-T03/e022-run1-0` | `baabfcc` | 2026-09-07 | 39 files changed, 3681 insertions(+), 1201 deletions(-) | superseded: another attempt of E022-T03  merged |
| `ao/worker/E022-T06/e022-run1-0` | `6b1d027` | 2026-09-07 | 22 files changed, 1174 insertions(+), 3 deletions(-) | superseded: another attempt of E022-T06  merged |
| `ao/worker/E022-T07/e022-run1-0` | `44b9948` | 2026-09-07 | 8 files changed, 846 insertions(+), 36 deletions(-) | superseded: another attempt of E022-T07  merged |
| `ao/worker/E022-T12/e022-run1-0` | `0cfe60b` | 2026-09-07 | 8 files changed, 491 insertions(+), 50 deletions(-) | superseded: another attempt of E022-T12  merged |
