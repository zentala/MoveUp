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
