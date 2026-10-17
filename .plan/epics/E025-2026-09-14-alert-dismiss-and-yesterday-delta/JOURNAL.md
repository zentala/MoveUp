# E025 Journal

## Source review round 1 — 2026-10-18

Independent review preflight is starting for T01 (5p) and T03 (8p), on branch `fix/alerts-yesterday-20261017`, after the ordinary commits `fix(alerts): dismiss without sensor ticks` and `feat(metrics): compare sitting with yesterday`. The author has stopped editing; the worktree is clean and the combined 44-file product diff passes ordinary whitespace checks. Source verdict is pending. This is the existing E025 campaign, with no separately imposed round ceiling; the older Analyst and Release campaigns remain unchanged.

The independent reviewer `alerts-yesterday-review` completed compatible unrestricted filesystem/test-output preflight and received source-review PROCEED. The reviewer owns the serialized test slot. An initial read-only process diagnostic returned exit 1; no product test or review verdict was inferred from it. The final preflight found no running Cargo/test conflict.

Author checks: `cargo test --manifest-path src-tauri/Cargo.toml --lib -- commands_poison` passed 3 tests; `$env:VITEST_MAX_WORKERS='4'; just check` exited 0 with 478 frontend tests in 57 files, 879 Rust unit tests and 16 Rust integration tests. Lint reported no errors and 16 warnings. The existing Rust binding exporter passed 2 tests; only TodaySummary has a generated content difference. Filter counts overlap and must not be summed as unique tests.

Retained failures: the initial Windows test executable exited 1 with STATUS_ENTRYPOINT_NOT_FOUND before running any test; this is not behavioral RED. The uncapped full check exited 1 after 135 frontend passes and 27 worker startup timeouts. A later full check had 478 frontend passes and 878 Rust passes with one stale poison-safe-lock-count assertion failure (Cargo 101, just 1), before the accepted 14-to-15 test correction. Negative controls were mutations: disabled dismiss listener (2 failures, 1 pass), omitted metric (3 failures, 5 passes), fabricated yesterday zero (2 failures, 6 passes); they are not exact historical-source executions. Earlier fixture and whitespace failures remain in author history.

Review covers actual event/callback behavior, exactly-once relay acknowledgement, optional query/cache data, all metric producers, nullable generated wire, UI tooltip/scenario, and Windows test linkage. Source review and synthetic tests do not establish the required real native alert flow or browser demonstration. Those gates remain NOT_CHECKED; no integration, source publication or E024 delivery is admitted by this entry.

### Round 1 result

Source correctness PASS: zero confirmed defects after full diff and surrounding-contract inspection. Independent checks exited 0: Cargo filters `e025_` (12), `metrics::` (16), `relay_commands_tests::` (25), `commands_poison` (3), and `ts_export::export_bindings_render_i64_as_number --exact` (1); focused Vitest for KpiStrip, deskReducer and useDesk (49 tests, 3 files, VITEST_MAX_WORKERS=4); typecheck and all three ordinary diff checks. Existing Rust warnings in unchanged session_tests_timers.rs remain. The writer exporter and full suite were not redundantly repeated by the reviewer. The candidate remained clean and no test job remained.

Acceptance evidence finding R1-1: Medium, 3p, unresolved/NOT_CHECKED. PLAN's exact reverted-source negative-run requirement is not proved by the supplied mutations or zero-test loader failure. The DM requested a bounded read-only feasibility assessment from the same author and reported the gap to PO; no acceptance override or new verification framework. Browser demonstration and real native alert flow are still separate missing gates.

Diagnostic exit 1 during preflight came from a no-match rg after a successful process inspection. Two subsequent source-read commands used nonexistent filenames and returned 1; later reads found the actual owners. These are retained read errors, not failed product tests. A compound check command inherited a later read error; each ordinary diff check was separately rerun and passed.

### Browser observation and proportional follow-up

The same independent reviewer checked the existing synthetic mockup gallery in dedicated automation Chrome. At 1440x1000, primary omitted the new badge; secondary showed `vs yesterday / −25 min` with the expected green value/border and no clipped text. Actual hover and the full explanatory title attribute were observed. This is PARTIAL: two screenshot writes were rejected by the MCP output-path boundary, so no image was produced. The all filter, return to primary, narrow viewport and native tooltip pixels were not checked. Console retained a development HMR proxy websocket error and favicon 404; the document/modules loaded and no real backend/relay request was observed. The reviewer closed its QA tab; DM stopped the owned Vite preview before any source mutation.

PO's dated PLAN/HANDOFF disposition retires the blanket per-test historical reconstruction requirement and admits exactly three compatible old-body checks after browser QA releases the source. T01's listener-disabled result remains a mutation, not historical RED. Required native no-sensor/cooldown behavior and final independent acceptance are unchanged. The same author is entering fresh three-path preflight; no result is implied by this authorization.

## Finding 2026-09-14 — epic created
→ Planned from the E001-E009 audit leftovers (E004-T05, E004-T04, E001-T12) on request; BACKLOG entries replaced by a pointer.
