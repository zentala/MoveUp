# Frontend architecture review — MoveUp (src/)

## TLDR

The frontend is small (73 source files, 37 test files, 248 tests, all green) and mostly
well-factored around one real pattern: `useWidgetData` → `ActiveWidget` (currently the
single `OneBarWidget`) as the sole production UI, with a separate Analyst window and a
dev-only mockup gallery. The biggest structural problem is **duplication, not chaos**:
`useDesk.ts` and `useRemoteDesk.ts` reimplement the same state machine twice instead of
sharing one reducer; Rust DTOs are hand-copied into `types.ts` with no codegen and no
test that catches drift; and a whole dead UI generation (5 components, 1 build entry
point) from the pre-OneBar era was never deleted. Tooling has two silent gaps that look
fine on the surface: `pnpm lint` is wired to an `eslint` binary that isn't installed
(no devDependency, no config file — it fails immediately), and the 80/80/75% coverage
gate in `vite.config.ts` is real but **not part of any build script** — `pnpm build`
never runs `test:coverage`, so the gate has never actually blocked anything, and real
coverage today is 76.5/68.1/66.0%, all three below threshold. No `justfile` exists.

## Map

| Path | Purpose | Files | Verdict |
|---|---|---|---|
| `src/App.tsx`, `src/main.tsx` | Root component + entry, hash-based mini-router (`#/analyst`, `#/mockup/analyst`, `#/mockup`) | 2 | OK, but router logic lives inline in the component (`App.tsx:28-52`) |
| `src/welcome.tsx`, `welcome.html` | Separate Tauri window entry for the welcome popup | 1 + html | OK, thin and correct |
| `src/overlay/main.tsx`, `overlay.html` | Canvas-based debug overlay entry | 1 + html | **Dead** — real overlay is native WinAPI in Rust (`.claude/rules/overlay.md`); this is unreferenced debug scaffolding still built by Vite |
| `src/hooks/useDesk.ts`, `useRemoteDesk.ts`, `useDeskAuto.ts`, `useDeskTypes.ts` | Data layer: Tauri IPC vs WebSocket, auto-selected at module load by `window.__TAURI_INTERNALS__` | 4 | Two working but near-duplicate implementations of the same state machine |
| `src/hooks/useWidgetData.ts` | Maps `UseDeskResult` → `WidgetProps` (presentation contract) | 1 | Clean, single mapping point |
| `src/hooks/useExponentialPoll.ts`, `useTimer.ts`, `useTimerAnimations.ts`, `useTimelineSkin.ts` | Small generic/UI hooks | 4 | Well-scoped, well-tested |
| `src/types.ts` | Hand-written mirror of every Rust DTO/event payload | 1 | No codegen (`ts-rs`/`specta`) from `src-tauri`; only shape-compatible by convention |
| `src/components/*.tsx` (top level) | Mixed bag: production (`StateIndicator`, `ConnectionOverlay`, `StepsWidget`, `ShareStats`, `WelcomePopup`, `AutostartToggle`, `SettingsPanel`) + **dead** pre-OneBar widgets (`AppProgressBar`, `HeightRail`, `SessionProgress`, `TodayStats`, `TransitionBanner`) | 21 (+13 tests) | Ownership overlaps with `widgets/one-bar/*`; dead files not pruned |
| `src/components/settings/*` | Settings sub-panels (calibration, notifications, profile selector, timeline skin, debug, telemetry, tab bar) | 9 (+1 test) | Clear, one concern per file, good size discipline |
| `src/widgets/` | Widget registry + the single `OneBarWidget` + its `one-bar/` sub-components | 10 | Current production UI; well-tested |
| `src/analyst/` | Separate Analyst window: Catalog tab, Explorer tab, charts, date navigation, hooks | 33 | Largest subtree; own hook family (`useDataCatalog`, `useRangeQuery`, …) with a cleaner discriminated-union loading/ready/error pattern than the main data layer |
| `src/pages/` | `AnalystLive`, `AnalystMockup`, `MockupGallery` — hash-route targets lazy-loaded from `App.tsx` | 3 | Correctly separated from `analyst/`, but `MockupGallery.tsx` is 89 lines of near-100% inline `style={{}}` |
| `src/test/` | Scenario fixtures (`scenarios.ts` + `scenarios-sitting.ts` + `scenarios-other.ts`) and analyst fixtures | 9 | Actively used by mockup gallery, unit and integration tests — genuinely one source of truth, not rotted |
| `src/styles/globals.css` | Design tokens + global layout, 886 lines | 1 | Well-organized token system; timeline-skin vars kept in sync with `colors.rs` by comment convention only (no test) |
| `src/utils/` | `colors.ts`, `format.ts`, `timeline.ts` | 3 (+2 tests) | Small, focused; but 3 more local `formatXxx` helpers exist outside this folder (see Findings) |
| `tests/integration/`, `tests/e2e/` | Vitest-against-running-app and Playwright-against-tauri-driver suites | 8 | Not run by `pnpm test:all`; require manual app/driver bring-up — not rotted in content, but unverified as part of any automated gate |

## Findings table

| # | Finding | Path:line | Importance | Points |
|---|---|---|---|---|
| 1 | `pnpm lint` is broken: `eslint` is invoked but not installed (no `devDependency`) and no `.eslintrc`/`eslint.config.*` exists anywhere in the repo. Running it fails immediately with "not recognized as an internal or external command." Lint has effectively never run. | `package.json:23` (`"lint": "eslint src/"`) | High | 3 |
| 2 | Coverage gate (80% lines/functions, 75% branches) in `vite.config.ts` is real but **not wired into any script that runs on build or CI** — `pnpm build` / `pnpm tauri:build` call `pnpm test:all`, which is `test:unit && test:unit:rust`, never `test:coverage`. The gate has never blocked a build. Actual measured coverage: lines 76.53%, functions 68.09%, branches 66.01% — all three under threshold today. | `vite.config.ts:22-31`, `package.json:9,11,19` | High | 3 |
| 3 | `useDesk.ts` (Tauri IPC) and `useRemoteDesk.ts` (WebSocket) reimplement the same state shape, the same `previousSession`/`todaySessions`/`breakResetProgress` derivations, and the same `applyStateChanged`/transition-timer logic almost line-for-line, instead of sharing one reducer/selector layer with two thin transport adapters. Any bugfix or new field (e.g. `continuous_computer_secs`, added per ADR 011) has to be applied twice — `useDesk.ts` and `useRemoteDesk.ts` were in fact both hand-edited to add it. | `src/hooks/useDesk.ts:37-247`, `src/hooks/useRemoteDesk.ts:43-252` | High | 8 |
| 4 | `useDesk.ts` has 0.78% statement coverage under the coverage run (only reachable via a full Tauri runtime, which the unit-test environment can't fake) — the entire desktop data layer is effectively untested at the unit level, while its near-duplicate `useRemoteDesk.ts` is well tested (94.6%) purely because WebSocket is fakeable in jsdom. This asymmetry is a direct consequence of finding #3: one reducer with two thin transports would let the reducer's logic be tested once, transport-agnostically. | `src/hooks/useDesk.ts` (coverage report, `hooks` section) | Medium | 5 |
| 5 | `src/types.ts` hand-duplicates every Rust DTO/event payload (`SessionStateDto`, `StateChangedPayload`, `DashboardState`, `MetricSnapshot`, …) with no codegen (`ts-rs`, `specta`, or similar) and no round-trip test. `SettingsTypes.ts`'s `DeskSettings` similarly hand-mirrors `AppConfig` in `src-tauri/src/config.rs` field-by-field. A field renamed or added on the Rust side (e.g. the `continuous_computer_secs`/`max_continuous_computer_secs` additions from ADR 011) has no compiler or test signal on the TS side until a manual runtime check catches the mismatch. | `src/types.ts:59-89`, `src/components/settings/SettingsTypes.ts:9-25`, `src-tauri/src/config.rs:28-54` | Medium | 8 |
| 6 | Five components from the pre-`OneBarWidget` UI generation are dead code, unreferenced anywhere outside their own test files: `AppProgressBar.tsx`, `HeightRail.tsx`, `SessionProgress.tsx`, `TodayStats.tsx`, `TransitionBanner.tsx`. All last touched in March 2026 (before the "Core provides data via useWidgetData, OneBarWidget handles presentation" architecture noted in `App.tsx:1-6`). Their `.test.tsx` siblings still run and count toward the "248 tests passed" total, silently inflating confidence and coverage numbers for code nothing renders. | `src/components/AppProgressBar.tsx`, `src/components/HeightRail.tsx`, `src/components/SessionProgress.tsx`, `src/components/TodayStats.tsx`, `src/components/TransitionBanner.tsx` (+ matching `*.test.tsx`) | Medium | 3 |
| 7 | `src/overlay/main.tsx` + `overlay.html` is dead debug scaffolding: hardcoded `TEST:` values, emoji `console.log`s, a 500ms `setTimeout` before attaching the listener, and a Canvas 2D renderer. Per `.claude/rules/overlay.md` the real overlay is a native WinAPI window rendered from Rust (`overlay_renderer.rs`), not a Tauri webview at all. This entry is still wired into `vite.config.ts`'s `rollupOptions.input`, so it is compiled and shipped on every build for no functional reason. Last touched 2026-03-29. | `src/overlay/main.tsx:1-59`, `overlay.html:1-24`, `vite.config.ts:34-42` | Medium | 2 |
| 8 | `.arch/UX-FLOW.md` (mandated single source of truth per `.claude/rules/ux-flow-sync.md`) was last updated 2026-05-16, but the Google Fit `StepsWidget` (mounted into `OneBarWidget` 2026-05-18/19/21) and the "click popup timeline to open the full Analyst window" behavior (2026-05-21) are both undocumented there — `grep -i "steps\|google fit\|click.*timeline"` finds zero matches. Violates the rule's own "mandatory updates" list (new UI element + new interaction). | `.arch/UX-FLOW.md`, `src/components/StepsWidget.tsx`, `src/widgets/OneBarWidget.tsx:73-78` | Medium | 2 |
| 9 | Six independent local `formatXxx`/`formatDate`-style helpers exist outside `src/utils/format.ts`, duplicating the "format a date/duration for display" concern instead of extending the shared module: `formatDate` (TimelineDetailHeader), `formatRefreshedAt`/`formatRangeLabel` (ExplorerTab), `formatIdleTime` (StateIndicator), `formatTime` (OneBarTimeline). None import from `utils/format.ts`, and each reinvents padding/rounding logic independently. | `src/analyst/charts/TimelineDetailHeader.tsx:28`, `src/analyst/ExplorerTab.tsx:44,54`, `src/components/StateIndicator.tsx:24`, `src/widgets/one-bar/OneBarTimeline.tsx:37` | Low | 3 |
| 10 | Legacy product name "Smart Desk" appears in user-facing share-card copy, contradicting `CLAUDE.md`'s naming section ("Legacy names... must not appear in code") and the current "MoveUp" branding used everywhere else (window titles, `productName`, autostart registry key). This is copy a real user could screenshot and share. | `src/components/ShareStats.tsx:57,70,179` | Low | 1 |
| 11 | `CatalogTab.tsx` (223 lines, part of the user-facing Analyst window, not a dev-only mockup) carries 16 inline `style={{}}` blocks instead of using `globals.css` tokens or a co-located stylesheet, unlike the rest of `analyst/charts/*` which mostly use CSS classes. Inconsistent styling approach within the same feature folder. | `src/analyst/CatalogTab.tsx` | Low | 3 |
| 12 | `MockupGallery.tsx` is ~90% inline styles (13 `style={{}}` blocks in 89 lines), hardcoding colors (`#DAA520`, `#0a0a15`, `#1a1a2e`) that don't reference the `globals.css` token set at all, so the mockup gallery — the tool meant to preview the real design system — visually diverges from it. Acceptable for a dev-only route but worth a token pass since it undermines the tool's own purpose (accurate preview). | `src/pages/MockupGallery.tsx:14-88` | Low | 2 |
| 13 | Accessibility is thin: only 15 of 46 non-test `.tsx` files use `aria-*` or `role=` at all. No global `:focus-visible` ring found in `globals.css` (contradicts the project's own `rules/css.md` accessibility checklist). No systematic keyboard-navigation pass evident (settings tabs, mockup filter buttons are plain `<button>` with no `aria-pressed`/`aria-selected`). Low severity given this is a single-user desktop app, but worth a note since `StateIndicator`/`SettingsTabBar` are interactive controls. | `src/styles/globals.css` (no `:focus-visible` rule found), `src/components/settings/SettingsTabBar.tsx` | Low | 3 |
| 14 | No `justfile` at the repo root, contradicting the user's global convention (every repo gets one; `just build`/`just dev`/`just test`/`just check` as the uniform command surface). All commands here are memorized as `pnpm <script>` / PowerShell scripts instead. | repo root (absence) | Low | 2 |
| 15 | `.claude/rules/overlay.md` documents `scripts/tauri-dev.sh` and `pnpm tauri:dev --force`, but the actual (and only) launcher is `scripts/tauri-dev.ps1` with a `-Force` PowerShell switch — the `.sh` file doesn't exist in `scripts/`, and `package.json`'s `tauri:dev` script has no argument pass-through for `-Force` at all. Doc drifted after the bash→PowerShell rewrite (the `.ps1` file's own header says "Replaces tauri-dev.sh for Windows PowerShell environments"). | `.claude/rules/overlay.md` ("pnpm scripts" section), `scripts/tauri-dev.ps1:1-12`, `package.json:8` | Low | 1 |
| 16 | `tests/integration/*` and `tests/e2e/*` are not rotted in content (commands like `inject_reading`/`get_session_state` still exist in `src-tauri/src/commands.rs`), but neither is runnable unattended: integration tests require a manually-started `pnpm tauri:dev` instance reachable over Tauri IPC from a separate `vitest` process, and e2e requires `tauri-driver --compatibility-mode` running first. Neither is part of `pnpm test:all`/`test:all:with-integration` is a separate manual script, and there is no evidence (no CI config found) either suite runs regularly. | `tests/integration/setup.ts:1-40`, `playwright.config.ts:1-40`, `package.json:16-18` | Low | 2 |

Sorted by importance (High → Medium → Low); ties broken by discovery order.

## Test/tooling status (verbatim results)

### `pnpm test:unit`

```
$ vitest run --config vite.config.ts

 RUN  v4.1.0 C:/Users/zentala/code/MoveUp

 Test Files  37 passed (37)
      Tests  248 passed (248)
   Start at  02:35:59
   Duration  8.65s (transform 22.28s, setup 20.26s, import 39.47s, tests 7.90s, environment 93.49s)
```

37 test files, 248 tests, all passed.

### `pnpm typecheck`

```
$ tsc --noEmit
```

Clean — no output, exit 0.

### `pnpm lint`

```
$ eslint src/
'eslint' is not recognized as an internal or external command,
operable program or batch file.
[ELIFECYCLE] Command failed with exit code 1.
```

Broken. `eslint` is not a `devDependency` in `package.json` and no eslint config file
exists in the repo (`.eslintrc*` / `eslint.config.*` both absent). See Finding #1.

### `pnpm test:coverage`

```
ERROR: Coverage for lines (76.53%) does not meet global threshold (80%)
ERROR: Coverage for functions (68.09%) does not meet global threshold (80%)
ERROR: Coverage for branches (66.01%) does not meet global threshold (75%)
[ELIFECYCLE] Command failed with exit code 1.
```

The threshold is defined and does fail correctly when run directly — but it is not
invoked by `pnpm build`/`pnpm tauri:build` (those call `test:all`, which is
`test:unit && test:unit:rust` only). See Finding #2. Weakest files: `useDesk.ts`
(0.78% lines), `DebugSection.tsx` (11.53%), several settings sub-panels (25-60%).

### `tests/integration` and `tests/e2e`

Not run as part of this review (both require a live app/driver process — see Finding
#16). Content inspected and appears current (commands referenced in
`tests/integration/session-flow.test.ts` — `inject_reading`, `get_session_state` — both
exist in `src-tauri/src/commands.rs`), so "rotted" would be the wrong word; "not
continuously verified" is accurate.

### `justfile`

Absent at repo root — confirmed via `ls justfile` (No such file or directory).

## Recommended target structure

Not a rewrite — the shape is mostly right. Concrete moves, roughly in priority order:

1. **Unify the data layer.** Extract a transport-agnostic reducer (e.g.
   `src/hooks/deskStateReducer.ts`) that owns the `DeskState`/`sittingSeconds`/
   `transition`/`previousSession` logic currently duplicated in `useDesk.ts` and
   `useRemoteDesk.ts`. Each hook becomes: subscribe to its transport (Tauri `listen`+
   `invoke` polling vs `WebSocket`+REST fallback), and dispatch normalized events into
   the shared reducer. This also fixes Finding #4 (untestable `useDesk.ts`) because the
   reducer becomes testable independent of Tauri.
2. **Delete the dead layer**: `AppProgressBar.tsx`, `HeightRail.tsx`,
   `SessionProgress.tsx`, `TodayStats.tsx`, `TransitionBanner.tsx` and their tests;
   `src/overlay/main.tsx` + `overlay.html` + its `vite.config.ts` build-input entry.
   Zero behavior change, removes ~2000 lines and one build artifact, and stops dead
   files from padding the "248 tests passed" and coverage numbers.
3. **Fix the tooling gate, not just the number**: either install `eslint` + add a flat
   config (matching whatever the rest of the ecosystem uses) or remove the `lint`
   script until it's real — a script that always fails is worse than no script. Wire
   `test:coverage` (or its threshold check) into `pnpm build`/`tauri:build`, or lower
   the documented claim in `CLAUDE.md` ("≥80%, enforced by vite.config.ts coverage
   gate") to match reality until it's actually wired in.
4. **Introduce one shared codegen boundary** for Rust↔TS types — `ts-rs` derive on
   `AppConfig`/`SessionStateDto`/`StateChangedPayload`/etc., emitted into
   `src/generated/` and re-exported from `types.ts` — so a Rust field rename becomes a
   TS compile error instead of a silent runtime mismatch.
5. **Consolidate formatting helpers** into `src/utils/format.ts`: move `formatDate`,
   `formatRefreshedAt`, `formatRangeLabel`, `formatIdleTime`, `formatTime` there (or
   compose them from the two existing `formatDuration`/`formatDurationShort`
   primitives) so there is one place to fix a date/duration bug.
6. **Sync `.arch/UX-FLOW.md`** with the Steps widget and the click-timeline-to-open-
   Analyst interaction (Finding #8) — a 15-minute doc fix that restores the file's
   claim to being the single source of truth.

## GAPS

- Did not run `tests/integration/*` or `tests/e2e/*` against a live app — no Tauri
  runtime or `tauri-driver` was started for this review, so their actual pass/fail
  status today is unverified (content freshness was checked by cross-referencing
  command names against `src-tauri/src/commands.rs`, not by execution).
- Did not audit `src-tauri/` Rust code itself — this review is scoped to the React/TS
  frontend per the task. The DTO-duplication finding (#5) is inferred from the TS side
  plus reading `config.rs`'s `AppConfig` struct, not a full Rust-side audit.
- Did not check bundle size or runtime performance (`.perf-baseline.json`,
  `test:perf`) — out of scope for an architecture review but adjacent to Finding #7
  (dead overlay entry adds to build/bundle size).
- Accessibility assessment (Finding #13) is a static grep-based signal (aria attribute
  presence, absence of `:focus-visible`), not a screen-reader or keyboard-only pass —
  a real a11y audit would need manual or automated (axe-core) testing.
- Did not verify whether CI (GitHub Actions or similar) exists and runs any of these
  scripts automatically — no CI config file was found in the repo during this review,
  but a dedicated CI-config search was not exhaustive.
- Coverage percentages reported are from a single `pnpm test:coverage` run on the
  current `main` working tree (uncommitted changes to `.plan/STATE.md` and one
  JOURNAL.md present per git status, neither touching `src/`) — not re-verified across
  multiple runs for flakiness.
