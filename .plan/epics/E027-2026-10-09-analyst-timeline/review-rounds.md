# Analyst source review campaign

Authority: operator-requested investigation, fixes and independent Herdr verification; current shared review-loop maximum six automatic rounds. Campaign covers the accepted Analyst correction and compatible data/timeline work. Historical command failures remain in JOURNAL and private delivery evidence. No earlier corrective review rounds consumed.

## Round 1 — data candidate

- Candidate: `45d3fac1097d2a1af69f09609cd79637c4d9fad4`.
- Independent reviewer: timeline worker, who did not author data candidate; source checked read-only. Coordinator adjudicated against actual source.
- Confirmed: unknown state coerces to Sitting and connection metadata is absent (High, 5p); missing score becomes zero (Medium, 3p); incomplete session endpoints/duration lose uncertainty (Medium, 3p); transition counter description omits break resets (Medium, 3p); signed unbounded score displayed as /100 (Medium, 3p); first-load false empty and stale refresh presentation (High, 3p).
- Evidence: `useSnapshotsRange.ts:26,45`, `explorer-derivations.ts:23,34`, `commands_analyst.rs:22`, `snapshot_logger.rs:25`, `session_reading.rs:134,208`, `session_daily.rs:115`, `explorer-day-kpis.ts:106,121`, `KpiDonutPanel.tsx:28,59,68`, `ExplorerTab.tsx:55,87,161` in the candidate.
- Source coverage passed for raw standing denominator, local initial date/storage validation, same-count updates and late-response rejection. These do not close the confirmed findings.
- Disposition: HOLD integration. Data-quality corrections return to implementation after existing queue reconciliation. Presentation findings enter corrected UI acceptance, avoiding duplicate tasks. Native behavior NOT_CHECKED.
- Remaining automatic rounds: 5. Round 2 not yet dispatched. Candidate tests are author evidence, not independent acceptance.

## Round 2 — backend candidate

- Independent native Herdr verifier inspected frozen5798d2d against parent6856edd; source remained clean. Basic mixed-history in-memory probe passed (five spans200/20/30/40/50 seconds, total340, raw standing60, credited1920).
- Confirmed source findings: backwards readings can overlap subsequent history (High,5p); sleep gap credited twice during a break (High,5p); local-day queries omit intersecting previous-day spans (High,5p); failed inserts discard unacknowledged spans (High,5p); legacy mixed-break provenance is not identifiable (High,3p).
- Executed counterexamples rollback, sleep_credit, midnight and persistence each failed exit101; these failures remain retained. Author225 passing Rust tests do not close them. Native lifecycle/runtime remains NOT_CHECKED.
- Disposition HOLD. First four findings return to backend correction in the same epic. Legacy uncertainty remains cross-task acceptance owned by shared timeline T03; do not add guessed provenance or mutate historical rows. A backend source correction alone cannot close that criterion.
- Remaining automatic rounds:4. Frontend source still awaits independent review. No main source integration or push.

## Round 3 — corrected data candidate (FAIL)

- Frozen frontend candidate9b9a17371cafed4d7ad2e282745a8e46c4afc51c;
  review covers the full T02 data change and retained round1 findings, not only
  the final four-file correction. Author156 frontend tests/typecheck passed;
  earlier failed checks remain retained and do not certify source acceptance.
- Two rounds consumed, round3 reserved, three further rounds after this attempt.
  Backend remains moving and outside this verdict unless separately frozen and
  explicitly supplied before review closes. Legacy timeline provenance remains
  outstanding T03 scope. No implicit acceptance of other tasks or native UI.
- Independent reviewer receives source-read-only scope, normal frontend checks
  and explicit filesystem preflight. Failed dispatch is NOT_CHECKED and consumes
  this reservation. No integration/publication under this reservation alone.

- Completed frontend-only at9b9a173: one confirmed High/3p loading-state defect.
  ExplorerTab renders KPI no-data when sessions resolve empty before snapshots
  resolve or while snapshots fail. Source evidence: ExplorerTab.tsx:162 and
  KpiDonutPanel.tsx:64. Add response-order regressions and preserve independent
  query error/stale behavior. DM independently confirmed the render condition.
- Independent typecheck exit0; full Vitest with normal vite.config.ts and
  maxWorkers1 failed startup exit1 (esbuild spawn EPERM, errno -4048), zero tests.
  Execution remains NOT_CHECKED; no escalation or permission change occurred.
- Backend1d27595 was supplied during reporting, but the final verdict explicitly
  excludes it. Only backend HEAD/clean inspected; no backend source acceptance
  or Cargo execution. Next review must name its frozen source explicitly.
- Three rounds consumed, three remain. Correct the confirmed frontend defect,
  then reserve the next round on the corrected frontend and frozen backend.
  Pending T03 provenance/Unknown rendering and T07 trajectory signs/local-time
  remain separate task dependencies, not accepted by this review.

## Round 4 — frozen frontend and backend (reserved)

- Prospective scope: full corrected T02 data candidate86a3204c598ab8f8aab59ba19249daa8522b8527
  including the round3 async-loading correction, and full T06 backend candidate
  1d27595deedde1f50732a03182ebd70cc05da34e including the accepted test-fixture extension.
  Both exact HEADs and clean trees independently inspected before dispatch.
- Review all earlier in-scope source findings and meaningful regressions; source
  read-only, generated test outputs permitted only after compatible effective
  filesystem preflight for both worker roots. No permission escalation, GUI,
  live data, integration or publication. Native gaps remain explicit.
- Three rounds consumed, round4 reserved, two further rounds after this attempt.
  A missing/failed review is NOT_CHECKED and consumes the reserved attempt.
  No retroactive change to frontend-only round3, task split or campaign reset.
- Author evidence: frontend typecheck0, targeted164/21files0, full504/57files0;
  backend filtered225/12/8/47/6/14 all0, overlapping counts not summed. Earlier
  failed tests and report-write refusal retained. These are not independent acceptance.
