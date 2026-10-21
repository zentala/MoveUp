# E027 — trustworthy core, chronology and Analyst delivery

## TLDR

Implement the complete accepted 2026-10-21 core review and the existing undelivered Analyst outcome in ONE owning queue. Deliver trustworthy observed intervals and credited load, one configurable notification decision, compatible persistence/wire semantics, shared day history and corrected expandable details. Reuse preserved candidates before writing. This serves VISION's understandable, configurable ergonomics experimentation platform and the PURPOSE direction of completing useful delivered products rather than accumulating branches.

Accepted implementation authority is recorded in ../../po/core-implementation-authority-2026-10-21.md; final findings in ../../reports/2026-10-21-core-logic-review.md. This replaces the earlier report-only boundary. No further operator approval is required for routine implementation choices within this scope.

## Problem and current state

The delivered main contains source-confirmed interval loss after long gaps, active-state errors at local midnight and duplicate credit, plus inconsistent hourly/debounce contracts. Alert decisions are split between policy and a second sender; Silent, built-in reminders, popup text and settings controls diverge. Analyst loses raw/credited/local-date/quality/counter distinctions. The review's independent validator corrected overclaims and rejected a local-DST hypothesis because current buckets use UTC. Its passing table test deliberately reproduces a faulty one-second raw total after forty observed minutes.

Startup/Store/tray reliability source is already pushed and one installed app is healthy under PM3. This does not mean earlier Analyst candidates are integrated. Preserved backend, frontend sequence, browser support and dirty UI work must be compared with current main before changes. Their histories, failed checks and exact denied operations remain intact.

## Scope

In scope: all thirteen confirmed bugs/contract mismatches; one alert owner; interval, intent/router and wire refactoring; channel validation and injected policy time; quality/Away/Unknown; loading/error/empty/stale; compatible checkpoint/restart and bounded silence persistence; accepted undelivered timeline/day composition and signed-score/recorded-credit details. Unconfirmed items get discriminating source/synthetic investigation followed by the architectural decisions in ADR015. The rejected local-DST hypothesis is not an implementation task.

Out of scope: full rewrite, cross-product package, new framework/daemon/admission system, historical backfill, deletion/repair of live rows, machine permission changes, PM3 supporting-tool repair, or claiming medical effectiveness. No replay of refused conhost/script execution, pm3d72H mutation or previously denied patch merely because source scope is now accepted. Existing actual refusals bind; fresh denial stops its operation while independent accepted work continues.

## Architecture before decomposition

Existing ADR015 (2026-10-21 appendix) is controlling: observations capture clock/quality; engine closes observed intervals and separately applies credit; metrics define units/local buckets; policy resolves one intent; executors perform side effects; persistence owns versions/acknowledgment/checkpoints; wire preserves semantics; UI renders domain outputs and uncertainty. ADR008 proportional formula and ADR010 configurable escalating silence remain, with explicit unknown-gap and repeat compatibility decisions in ADR015.

Reuse React/Tauri/Rust, existing Store/SQLite, generator, Recharts and browser tooling. No new framework or shared package. Pattern selection: how-we-build #6 disposable bounded comparison, #12 missing information must not be success; apply current Layout/Interface/Boundary/Toolchain conventions. Existing review commands proved pnpm forwarding can accidentally widen a suite; use actual supported test selection and count executed tests. Zero tests is NOT_CHECKED.

## Implementation strategy

### Full estimate and execution route

Full total **115p**, including preserved original47p, previously disclosed provenance5p + selected-day3p, and added core/review implementation60p. This is a full-scope estimate, not remaining effort and not a reset of accepted checkpoints. Do not double-count a fix implemented by a reused candidate; close by evidence without silently shrinking the controlling route.

**Approved route: DIRECT Herdr for the ENTIRE115p**, by explicit operator exception2026-10-21: "Tak — wykonaj zwykłymi subagentami". The >40p policy would select AO; the approved exception preserves that full total and its historical selected-route conflict. Model for bounded workers: actual supported GPT-6-Luna HIGH, verified before admission. At most3 active worker/reviewer slots, usually fewer; one Cargo/shared test slot. No agent work timer by default.

DM source inspection established that the selected local AO route requires rejected context digest/pin/verify and automatic verifyAndMerge. It was not executed. The operator explicitly selected ordinary supervised subagents instead; no capability repair is requested. Do not operate rejected hash-based context/verification, invent an unlimited deadline, treat `autoMerge:false` as a proven pause, or revive old Harness repair. DIRECT execution is now admitted by the actual operator answer, not inferred from the old sequential exception. No AO readiness gate or compatible-AO dependency remains active for this plan; preserve the historical conflict without operating it.

### Waves and final outcomes

| Wave | Tasks | Total | Observable outcome | Done / not done |
|---|---|---:|---|---|
| 1 | T08 interval boundary13, T06 individual history8, T16 provenance5, T09 time contracts8 |34p|Observed prefixes survive gaps/midnight, credit is exactly once, individual spans are acknowledged, exact/legacy provenance is honest, debounce/hour metrics follow explicit time.|Not done; compare preserved backend first.|
| 2 | T10 unified intent8, T11 profile/clock semantics8, T12 effective settings5, T13 restart durability5 |26p|One notification decision, real Silent/repeat schedule/valid channels, effective controls, bounded compatible checkpoint and silence restore.|Not done.|
| 3 | T14 producer/wire5, T02 data/loading5, T03 shared chronology8, T17 selected day3 |21p|Raw/credited/local-day/quality/counter survive real wire, truthful initial/refresh states and one compact/detail chronology/day handoff.|Not done; preserved data/timeline candidates are inputs.|
| 4 | T01 design5, T04 connected day composition8, T07 signed details8, T05 integration/browser5 |26p|Accepted charcoal shared UI, timeline with seven connected day tabs, full local-day denominators, corrected expandable signed score/recorded credit, independent actual-host wide/narrow acceptance.|Not done; preserve dirty UI/browser support.|
| 5 | T15 release/runtime8 |8p|Reviewed source pushed, real installed release, real save/Quit/recovery/logs/health, exactly one healthy app under PM3 and truthful remaining login/dark/stop limits.|Not done.|

Tasks within a wave follow explicit data dependencies and exclusive claims; wave grouping is not blanket parallel dispatch. No wave exceeds70p. Sequential source collisions serialize. Existing earlier task receipts/statuses remain evidence, not proof of current integrated completion.

### First useful implementation

After actual candidate comparison and fresh effective preflight, T08 (13p after actual adapter necessity) is the first source slice: preserve observed raw time at a long-gap boundary, split active Sitting/Standing at local midnight, and apply break credit once. Start with positive credited load and existing table regression that currently erases40minutes, plus gap/standing/return and midnight/DST cases. Reuse T06 candidate functions where they already meet the contract, without importing unrelated source/admin changes. Do not change live data or process. Exact absolute write paths and effective filesystem profile precede writes; a source-ready plan is not an actual launch receipt.

### Compatibility, verification and rollback

Each coherent group: retain a discriminating failing regression on old behavior, implement/reuse, targeted tests with real counts, required `just check`, independent actual-source review/corrections, normal integration/push. Serialize checks; no weakening coverage/assertions or repeating green suites without a new change/concern. New regression tests may change source only within admitted exact paths. Keep original failed exits and distinguish loader/zero-test failures from behavioral RED.

Use old/new profile/Store/wire fixtures, nullable/unknown/malformed history, clock rollback, local23h/25h/midnight, sensor outage, rapid/irregular readings, popup/silent/repeat/dismiss/reload, same-count query changes, delayed/error/empty responses. UI reads DESIGN before code, records new decisions, uses interface-design for substantial work and scoped design-review plus independent actual-host screenshots wide/narrow after implementation; passing synthetic fixture alone is not visual acceptance.

No rewrite of old history. Nullable provenance is new-producer-only; generated bindings preserve defaults. Persisted versions are additive/read-compatible and refuse unsupported future shapes. State checkpoint at existing minute cadence bounds crash exposure when IO succeeds; do not fabricate stopped time. Installation uses existing record/backup/rollback and ordinary supported lifecycle operations; refused activation is not replayed. Keep old release/config until new healthy instance verified.

## Execution contract — approved Direct Herdr

Identity: E027 core continuation2026-10-21, repository MoveUp, base main. Execution: approved DIRECT Herdr for full115p, sole existing DM; max3 active workers/reviewers and one shared test slot. Actual GPT-6-Luna HIGH and fresh effective exact absolute-path access precede author writes. Each group requires discriminating regressions, targeted checks, just check and independent actual-source review/corrections BEFORE normal integration/push. No author self-certification. Historical AO selection/conflict retained as evidence only; no manifests/ledgers, hashes, fake adoption, work timers or new daemon are required or authorized.

Every admitted task binds an exact relative write_set, narrow exclusive claims, ordinary reference/handoff and actual effective executor/profile evidence. Unknown scope is returned before writes. Implementation does not commence through a route that cannot satisfy current controls. Output: status, changed paths, commit/branch/worktree, tests with counts/exits, independent verdict, UI/native evidence or NOT_CHECKED, compatibility/rollback, blockers/gaps. No new collector or receipt system.

## Definition of done

Observed time conserved on known boundaries; credit applied once; no unknown interval fabricated as posture/Away. Local-day aggregates agree through backend/wire/UI. Policy channels/settings/silence actually match one decision and configured repeats. Older snapshots/history remain safe and uncertain where appropriate. Popup/Analyst share chronology and colors, load truthfully, expose full-day composition and retained signed details. All accepted source groups pass regressions/just check/independent review, UI passes actual-context wide/narrow review, normal verified commits pushed, final release actually running as one healthy PM3 instance. Actual login/dark observation and Windows force-stop limitations remain separately stated unless actually resolved; do not call unrelated prior goals done.

## Subtasks

Existing T01–T07 retain original IDs/points47 and historical acceptance; this plan supersedes obsolete active AO/hash/timer/no-push/publication recipes in old text. Task files T08–T17 define new outcomes and interfaces. Previously approved provenance/day work is explicitly accounted as T16/T17, without discarding existing source. Agent capability ts-dev (Rust/Tauri/TypeScript as needed), execution Codex GPT-6-Luna HIGH; independent verifier distinct from author. Historical Analyst campaign4/6 remains retained for its old contract; no silent count reset or reinterpretation as complete. Current expanded work follows no default review cap, without reopening any explicitly exhausted old campaign.

## Risks and GAPS

Direct route is explicitly approved. Exact reused-candidate compatibility remains a source concern to test, not a new route admission condition. Candidate comparison and current dirty ownership are DM prerequisites, not permission to overwrite. Meaningful active runtime and actual UI context must be preserved. Additional IO/version/live migration needs a finite concrete disposition before action; no synthetic PASS stands for native effect. Refused operations remain held. Independent verification can uncover further necessary work; disclose changed task estimates and keep one queue. Current E025/E024 unrelated accepted work remains preserved, not swallowed or marked done by this continuation.

## Final readiness and route checkpoint — 2026-10-21

Full17 task points were independently summed from frontmatter:115p; five wave totals34/26/21/26/8 agree. First engine scope is source-qualified exactly11 existing paths with no wiring/additions, reuse and pure-clock constraints in T08/HANDOFF and DM comparison note. Selected local AO incompatibility is source-established, not a global absence claim; rejected machinery was NOT invoked. The sole T3 route question was actually answered: "Tak — wykonaj zwykłymi subagentami". Full-scope Direct Herdr is approved; no AO readiness gate or operator-owned route wait remains. The complete plan is admitted for execution through sole DM, first task requires fresh effective preflight before writes. Author activity is recorded by actual uptake, not assumed from this approval.


## Current controlling disposition — 2026-10-21

The operator replied to the ONE T3 route question: "Tak — wykonaj zwykłymi subagentami". Pending route is RESOLVED. Full115p and all scope/refusal boundaries are unchanged. Direct Herdr is approved for all waves; T08 exact11paths released now to sole DM. Any earlier pending/selected-AO preparation text in the dated checkpoint is historical, not an active prerequisite. No further plan/audit/consent before the ready first author preflight and regressions.

## Actual first-slice test necessity — 2026-10-21

T08 retains8p and full115p unchanged. Add one existing test-only session_tests_flush.rs after actual Rust101/fullcheck1, frontend470PASS. Current first scope is12paths (original11+test1); known100+50 break excludes700unknown, midnight observed fixtures verify split totals and delayed reset pending/persisted known50 exactly once. No additional product paths or new decision needed; independent review checks corrected oracles.


## Source-qualified T08/T06 integration dependency — 2026-10-21

The first T08 source slice depends on the retained T06 observation anchors, pending-history acknowledgment and cumulative-credit state; it is not independently integratable into main. Author tests/frozen diff and T08 source review do not accept the old whole branch or prove composed runtime compatibility. Collect the current independent T08 verdict first. Then sole DM supplies the finite T06 composition handoff focused on session_persistence.rs, serial_periodic_reset.rs, app_lifecycle_runtime.rs and the direct serial_periodic.rs acknowledgment consumer, with only demonstrated necessary test/wiring changes identified before writes. Preserve delivered AppStore and prepare_shutdown/cancel semantics; never overwrite them with the older lifecycle implementation. Reuse compatible source through ordinary diffs and keep all candidate histories. Current read-only assessment grants no new author writes. The coherent composed T06/T08 group must pass meaningful acknowledgment/failed-write/retry/exactly-once and shutdown-cancel regressions, required checks and independent actual-source review before main integration/push. The underlying dependency is part of the accepted110p, not a new tooling prerequisite or route question. Task completion distinguishes a source-accepted slice from an integrated accepted outcome.

## Current finite production-order scope — 2026-10-22

PO FULL-read DM source-qualified amendment and accepts exactly serial_periodic.rs, serial_periodic_reset.rs, serial_periodic_tests.rs in the same first task; current15paths, T08 High13p, total115p, waves34/26/21/26/8. Rationale for13p: one indivisible engine/actual-adapter boundary invariant with exclusive tests, not an independently releasable engine-only fix. Explicit pre-reading new-day reset must precede transition/persistence with one captured instant, without running minute notifications/snapshots every tick. serial.rs excluded; no Store/live changes. This is routine finite implementation necessity within the approved full-report scope and Direct exception, not renewed route or product approval. Earlier110p/12path/8p facts are retained history, not current totals.


## Current T08 acceptance and next coherent review — 2026-10-22

Independent T08 round2 PASS accepted as SOURCE SLICE only, with22 target tests and required worker4 wholecheck. Ordinary scoped local commit preserves reviewed source; not main integration/runtime acceptance. Next finite main-based T06 composition preserves current AppStore/prepare_shutdown/cancel, excludes later T16 schema/wire, and requires failedwrite/retry/exactlyonce/cancel regressions plus source review before integration. Review accounting is explicitly reconciled in review-rounds.md: ONE upcoming composed verdict evaluates original T06 and expanded core, prospective original Analyst5/6 plus expanded-core3, counted only on actual review admission; old4/6 and T08 round1FAIL/2PASS unchanged until then. No duplicate reviewers/suites, implicit resets or new approval gate.


## Actual T06 composition release — 2026-10-22

The finite main-based composition handoff is FULL-read and accepted: taskT06 frontmatter now lists26 literal existing source/test paths. Only two test-only additions outside prior accepted T06/T08 scope: app_store_tests.rs and app_lifecycle_runtime_tests.rs. Same8p T06/13p T08/full115p; current main AppStore/prepare_shutdown/cancel preserved, laterT16 schema/marker/wire excluded. Sole DM establishes ordinary owned main-based target, same saved actualLunaHIGH author and fresh effective absolute26path preflight then PROCEED, without another operator decision. Full failedwrite/partialwrite/retry/crashbeforeack/exactlyonce/cancellation-ownership/localday acceptance is in T06 and owning finite handoff. Current healthy installed app is untouched. Required checks and one independent composed review precede integration/push; historical failures/counts and actual refusals preserved.
