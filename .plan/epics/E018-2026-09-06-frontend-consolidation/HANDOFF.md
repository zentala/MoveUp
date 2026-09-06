---
formatVersion: 1
type: handoff
status: todo
---

# E018 Handoff — frontend consolidation

## TLDR

Implementing session/agent reads only this file plus [`PLAN.md`](PLAN.md).
Three waves: Wave 1 (T01, T03, T04, T06, T07, T08 — 19 pts, all file-disjoint,
parallel-safe), Wave 2 (T02, T05 — 10 pts, must start after Wave 1 lands so
lint/dead-code-deletion don't fight codegen/coverage over the same files),
Wave 3 (T09 → T10 → T11 — 16 pts, sequential by `depends_on` despite sharing
a wave label, because each genuinely edits files the previous task just
created). Work in a worktree via `wt-add`, branch
`feat/E018-frontend-consolidation`. **Depends on E015** — if E015 has not
merged yet, wait; T01 and T02 both assume `limit_used_secs` is already the
only credited field (E015's rename), not `current_session_secs`.

## Decisions already made (apply unless Paweł overrides)

- D3: codegen library is **ts-rs**, not specta — see PLAN.md Alternatives.
  Emit into `src/generated/`, re-export from `src/types.ts`.
- D4: coverage threshold — try to hit 80/80/75% after T01's reducer split
  makes `useDesk.ts` testable; if still short, lower the numbers in
  `vite.config.ts` with a comment `# lowered 2026-0X-XX, see BACKLOG #<n>`
  and file that BACKLOG entry in the same commit. Never leave a threshold
  that silently never runs (that was the original bug).
- T02 deletes E015's manual DTO-drift test rather than keeping both checks.
  Grep `src/` for `dto-drift` (or similar) before starting T02 — if E015
  named it something else, find it by searching for `serde_json::to_string`
  usage referenced from a `.test.ts` file.
- Wave 3 (T09/T10/T11) references but does not copy the content of
  `.plan/epics/E012-2026-05-16-analyst-dashboard/tasks/E012-T09-analyst-layout-flip.md`,
  `E012-T10-recharts-donut-kpis.md`, `E012-T11-selected-day-pulse.md` — open
  those files for the full context, decisions, and acceptance criteria; this
  handoff only adds the AO wiring and the mockup-first / browser-verify
  requirement those files don't carry.
- Per `.claude/rules/ux-design-flow.md`, wave 3 UI is **mockup-first**: the
  implementing session shows the change via the existing `/mockup` dev
  route (or the mockup gallery) before calling the task done, then a
  browser pass confirms the live Analyst window matches. Both live in
  `## Outside AO` below — the AO YAML tasks cover only the code + unit
  tests, not the visual sign-off.

## Mental model

- **Reducer/adapter split (T01).** Today: `src/hooks/useDesk.ts:37-247`
  (Tauri `invoke`/`listen`, polling `get_dashboard_state` every 1s and
  `get_today_summary` every 10s) and `src/hooks/useRemoteDesk.ts:43-252`
  (WebSocket + REST fallback) each own their own `useState` calls and
  derive `previousSession`/`todaySessions`/`breakResetProgress`
  independently. Target: a new `src/hooks/deskReducer.ts` exporting a pure
  `(state, action) => state` plus the derived-value selectors; `useDesk.ts`
  and `useRemoteDesk.ts` shrink to: own their transport (poll+listen vs
  WS+fetch), translate raw payloads into the reducer's action shape,
  `useReducer(deskReducer, initialDeskState)`. `useDeskTypes.ts`'s
  `UseDeskResult` interface is the contract both must still satisfy — do
  not change its shape in this task (widgets/`useWidgetData.ts` depend on
  it unchanged).
- **Codegen boundary (T02).** DTOs live today as hand-typed interfaces in
  `src/types.ts:59-89` (`SessionStateDto`, `StateChangedPayload`,
  `DashboardState`, `MetricSnapshot`, `TodaySummaryDto`) and
  `src/components/settings/SettingsTypes.ts:10-26` (`DeskSettings`
  mirroring `src-tauri/src/config.rs`'s `AppConfig`). Target: add
  `#[derive(ts_rs::TS)] #[ts(export, export_to = "../src/generated/")]` to
  the Rust structs in `session_types.rs`, `config.rs`, and any DTO struct
  in `commands.rs`/`commands_analyst.rs`; regenerate via
  `cargo test --manifest-path src-tauri/Cargo.toml --lib -- export_bindings`
  (ts-rs's own export-on-test-run mechanism — no new `package.json` script
  needed, which keeps this task from touching the same file T05 touches in
  the same wave); `types.ts` and `SettingsTypes.ts` re-export from
  `src/generated/*.ts` instead of hand-typing. Do not
  touch the Rust field values or session logic — E015 already fixed those
  names; this task only changes how the TS mirror is produced.
- **Dead code (T03).** Before deleting, run
  `grep -rn "AppProgressBar\|HeightRail\|SessionProgress\|TodayStats\|TransitionBanner\|src/overlay/main" src --include=*.tsx --include=*.ts` and
  confirm every hit is inside the files being deleted or their own test —
  `frontend.md` already did this check (finding #6/#7) but re-verify before
  deleting, since new imports may have landed since. `vite.config.ts:38`'s
  `overlay: resolve(__dirname, "overlay.html")` build input goes with it.
- **Tooling (T04, T05).** `package.json:28`'s `"lint": "eslint src/"` has
  no matching devDependency or config — add both plus an `eslint.config.mjs`
  (flat config, `typescript-eslint`, `eslint-plugin-react-hooks`).
  `vite.config.ts:22-31` already has real thresholds; the gap is that
  `package.json`'s `"build"` script (line 8: `pnpm test:all && tsc && vite build`)
  never calls `test:coverage`. Add a `justfile` per `~/.claude/rules/just.md`'s
  template (`setup`/`dev`/`build`/`test`/`check`/`lint`/`typecheck`/`clean`
  targets wrapping the existing `pnpm` scripts) — this repo has none today
  (`frontend.md` finding #14).
- **Format consolidation (T06).** `src/utils/format.ts` already has
  `formatDuration`/`formatDurationShort`. Move `formatDate`
  (`src/analyst/charts/TimelineDetailHeader.tsx:28`),
  `formatRefreshedAt`/`formatRangeLabel` (`src/analyst/ExplorerTab.tsx:44,54`),
  `formatIdleTime` (`src/components/StateIndicator.tsx:24`), `formatTime`
  (`src/widgets/one-bar/OneBarTimeline.tsx:37`) there, composing from the
  two existing primitives where possible.
- **Wave 3 file overlap is intentional.** `T09` creates
  `src/analyst/AnalystHeader.tsx` and edits `ExplorerTab.tsx` +
  `TimelineDetailHeader.tsx` + `DateNavigator.tsx`/`DateNavigatorHeader.tsx`/
  `DateNavigatorColumn.tsx` (add a `placement` prop, per E012-T09's own
  decision). `T10` creates `KpiDonut.tsx`/`KpiDonutPanel.tsx`/
  `explorer-day-kpis.ts`, edits `ExplorerTab.tsx` again (swaps `KpiTrend.tsx`
  + `BreakCreditHistogram.tsx` cells for the new panel), adds `recharts` to
  `package.json`, writes ADR 016. `T11` edits `KpiDonut.tsx` +
  `KpiDonutPanel.tsx` again and adds `kpi-donut.css`. Because T10 needs
  files T09 creates, and T11 needs files T10 creates, they run in
  `depends_on` order, never in parallel, even though they're grouped as one
  "wave" for point-budget purposes.

## Tasks

- [ ] **T01** (8, ts-dev) — shared `deskReducer` + thin Tauri/WS adapters;
  `useDesk` unit-tested. Verify: `npx vitest run --config vite.config.ts src/hooks`.
- [ ] **T03** (3, ts-dev) — delete 5 dead components + overlay entry, after
  a live grep. Verify: `node scripts/check-e018-t03-dead-code.mjs`.
- [ ] **T04** (3, ts-dev) — install + configure ESLint. Verify: `pnpm lint`.
- [ ] **T06** (3, ts-dev) — consolidate `formatXxx` helpers. Verify:
  `npx vitest run --config vite.config.ts src/utils/format.test.ts`.
- [ ] **T07** (1, main) — fix `.claude/rules/overlay.md`. Verify:
  `node scripts/check-e018-t07-overlay-doc.mjs`.
- [ ] **T08** (1, main) — sync `.arch/UX-FLOW.md`. Verify:
  `node scripts/check-e018-t08-ux-flow.mjs`.
- [ ] **T02** (5, ts-dev) — ts-rs codegen, ADR 017, delete E015's manual
  drift test. Verify: `pnpm typecheck`.
- [ ] **T05** (5, ts-dev) — wire coverage gate into `build`, add `justfile`.
  Verify: `pnpm build`.
- [ ] **T09** (5, ts-dev) — Analyst layout flip (ref. E012-T09). Verify:
  `npx vitest run --config vite.config.ts src/analyst/AnalystHeader.test.tsx`.
- [ ] **T10** (8, ts-dev) — Recharts donut KPIs (ref. E012-T10), depends on
  T09. Verify: `npx vitest run --config vite.config.ts src/analyst/charts/KpiDonut.test.tsx`.
- [ ] **T11** (3, ts-dev) — pulse highlight (ref. E012-T11), depends on T10.
  Verify: `npx vitest run --config vite.config.ts src/analyst/charts/KpiDonut.test.tsx`.

## Outside AO

- **Mockup approval for T09/T10/T11** — per `.claude/rules/ux-design-flow.md`,
  before any of the three is called done, run `pnpm dev` (or confirm it's
  already running) and show the change at `http://localhost:1443/#/mockup/analyst`.
  This is a human-in-the-loop step; AO's automated verification only proves
  the components render and pass their unit tests, not that the layout
  reads right.
- **Browser pass on the live Analyst window** — after T09+T10+T11 land,
  dispatch the `browser` agent against `pnpm tauri:dev` → Analyst window:
  confirm the date header + arrows on top, sticky day-nav at bottom, ≥3
  donut KPI cards with arc + centre numbers, and a visible pulse on day
  change. Record the result at `evidence/records/T09-11-ui-polish-visual.json`.
  This is the check_id `ui-polish-visual` in PLAN.md's evidence contract —
  it is `class: visual`, which AO does not automate.
- **Coverage threshold call (D4)** — if T05 cannot reach 80/80/75% within
  its budget, the decision to lower thresholds is Paweł's to confirm before
  merging Wave 2, not the implementing agent's to make silently. Surface it
  as a normal PR/commit note, not a question in chat.

## Done means

All 8 acceptance criteria in PLAN.md hold, evidence records are `current`,
`.plan/HISTORY.md` gets an E018 entry, `STATE.md` is updated, and E012's
`ORCHESTRATOR.md` gets a one-line pointer to this epic closing its last 3
tasks (E016's close-out ceremony still owns marking E012 itself done).

## AO

```yaml
project: MoveUp
epic: E018
base_ref: main
tasks:
  - id: E018-T01
    repo: MoveUp
    executor: ts-dev
    depends_on: []
    write_set: ["src/hooks/deskReducer.ts", "src/hooks/deskReducer.test.ts", "src/hooks/useDesk.ts", "src/hooks/useDesk.test.ts", "src/hooks/useRemoteDesk.ts", "src/hooks/useRemoteDesk.test.ts", "src/hooks/useDeskTypes.ts"]
    claims: ["src/hooks/deskReducer.ts", "src/hooks/deskReducer.test.ts", "src/hooks/useDesk.ts", "src/hooks/useDesk.test.ts", "src/hooks/useRemoteDesk.ts", "src/hooks/useRemoteDesk.test.ts", "src/hooks/useDeskTypes.ts"]
    verification: "npx vitest run --config vite.config.ts src/hooks"
    budget_minutes: 90
  - id: E018-T03
    repo: MoveUp
    executor: ts-dev
    depends_on: []
    write_set: ["src/components/AppProgressBar.tsx", "src/components/AppProgressBar.test.tsx", "src/components/HeightRail.tsx", "src/components/HeightRail.test.tsx", "src/components/SessionProgress.tsx", "src/components/SessionProgress.test.tsx", "src/components/SessionProgress.floating.test.tsx", "src/components/TodayStats.tsx", "src/components/TodayStats.test.tsx", "src/components/TransitionBanner.tsx", "src/components/TransitionBanner.test.tsx", "src/overlay/main.tsx", "overlay.html", "vite.config.ts", "scripts/check-e018-t03-dead-code.mjs"]
    claims: ["src/components/AppProgressBar.tsx", "src/components/AppProgressBar.test.tsx", "src/components/HeightRail.tsx", "src/components/HeightRail.test.tsx", "src/components/SessionProgress.tsx", "src/components/SessionProgress.test.tsx", "src/components/SessionProgress.floating.test.tsx", "src/components/TodayStats.tsx", "src/components/TodayStats.test.tsx", "src/components/TransitionBanner.tsx", "src/components/TransitionBanner.test.tsx", "src/overlay/main.tsx", "overlay.html", "vite.config.ts", "scripts/check-e018-t03-dead-code.mjs"]
    verification: "node scripts/check-e018-t03-dead-code.mjs"
    budget_minutes: 45
  - id: E018-T04
    repo: MoveUp
    executor: ts-dev
    depends_on: []
    write_set: ["package.json", "pnpm-lock.yaml", "eslint.config.mjs"]
    claims: ["package.json", "pnpm-lock.yaml", "eslint.config.mjs"]
    verification: "pnpm lint"
    budget_minutes: 60
  - id: E018-T06
    repo: MoveUp
    executor: ts-dev
    depends_on: []
    write_set: ["src/utils/format.ts", "src/utils/format.test.ts", "src/analyst/charts/TimelineDetailHeader.tsx", "src/analyst/ExplorerTab.tsx", "src/components/StateIndicator.tsx", "src/widgets/one-bar/OneBarTimeline.tsx"]
    claims: ["src/utils/format.ts", "src/utils/format.test.ts", "src/analyst/charts/TimelineDetailHeader.tsx", "src/components/StateIndicator.tsx", "src/widgets/one-bar/OneBarTimeline.tsx"]
    verification: "npx vitest run --config vite.config.ts src/utils/format.test.ts"
    budget_minutes: 45
  - id: E018-T07
    repo: MoveUp
    executor: main
    depends_on: []
    write_set: [".claude/rules/overlay.md", "scripts/check-e018-t07-overlay-doc.mjs"]
    claims: [".claude/rules/overlay.md", "scripts/check-e018-t07-overlay-doc.mjs"]
    verification: "node scripts/check-e018-t07-overlay-doc.mjs"
    budget_minutes: 30
  - id: E018-T08
    repo: MoveUp
    executor: main
    depends_on: []
    write_set: [".arch/UX-FLOW.md", "scripts/check-e018-t08-ux-flow.mjs"]
    claims: [".arch/UX-FLOW.md", "scripts/check-e018-t08-ux-flow.mjs"]
    verification: "node scripts/check-e018-t08-ux-flow.mjs"
    budget_minutes: 30
  - id: E018-T02
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E018-T01"]
    write_set: ["src-tauri/Cargo.toml", "src-tauri/Cargo.lock", "src-tauri/src/session_types.rs", "src-tauri/src/config.rs", "src-tauri/src/commands.rs", "src-tauri/src/commands_analyst.rs", "src-tauri/src/db.rs", "src-tauri/src/lib.rs", "src-tauri/src/metrics/mod.rs", "src-tauri/src/serial_parser.rs", "src-tauri/src/session_dto_fixture_tests.rs", "src/generated/**", "src/types.ts", "src/components/settings/SettingsTypes.ts", "src/components/settings/DebugSection.tsx", "src/hooks/deskReducer.ts", "src/hooks/deskReducer.test.ts", "src/hooks/useDesk.test.ts", "src/test/dto-drift.test.ts", "src/test/fixtures/session-dto.json", "src/test/scenario-helpers.ts", "src/widgets/one-bar/OneBarTimeline.tsx", "src/widgets/one-bar/OneBarTimeline.test.tsx", ".arch/ADR/017-ts-rs-for-rust-ts-codegen.md", ".arch/ARCHITECTURE.md", ".plan/decisions.jsonl"]
    claims: ["src-tauri/Cargo.toml", "src-tauri/Cargo.lock", "src-tauri/src/session_types.rs", "src-tauri/src/config.rs", "src-tauri/src/commands.rs", "src-tauri/src/commands_analyst.rs", "src-tauri/src/db.rs", "src-tauri/src/lib.rs", "src-tauri/src/metrics/mod.rs", "src-tauri/src/serial_parser.rs", "src-tauri/src/session_dto_fixture_tests.rs", "src/generated/**", "src/types.ts", "src/components/settings/SettingsTypes.ts", "src/components/settings/DebugSection.tsx", "src/hooks/deskReducer.ts", "src/hooks/deskReducer.test.ts", "src/hooks/useDesk.test.ts", "src/test/dto-drift.test.ts", "src/test/fixtures/session-dto.json", "src/test/scenario-helpers.ts", "src/widgets/one-bar/OneBarTimeline.tsx", "src/widgets/one-bar/OneBarTimeline.test.tsx", ".arch/ADR/017-ts-rs-for-rust-ts-codegen.md", ".arch/ARCHITECTURE.md", ".plan/decisions.jsonl"]
    verification: "pnpm typecheck"
    budget_minutes: 90
  - id: E018-T05
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E018-T01"]
    write_set: ["vite.config.ts", "package.json", "justfile", "src/hooks/useDesk.test.ts"]
    claims: ["vite.config.ts", "package.json", "justfile"]
    verification: "pnpm build"
    budget_minutes: 90
  - id: E018-T09
    repo: MoveUp
    executor: ts-dev
    depends_on: []
    write_set: ["src/analyst/AnalystHeader.tsx", "src/analyst/AnalystHeader.test.tsx", "src/analyst/ExplorerTab.tsx", "src/analyst/charts/TimelineDetailHeader.tsx", "src/analyst/charts/DateNavigator.tsx", "src/analyst/charts/DateNavigatorHeader.tsx", "src/analyst/charts/DateNavigatorColumn.tsx"]
    claims: ["src/analyst/AnalystHeader.tsx", "src/analyst/AnalystHeader.test.tsx", "src/analyst/ExplorerTab.tsx", "src/analyst/charts/TimelineDetailHeader.tsx", "src/analyst/charts/DateNavigator.tsx", "src/analyst/charts/DateNavigatorHeader.tsx", "src/analyst/charts/DateNavigatorColumn.tsx"]
    verification: "npx vitest run --config vite.config.ts src/analyst/AnalystHeader.test.tsx"
    budget_minutes: 60
  - id: E018-T10
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E018-T09"]
    write_set: ["src/analyst/charts/KpiDonut.tsx", "src/analyst/charts/KpiDonut.test.tsx", "src/analyst/charts/KpiDonutPanel.tsx", "src/analyst/explorer-day-kpis.ts", "src/analyst/ExplorerTab.tsx", "package.json", ".arch/ADR/016-recharts-for-kpi-donuts.md"]
    claims: ["src/analyst/charts/KpiDonut.tsx", "src/analyst/charts/KpiDonut.test.tsx", "src/analyst/charts/KpiDonutPanel.tsx", "src/analyst/explorer-day-kpis.ts", "src/analyst/ExplorerTab.tsx", ".arch/ADR/016-recharts-for-kpi-donuts.md"]
    verification: "npx vitest run --config vite.config.ts src/analyst/charts/KpiDonut.test.tsx"
    budget_minutes: 90
  - id: E018-T11
    repo: MoveUp
    executor: ts-dev
    depends_on: ["E018-T10"]
    write_set: ["src/analyst/charts/KpiDonut.tsx", "src/analyst/charts/KpiDonut.test.tsx", "src/analyst/charts/KpiDonutPanel.tsx", "src/analyst/charts/kpi-donut.css"]
    claims: ["src/analyst/charts/KpiDonut.tsx", "src/analyst/charts/KpiDonut.test.tsx", "src/analyst/charts/KpiDonutPanel.tsx", "src/analyst/charts/kpi-donut.css"]
    verification: "npx vitest run --config vite.config.ts src/analyst/charts/KpiDonut.test.tsx"
    budget_minutes: 45
```
