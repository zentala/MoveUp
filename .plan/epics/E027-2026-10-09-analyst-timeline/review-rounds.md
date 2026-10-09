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

## Round 4 — frozen frontend and backend (functional PASS; hygiene findings retained)

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

- Final independent verdict: T02 and T06 source/functionality PASS, execution
  PASS, no blocking defect in the accepted contracts. Exact candidates unchanged
  and both trees clean after checks. Four rounds consumed, two remain.
- Independent serial execution: frontend typecheck exit0; full Vitest504/57files
  exit0; backend full Rust library869/869 exit0; additional first frontend Rust
  commands_analyst filter16passed/839filtered exit0. The last check verifies that
  branch's optional query fields; backend tests do not certify frontend additions.
- H1 Low/1p retained: cumulative frontend CRLF in explorer-day-kpis.test.ts,
  explorer-day-kpis.ts, explorer-derivations.ts and hooks/useSnapshotsRange.ts.
  H2 Low/1p retained: correction-manifest.json and backend-correction-manifest.json
  CRLF; backend has no flagged Rust source. All flagged blobs are unchanged by
  final commits versus their direct parents; H1 still belongs to full T02 scope.
  No trailing spaces/tabs after line terminators are removed. No normalization or
  Git configuration changes performed. Both original broad diff-check command
  exits1 remain FAILED; separate raw Git diagnostic exit2 does not replace them.
- These hygiene findings are nonblocking for functional acceptance, not a clean
  hygiene verdict. Legacy height-chart empty presentation belongs to accepted
  T04 removal; no shared loading regression found. T03/T04/T07 and native/browser,
  real-store restart, integration and publication remain outstanding.
- Reviewer receipt path references were corrected against frozen source before
  final closure; source never changed. No new round or repeated runtime suite.

## Post-round4 AO gates and isolated dependency integration

- First verify-only gates on the accepted pair both FAILED, CLI exit2, solely
  for six historical administrative paths outside the current write sets.
  Frontend2/2 and backend3/3 declared commands exited0. The administrative trees
  matched their integration targets; separately materialized commits appeared
  in AO's triple-dot audit. No product-source or completeness cause was reported.
- Narrow metadata reconciliation b1817db added exactly those six historical
  allowances to the two frozen task contracts. Source write sets and claims,
  worker HEADs, original context snapshots and prior failures were preserved.
  No new implementer metadata rights or source replay was granted.
- One corrected verify per run: both gate PASS with exact reviewed sourceHead;
  frontend2/2 commands exit0, backend3/3 commands exit0. Aggregate CLI exit2
  retains historical attention, not a failed current gate. No review-round reset.
- Explicitly admitted frontend-only AO merge produced
  6d22d73875c85f8e1baf837a8312ed17dae74510 on
  ao/integration/E027-20261009-correction. Merge verdict merged, basis gate,
  aggregate exit2; cleanup pending, promotion not ready. Candidate ancestry
  proved and the entire resulting tree equals accepted frontend86a3204:
  tree13630b3fc3d838737632da5ba1aaeae516576db3. Worker status merged and
  verification.merged=true satisfy T03's T02 dependency. Backend remains
  unmerged in its separate worker; no main source integration or publication.
- Future author dispatch remains held pending a supported source-only
  materialization/preflight route. Current AO run/resume reaches verifyAndMerge
  without enforcing execution.autoMerge=false as an independent-review pause.
  No uncontrolled resume, fake failure/result, runtime edit or approval override.
  Final UI, combined runtime, browser/native and delivery remain NOT_CHECKED.
