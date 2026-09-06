---
formatVersion: 1
type: epic
status: todo
readiness: ready
points: 45
agent: ts-dev
wave: 4
parallel: [E019]
depends-on: [E015]
blocked-by: ""
---

# E018 — Frontend consolidation

Status: planned (2026-09-06).
Source reviews: [`../../reports/_review-2026-09-06/frontend.md`](../../reports/_review-2026-09-06/frontend.md),
[`../../reports/2026-09-06-pelny-przeglad-architektury-i-release.md`](../../reports/2026-09-06-pelny-przeglad-architektury-i-release.md)
§3 Frontend, §6 E018.
Board deck (Polish): [`PRES.md`](PRES.md). Handoff: [`HANDOFF.md`](HANDOFF.md).

## TLDR

The frontend works (73 source files, 37 test files, 248 green tests) but
carries a previous generation's debris: `useDesk.ts` and `useRemoteDesk.ts`
reimplement the same state machine twice (one of them 0.78% covered), Rust
DTOs are hand-copied into `types.ts` with no drift signal, five pre-OneBar
components and a dead `overlay.html` build entry are still compiled, `pnpm
lint` calls an `eslint` binary nobody installed, and the 80% coverage gate
in `vite.config.ts` has never been wired into a build. This epic closes
those seams — one reducer with two transport adapters, generated Rust→TS
types, dead code removed, real lint and a real coverage gate, one
formatting module, and two docs (`overlay.md`, `UX-FLOW.md`) brought back
in line with the code — then finishes three UI-polish tasks E012 left
unstarted. 45 points across three waves. **Depends on E015** (lands the
`limit_used_secs`/`current_session_secs` rename this epic's reducer must
carry forward) but does not repeat E015's work.

## Problem

- `src/hooks/useDesk.ts:37-247` (Tauri IPC) and `src/hooks/useRemoteDesk.ts:43-252`
  (WebSocket) hand-duplicate the same state shape, the same
  `previousSession`/`todaySessions`/`breakResetProgress` derivations, and the
  same transition-timer logic. Every new field (e.g. `continuous_computer_secs`,
  ADR 011) gets hand-edited into both. `useDesk.ts` has 0.78% statement
  coverage because the unit-test environment cannot fake a live Tauri
  runtime; `useRemoteDesk.ts` sits at 94.6% purely because `WebSocket` is
  fakeable in jsdom. (`frontend.md` findings #3, #4.)
- `src/types.ts:59-89` and `src/components/settings/SettingsTypes.ts:9-25`
  hand-mirror `src-tauri/src/session_types.rs` / `config.rs` field by field,
  with no codegen and no compiler signal when a Rust field is renamed or
  added. (`frontend.md` finding #5.)
- Five components from the pre-`OneBarWidget` era —
  `AppProgressBar.tsx`, `HeightRail.tsx`, `SessionProgress.tsx`,
  `TodayStats.tsx`, `TransitionBanner.tsx` — are unreferenced outside their
  own tests, last touched March 2026, and their tests still count toward
  "248 passed" and coverage. `src/overlay/main.tsx` + `overlay.html` is dead
  debug scaffolding still built by `vite.config.ts:38`; the real overlay is
  the native WinAPI window in `overlay_renderer.rs` per
  `.claude/rules/overlay.md`. (`frontend.md` findings #6, #7.)
- `package.json:28` wires `pnpm lint` to `eslint src/`, but no `eslint`
  devDependency and no config file exist anywhere in the repo — the command
  fails immediately, so lint has effectively never run.
  `vite.config.ts:22-31` defines a real 80/80/75% coverage threshold, but
  `pnpm build`/`pnpm tauri:build` call `test:all` (`test:unit && test:unit:rust`)
  and never `test:coverage` — the gate has never blocked a build, and
  measured coverage today (76.5/68.1/66.0%) is under it on all three axes.
  (`frontend.md` findings #1, #2.)
- Six local `formatXxx` helpers duplicate the "format a date/duration"
  concern outside `src/utils/format.ts`: `formatDate`
  (`TimelineDetailHeader.tsx:28`), `formatRefreshedAt`/`formatRangeLabel`
  (`ExplorerTab.tsx:44,54`), `formatIdleTime` (`StateIndicator.tsx:24`),
  `formatTime` (`OneBarTimeline.tsx:37`). (`frontend.md` finding #9.)
- `.claude/rules/overlay.md` documents `scripts/tauri-dev.sh` and `pnpm
  tauri:dev --force`; the real launcher is `scripts/tauri-dev.ps1 -Force`
  and `package.json`'s `tauri:dev` script has no argument pass-through.
  (`frontend.md` finding #15.) `.arch/UX-FLOW.md` was last touched
  2026-05-16 and has zero mentions of the Google Fit `StepsWidget` (mounted
  2026-05-18/19/21) or the click-timeline-to-open-Analyst interaction
  (2026-05-21). (`frontend.md` finding #8.)
- E012 left three task files unstarted:
  [`E012-T09-analyst-layout-flip.md`](../E012-2026-05-16-analyst-dashboard/tasks/E012-T09-analyst-layout-flip.md),
  [`E012-T10-recharts-donut-kpis.md`](../E012-2026-05-16-analyst-dashboard/tasks/E012-T10-recharts-donut-kpis.md),
  [`E012-T11-selected-day-pulse.md`](../E012-2026-05-16-analyst-dashboard/tasks/E012-T11-selected-day-pulse.md).
  E012's own `ORCHESTRATOR.md` is not edited by this plan; E016/E018 closes
  E012 (noted for whoever runs E016's close-out ceremony).

## Decisions and ADRs

- `.plan/decisions.jsonl` — created by E015-T04 (D1 multiplier, D2 delete
  `current_session_secs`). If E015 has not landed it yet when this epic
  starts, T02 creates the file. This epic appends:
  - **D3** — Rust→TS codegen library: **ts-rs**, not specta (see
    Alternatives). Recorded in `.plan/decisions.jsonl` and ADR 017.
  - **D4** — coverage threshold policy: meet 80/80/75% if T01's reducer
    work gets there; otherwise lower the threshold in `vite.config.ts` to
    the measured number with a dated comment and a `.plan/BACKLOG.md` entry
    to raise it back, rather than leaving a gate that silently never ran.
- ADR 008/009/011 (break credit, day reset, unified cycle) — unchanged;
  E015 owns their revision.
- **New ADR 016** — `recharts` for KPI donuts (T10; number pre-claimed by
  `E012-T10-recharts-donut-kpis.md`'s own text, so T10 keeps it rather than
  renumbering that source task).
- **New ADR 017** — ts-rs vs specta for Rust→TS codegen (T02).
- This epic does **not** duplicate E015's DTO drift test. E015-T02 adds a
  manual JSON-fixture key-set check between a Rust test's
  `serde_json::to_string` output and `src/types.ts`. E018-T02 supersedes it:
  once fields are generated, a renamed/removed Rust field is a TypeScript
  compile error, which is strictly stronger than a runtime key-set diff.
  T02 must locate and delete E015's manual drift test file (grep for
  `dto-drift` or similar under `src/`) instead of running both checks
  side by side.

## Alternatives

| | A. Minimum | B. Target architecture | C. Rewrite state layer |
|---|---|---|---|
| Summary | Fix only the two loudest tooling gaps: install eslint, wire the coverage gate into build. Leave the hook duplication, hand-copied DTOs, and dead code in place. | Reducer + two transport adapters, Rust→TS codegen, delete 5 dead components + overlay entry, real lint, real coverage gate, one format module, two docs synced, plus E012's 3 leftover UI tasks | Replace `useDesk`/`useRemoteDesk` with a state library (Zustand/Redux Toolkit) instead of a hand-rolled reducer, and generate a full Tauri command binding layer (tauri-specta) instead of DTO-only codegen |
| Effort | S | M | XL |
| Risk | L | M | H |
| Pros | Ships in under a day; unblocks CI-shaped confidence quickly | Closes every seam the review found without discarding 248 green tests; each task is independently verifiable | Cleanest long-term state model; specta could also generate command signatures, not just DTOs |
| Cons | The hook duplication and DTO drift keep costing double edits on every future field; dead code keeps padding coverage/test counts | Touches more files than A; requires sequencing (reducer before codegen, codegen before UI polish) | Discards a working, tested pattern for a new dependency and a new failure mode; specta's Tauri-command generation is a larger architectural commitment than this epic's DTO-mirroring problem calls for; re-risks the E015 fix by moving state logic mid-migration |
| Reuses | Existing hooks and tests unchanged | `useWidgetData`, `WidgetProps`, `scenarios.ts`, existing 248 tests as the regression net | Little — new library, new patterns to learn |

**Recommendation: B.** The review is explicit that this is not a rewrite —
"the shape is mostly right" (`frontend.md` §Recommended target structure).
A leaves the exact duplication that made `continuous_computer_secs` a
double edit; C reintroduces migration risk right after E015 closed one
(nine March rewrites of the same seam, per `plans-and-history.md` §2) and
picks up a dependency (specta / a state library) this problem does not
need. B matches `frontend.md`'s own six-step recommended structure and lets
each task be verified independently.

## Scope

**In:** `src/hooks/useDesk.ts`, `useRemoteDesk.ts`, `useDeskTypes.ts`, a new
shared reducer module; `src/types.ts`, `src/components/settings/SettingsTypes.ts`,
`src-tauri/Cargo.toml` and its DTO-bearing structs (`session_types.rs`,
`config.rs`, `commands.rs`, `commands_analyst.rs`); the 5 dead components +
`src/overlay/main.tsx` + `overlay.html` + `vite.config.ts`'s build input;
`package.json` (lint, coverage, build scripts), a new `eslint.config.mjs`, a
new `justfile`; `src/utils/format.ts` and its 4 satellite files;
`.claude/rules/overlay.md`; `.arch/UX-FLOW.md`; E012's `ExplorerTab.tsx`,
`AnalystHeader.tsx` (new), `DateNavigator*.tsx`, `KpiTrend.tsx`,
`BreakCreditHistogram.tsx`, `KpiDonut.tsx`/`KpiDonutPanel.tsx` (new),
`explorer-day-kpis.ts` (new).

**Out:** engine/Rust session-logic changes (E015, E020); backend hardening —
mutex poisoning, persistence precedence, event-name constants (E019);
release docs/licensing/signing (E017); a state-management library or full
command-binding codegen (Alternative C, rejected above).

## Tasks

| ID | Task | Points | Agent | Wave |
|---|---|---|---|---|
| T01 | Shared `deskReducer` + two thin transport adapters (Tauri IPC, WS/REST) replacing the duplicated hooks; `useDesk` finally unit-tested via mocked `@tauri-apps/api/*` | 8 | ts-dev | 1 |
| T03 | Delete 5 dead components + `src/overlay/main.tsx` + `overlay.html` build entry, after grepping for live imports | 3 | ts-dev | 1 |
| T04 | Install ESLint (flat config, typescript-eslint, `eslint-plugin-react-hooks`); `pnpm lint` exits 0 | 3 | ts-dev | 1 |
| T06 | Consolidate 6 local `formatXxx` helpers into `src/utils/format.ts` | 3 | ts-dev | 1 |
| T07 | Correct `.claude/rules/overlay.md` to `scripts/tauri-dev.ps1 -Force` | 1 | main | 1 |
| T08 | Sync `.arch/UX-FLOW.md` for the Steps widget and click-timeline-to-Analyst | 1 | main | 1 |
| T02 | Rust→TS type codegen with `ts-rs` (ADR 017), replacing hand-written DTOs; delete E015's manual drift test | 5 | ts-dev | 2 |
| T05 | Wire the coverage gate into `pnpm build`/`just check`; add `justfile`; meet 80/80/75% or lower with a dated note + backlog entry (D4) | 5 | ts-dev | 2 |
| T09 | Analyst layout flip — date header on top, day-nav sticky at bottom (ref. E012-T09) | 5 | ts-dev | 3 |
| T10 | Recharts donut KPIs with numeric centre (ref. E012-T10); ADR 016 | 8 | ts-dev | 3 |
| T11 | Pulse highlight on KPI donuts when `selectedDay` changes (ref. E012-T11) | 3 | ts-dev | 3 |

Wave 1 = 19 points, Wave 2 = 10 points, Wave 3 = 16 points — all ≤ 40.
Per the caller's instruction, T01 (reducer merge) and T02 (codegen) sit in
different waves so no task in a single wave can claim the same file twice.
Wave 3 is sequential internally (T10 depends on T09, T11 depends on T10) —
see HANDOFF's Mental model for why that overrides the usual "no shared
claims per wave" rule for this one wave.

## Test strategy

- **T01** — happy: dispatching a `state-changed` action updates
  `sittingSeconds`/`transition` in the reducer, asserted with a pure
  `deskReducer` unit test (no Tauri, no WebSocket). nil: initial state
  before any event has `previousSession === null`. empty:
  `todaySummary.sessions.length === 0` still returns `todaySessions: []`,
  not undefined. error: a rejected `invoke("get_dashboard_state")` leaves
  `connected` false and does not throw out of the hook. New file
  `src/hooks/deskReducer.test.ts`; `useDesk.test.ts` mocks
  `@tauri-apps/api/core` and `@tauri-apps/api/event` the way
  `useRemoteDesk.test.ts` already mocks `WebSocket`. Fails today: neither
  file exists, `useDesk.ts` is 0.78% covered.
- **T02** — happy: a generated `src/generated/SessionStateDto.ts` matches
  the Rust struct's field set 1:1 (compile-time, via `tsc --noEmit`). nil:
  an `Option<T>` field renders as `T | null` in the generated output, not
  `T | undefined`. error (the drift case): renaming a Rust field without
  regenerating breaks `pnpm typecheck`, proving the check is load-bearing.
  Fails today: no `#[derive(TS)]`, no `src/generated/`.
- **T03** — assertion script `scripts/check-e018-t03-dead-code.mjs` fails
  today because the 5 components + `src/overlay/main.tsx` + `overlay.html`
  still exist; after the task it asserts all 7 paths are absent AND
  `pnpm typecheck`/`pnpm test:unit` still pass (no dangling import).
- **T04** — `pnpm lint` fails today (`'eslint' is not recognized`); after
  the task it exits 0.
- **T05** — `pnpm build` fails to call `test:coverage` today (verified: it
  only runs `test:all`); after the task `pnpm build` runs the coverage
  check and exits 0 against the (possibly lowered) threshold.
- **T06** — each of the 4 satellite files' existing tests continue to pass
  after importing from `utils/format.ts`; a new `format.test.ts` case
  covers the newly-moved functions. Fails today: the functions are private
  to their files, not exported from `utils/format.ts`.
- **T07/T08** — doc-assertion scripts (below) fail today because the stale
  strings are present / the new strings are absent.
- **T09/T10/T11** — component tests per the referenced E012 task files'
  own acceptance criteria (`AnalystHeader` renders + fires nav; `KpiDonut`
  arc matches `valuePct`; pulse class toggles on `pulseKey` change). All
  fail today because the components don't exist yet.

## Evidence contract

| check_id | class | procedure | expected | record |
|---|---|---|---|---|
| reducer-unit | test | `npx vitest run --config vite.config.ts src/hooks` | pass | `evidence/records/T01-reducer-unit.json` |
| dead-code-gone | test | `node scripts/check-e018-t03-dead-code.mjs` | pass | `evidence/records/T03-dead-code-gone.json` |
| lint-clean | test | `pnpm lint` | pass | `evidence/records/T04-lint-clean.json` |
| codegen-typecheck | test | `pnpm typecheck` | pass | `evidence/records/T02-codegen-typecheck.json` |
| coverage-gate | test | `pnpm build` | pass | `evidence/records/T05-coverage-gate.json` |
| format-consolidated | test | `npx vitest run --config vite.config.ts src/utils/format.test.ts` | pass | `evidence/records/T06-format-consolidated.json` |
| overlay-doc-fixed | manual | `node scripts/check-e018-t07-overlay-doc.mjs` | pass | `evidence/records/T07-overlay-doc-fixed.json` |
| ux-flow-synced | manual | `node scripts/check-e018-t08-ux-flow.mjs` | pass | `evidence/records/T08-ux-flow-synced.json` |
| analyst-layout | test | `npx vitest run --config vite.config.ts src/analyst/AnalystHeader.test.tsx` | pass | `evidence/records/T09-analyst-layout.json` |
| kpi-donuts | test | `npx vitest run --config vite.config.ts src/analyst/charts/KpiDonut.test.tsx` | pass | `evidence/records/T10-kpi-donuts.json` |
| kpi-pulse | test | `npx vitest run --config vite.config.ts src/analyst/charts/KpiDonut.test.tsx` | pass | `evidence/records/T11-kpi-pulse.json` |
| ui-polish-visual | visual | browser agent on `pnpm tauri:dev` → Analyst window (Outside AO) | layout matches E012-T09/T10/T11 acceptance criteria | `evidence/records/T09-11-ui-polish-visual.json` |

## Architecture impact

`.arch/ARCHITECTURE.md`: document the frontend data-layer boundary as
"one reducer, two transport adapters" (replacing the two-hook description);
document the Rust→TS codegen boundary (`src/generated/`, `ts-rs`). New
ADR 016 (recharts) and ADR 017 (ts-rs). No new Tauri command, no new IPC
event, no change to the Rust session model (that's E015/E020).

## Acceptance criteria

1. `src/hooks/useDesk.ts` and `useRemoteDesk.ts` both dispatch into one
   shared reducer; no duplicated derivation logic remains between them.
2. `useDesk.ts` has a real unit test suite (not just 0.78% incidental
   coverage from other tests).
3. `src/types.ts`'s DTO interfaces are generated from Rust structs, not
   hand-typed; a Rust field rename breaks `pnpm typecheck`.
4. The 5 dead components, their tests, `src/overlay/main.tsx`, and
   `overlay.html` no longer exist; `vite.config.ts` has no `overlay` build
   input.
5. `pnpm lint` exits 0. `pnpm build` runs the coverage check and exits 0.
6. `src/utils/format.ts` is the single home for date/duration formatting
   used across the 4 previously-independent call sites.
7. `.claude/rules/overlay.md` and `.arch/UX-FLOW.md` match current behavior.
8. Analyst window shows the E012-T09/T10/T11 layout, donuts, and pulse —
   confirmed by a browser pass (Outside AO).
