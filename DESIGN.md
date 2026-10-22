# MoveUp design contract

Updated 2026-10-09. **Accepted target; implementation pending.** Canonical visual contract for popup and Analyst. The operator accepted the tokens below. Source baseline: `6e81da375fe4265325af2f33d2c1c1fac263640f` and working files read in this review. Browser/live QA: **NOT_CHECKED**. Independent coordinator review is required.

## Direction and scope

MoveUp is a modular desk ergonomics experimentation platform. Popup: understand the current state at a glance. Analyst: inspect the selected day's observations. Preserve the compact precision-instrument feel and popup monospace typography. Unify charcoal surfaces, posture semantics and Lucide line icons. Retain React/TypeScript, Tauri, SVG charts and Recharts donuts; no stack replacement or ergonomic-engine change.

Preserve popup 420×240, frameless; Analyst 1280×800, decorated and resizable ([window decision](.arch/ADR/012-analyst-dashboard-separate-window.md)). Settings, Catalog, help and menus share chrome tokens. Native tray/overlay urgency remains separately governed by CommunicationPolicy.

## Accepted tokens

Preserve existing token names as the migration interface. New `--state-*` names are target additions, not existing exports. CSS and `chartColors` must resolve to one palette rather than independent hex inventories.

| Token | Target | Role |
|---|---|---|
| `--panel-base` | `#161616` | Page/window background |
| `--panel-raised` | `#262626` | Panels and connected selected day tab |
| `--panel-elevated` | `#303030` | Hover and inactive day tabs |
| `--panel-overlay` | `#393939` | Menus, tooltips and pressed controls |
| `--rail-subtle` | `#393939` | Decorative separators, quiet badge borders |
| `--rail-default` | `#525252` | Secondary separators, not sole control affordance |
| `--rail-emphasis` | `#6f6f6f` | Required control boundary on base/raised |
| `--rail-active` | `#78a9ff` | Focus outline |
| `--ink-primary` | `#f4f4f4` | Primary text/readouts |
| `--ink-secondary` | `#c6c6c6` | Labels |
| `--ink-tertiary` | `#a8a8a8` | Readable metadata |
| `--ink-muted` | `#6f6f6f` | Disabled text only |
| `--state-sitting` | `#fa4d56` | Sitting posture, never urgency |
| `--state-standing` | `#42be65` | Standing posture |
| `--state-away` | `#8d8d8d` | Explicitly observed Away |
| `--state-walking` | `#78a9ff` | Existing Walking state, separately labeled |
| `--state-unknown` | `#525252` | Unobserved elapsed time, hatched |
| `--state-future` | `#393939` | Future time, outlined/unfilled |
| `--signal-warn`, `--signal-alert` | `#ffc107`, `#f44336` | Existing urgency meanings |
| `--signal-ok` | `#4caf50` | Existing KPI evaluation, not posture |
| `--beam`, `--beam-hi` | `#d97706`, `#f59e0b` | Retained height/hardware accent, not Standing |

Timeline `--tl-sitting/standing/walking/away` and matching `chartColors` fields alias state tokens. Use opaque chart fills and ink text next to state swatches. Legacy amber/clinical skin settings need an explicit migration decision before modifying persisted preferences; they must not silently override the accepted shared posture mapping. Urgency labels/indicators never recolor historical posture.

Use opaque base/raised backgrounds for reconciliation. Existing popup translucency/blur is observed, not required; retaining it requires measured contrast over actual desktop backgrounds. No decorative glow or depth shadows. Quiet badge: raised fill, subtle 1px border, readable label and small swatch; border alone conveys no meaning.

## Typography, density and icons

Preserve popup `--font-mono: ui-monospace, "Cascadia Code", "Fira Mono", "Consolas", monospace`; apply to Analyst, SVG labels, controls and menus. No font download. Base 13px/1.5; metadata 11px/1.5; compact labels 10–11px, tracking 0.08em, uppercase only for short labels. Readouts 26px/1.1, weight 700, tabular numerals; panel title 14px/600, page title 20px/600. Essential 8–9px labels in existing source must rise to at least 10px.

Preserve 4px spacing basis and compact gaps 4/7/8/11/13px; section padding 11–13px, outer padding 12–16px. Preserve radii `--r-sm:3px`, `--r-md:5px`, `--r-lg:8px`.

Use named `lucide-react` imports: Settings, Share2, ChevronLeft, ChevronRight, RefreshCw, CircleHelp, X. Default size 16px, strokeWidth 1.5, currentColor; replace action glyphs/emoji. Decorative icons are aria-hidden. Icon-only buttons have accessible action names and hover/focus tooltips. Hit areas ≥24×24px, preferably 28×28px popup and 32×32px Analyst. Dependency installation is pending. Chart posture swatches retain text labels; do not invent posture pictograms.

## Timeline and connected day tabs

Explorer order: **selected-day heading**, immediately **Range/refresh controls**, **large selected-day horizontal timeline**, then **smaller daily composition donut tabs directly below** in one coherent section. Show seven days by default, including reserved loading geometry; the row scrolls at narrow widths. One donut per day replaces that day's grouped bars. Remove Day at a Glance and Desk Height Timeline from the main view. Range-wide KPI trends may follow in a separate section. Operator chose to keep both score trajectory and break-credit histogram in a keyboard-operable expandable details section outside the main day composition, after fixing calculations/descriptions. Show signed recorded profile points with real local timestamps and gaps, not a normalized score. Credit categories describe recorded load effects and explicit unknown coverage, not universal duration thresholds. No intervening grid or detached sticky bottom navigator. One range/selected-day model governs every chart, retaining [ADR 013's single date authority](.arch/ADR/013-date-navigator-merge.md) while superseding mini bars and placement.

Panel: raised background, 12px padding, 8px outer radius. Target strip height 96px including ticks, posture band ≥56px. Default visible domain: full selected local day fitted to width. Optional neighboring-day pan/zoom preserves proportional elapsed-time scale and explicit midnight boundaries; no compressed side previews. Show selected date, timezone, legend and Now when applicable.

Daily donut target: 48px diameter, 6px ring; tab width ≥72px, weekday/date and observation coverage. Single horizontally scrollable row, no wrapping. Selected tab and panel share `#262626`, no vertical gap; remove shared top seam under the selected tab. Others use `#303030`, subtle 1px separators, 4px gaps. Selection has readable date/semantics, no gold glow. Focus remains visible across the seam. Keep selected tab visible when resizing; stack lower charts if needed, avoiding whole-page horizontal overflow.

Popup uses the shared compact timeline fitted to its width without Analyst navigation chrome. Activating it opens Analyst on the same local day; dismissal returns invoking focus.

## Shared data and full-day composition

One timeline component has compact/detail modes and one normalization/aggregation layer serves both modes and day donuts. Modes change dimensions, ticks and navigation, never durations, coverage or state identity. Source adapters stay outside rendering. Target input: half-open intervals `{startMs,endMs,state,provenance}`, local-day bounds, timezone and injectable `nowMs`; explicit Unknown/Future coverage accompanies confirmed states.

For local date D, start is local midnight D and end is local midnight D+1 by calendar arithmetic in the desk's authoritative timezone. `daySeconds=(endMs-startMs)/1000`: normally 86400, DST 82800/90000. Never select days by timestamp string prefix, UTC today, or fixed 24-hour increment. Show timezone; repeated hours need offset-disambiguated tooltips. Authoritative timezone provenance remains a source dependency.

Sort, validate and deduplicate records; clip at boundaries/Now and split midnight crossings. Session intervals take precedence over snapshot approximations only where their producer semantics establish individual states; legacy mixed-break records are not proven posture intervals. Label recorded provenance and uncertainty; never silently reconstruct historical states or mutate the database. Reject/report irreconcilable overlap rather than double counting. Chronological Sitting uses raw observed time, never credited `limitUsedSecs`; credit remains for timer/urgency ([engine ownership](.arch/ADR/015-pure-ergo-engine.md)).

Snapshots require a bounded observation horizon grounded in producer cadence. If none is established, mark coverage uncertain/pending rather than filling a long gap. Unknown enums, missing samples, disconnected sensors or stopped app mean Unknown, not Away. Live coverage ends at Now or last confirmed horizon, whichever is earlier. Detail tooltips expose provenance. Producer horizon is pending, not guessed here.

### Point observations without interval coverage — 2026-10-22

A timestamped snapshot proves a reading at an instant, not a sixty-second posture interval. Where no source-qualified interval/horizon exists, keep elapsed coverage Unknown. The shared compact/detail timeline may show a distinct observation tick at the actual timestamp: fixed2 CSS px wide and6px high at the top edge, not a full-height filled span. This precision-instrument tick is a location cue, never a minimum duration or area share. Use existing posture colors only for a known-quality/known-state observation; otherwise use readable ink-secondary rather than presenting an uncertain state as confirmed Away or Sitting. No new palette, glow, animation or control.

Keep the underlying Unknown hatch and all duration/ring/coverage totals unchanged. Never join ticks into coverage, extend a lone point to now/nominal cadence or infer contiguous-standing maximum from points. Clip to the selected time window and actual now; an invalid future reading is not confirmed elapsed history. Dense ticks may coincide visually, while existing accessible details retain the observations and their uncertainty. Use the existing keyboard-accessible observation list/details: identify the instant, reported state/quality, available source time/offset and **duration unknown**. A short visible explanation in detail distinguishes observation ticks from filled intervals; the compact label/expansion must preserve that distinction without adding another button.

Intent: let the operator see that a reading exists without mistaking it for time spent in that state. Intervals remain the main visual; ticks and existing monospace/ink details are secondary. Continue current charcoal layers/spacing/geometry outside the narrow tick itself. This decision accepts the bounded representation for implementation, not its actual rendered fit: independently inspect wide/narrow real host, clipping, small-point discoverability and keyboard details before UI delivery. The author had already coded a full-height marker proposal before this decision; preserve that chronology, revise it rather than claiming design-before-code compliance retroactively.

Each daily ring uses **the full actual local calendar day**: `stateShare=confirmedStateSeconds/daySeconds`. Unknown is uncovered elapsed time; Future is time after Now in today/future days. Confirmed states + Unknown + Future equal daySeconds before rounding. Past normal-day example: 2h Sitting + 1h Standing + 1h observed Away + 20h Unknown = 24h, never 50% Sitting or 21h Away. At noon today with 3h confirmed coverage: 9h Unknown + 12h Future remain on a normal day.

Unknown is hatched/labeled; Future outlined/unfilled/labeled; Away solid gray. Accessible descriptions and focus/hover details expose durations, percentages, denominator and coverage. Center shows date or coverage. Preserve tiny durations mathematically; enlarge hit targets rather than visual proportions. Rounding never fabricates observations.

Daily composition is distinct from secondary metrics. Standing share of Sitting+Standing must name that denominator and use raw counters, never credited load. Missing raw values remain unknown. The engine's signed motivational points have no 0–100 bound; do not clamp values or display them as a percentage gauge. Recorded change events are not confirmed body movements; describe the engine definition and provenance without an invented goal. Missing counters/scores stay unknown. Remove the three-gauge Day at a Glance panel from the main view. Retain [Recharts decision](.arch/ADR/016-recharts-for-kpi-donuts.md) where applicable: explicit size and actual SVG text; no second chart library required.

## Interaction, accessibility and asynchronous states

Default: readable ink, quiet separators. Hover: elevated fill/primary text; pressed: overlay fill. Selected: persistent connected raised tab and aria-selected. Disabled: native disabled where applicable, muted text, no hover affordance; explanations remain readable outside disabled controls.

Focus-visible: 2px rail-active outline with 2px offset, never removed/clipped. Required control boundaries on elevated/overlay surfaces use ink-tertiary or another measured ≥3:1 indicator; rail-emphasis meets 3:1 only on base/raised. Badge dividers are decorative, not sole interactive affordances. Red/gray-state descriptions use ink text on elevated backgrounds.

Day navigation: labeled tablist, tab roles, roving tabindex, linked tabpanel/stable IDs. Left/Right and Home/End move focus; Enter/Space selects (manual activation avoids load delays). Tab enters selected tab, then panel controls. Day updates retain initiating-tab focus. Catalog/Explorer use the same pattern. Help opens on hover/focus, Escape closes; dialogs restore invoking focus. Provide a keyboard-accessible timeline interval list/details rather than pointer-only tooltips or hundreds of required tab stops.

Loading reserves geometry with aria-busy and readable status; never show a zero ring or empty claim before success. Ready-empty past day: “No observations recorded”, 100% Unknown; today splits elapsed Unknown/Future. Error: readable explanation + Retry, selected date preserved; retained successful data must say stale and show timestamp. Invalid/overlapping time data is a data-quality error, not plausible zero. Partial observations show honest coverage. Announce meaningful updates politely, not every clock tick.

Reduced motion disables pulse, arc animation and smooth scrolling; selection applies immediately. No decorative continuous animation. WCAG 2.2 AA is a verification target, not certified. Calculated opaque token minima: tertiary text 4.86:1, focus 4.90:1; Sitting/Away fills 3.44:1/3.48:1 against surfaces. Use ink labels and separators/other cues between touching states. Computed contrast, keyboard, targets and overflow require rendered verification.

## Observed source baseline / pending reconciliation

| Evidence | Observed | Pending target |
|---|---|---|
| [globals.css](src/styles/globals.css):16–53,121,244–247 | Warm tokens, translucent shell, Standing dot uses amber beam | Charcoal opaque layers, posture aliases |
| [AnalystWindow](src/analyst/AnalystWindow.tsx):90–92; [chart-utils](src/analyst/charts/chart-utils.ts):33–44 | Explicit sans-serif, navy surfaces, separate state hexes | Monospace/shared palette; migrate overrides |
| [ExplorerTab](src/analyst/ExplorerTab.tsx):155–175; [DateNavigatorColumn](src/analyst/charts/DateNavigatorColumn.tsx) | Timeline, grid, bottom navigator; mini bars/amber selection | Connected donut tabs immediately below timeline |
| [OneBarTimeline](src/widgets/one-bar/OneBarTimeline.tsx):31,49–77 | Unknown→Away; observed-sum widths/minimum 0.5%; live Sitting uses credit | Shared raw chronology, truthful widths/coverage |
| [timeline-segments](src/analyst/charts/timeline-segments.ts):40–64; [timeline-utils](src/analyst/charts/timeline-utils.ts):77–87 | Extends to next sample; separate fixed-sample aggregation; fixed-day boundary steps | Bounded horizon, single aggregation, calendar bounds |
| [DateNavigator](src/analyst/charts/DateNavigator.tsx):44–45; [TimelineDetail](src/analyst/charts/TimelineDetail.tsx):40–60 | UTC today; width uses fixed DAY_MS while scale helper uses calendar end | Local today and consistent calendar scale |
| [explorer-day-kpis](src/analyst/explorer-day-kpis.ts):51,113; [KpiDonut](src/analyst/charts/KpiDonut.tsx):58 | Date prefix; standing/active ratio; navy remainder | Full-day composition separate from KPI gauges |
| [OneBarWidget](src/widgets/OneBarWidget.tsx); [ChartCard](src/analyst/charts/ChartCard.tsx); package.json | Glyph share/settings/help; no lucide-react dependency found | Named line icons/accessibility; installation pending |

Source precedence checked: App imports globals; OneBarWidget imports one-bar CSS and skin class; Analyst explicitly overrides inherited font. These are source findings, not computed styles. No live app, screenshots or application tests ran for this documentation change.

## Decision history and acceptance

### Native Windows tray contrast — 2026-10-21

Use a dark desk silhouette (#161616) on the light Windows system tray and the
existing white silhouette on the dark tray. Follow SystemUsesLightTheme,
independently of the application's window theme. Preserve the current image,
geometry, alpha, status colors, dot/no-dot and blink phase; do not replace PNG
assets or change CommunicationPolicy. Recolor only the canonical white silhouette.
Initial rendering and subsequent system-theme changes use the same transformation.
Coalesce checks through the existing event loop/runtime, at most once per 500 ms;
retain the last known theme if the system value cannot be read. Keep the complete
tray-handle lookup/use/drop on the main thread. Native light/dark inspection remains
NOT_CHECKED until independently observed; this decision changes no popup/Analyst theme.

| Date | Change / previous direction | Reason | Status |
|---|---|---|---|
| Prior contract, date unknown | Warm oiled-steel, amber active/Standing, warm ink, monospace and borders-only | Bespoke desk instrument, not generic SaaS | Historical rationale retained; palette/Standing superseded |
| 2026-10-09 | Charcoal replaces warm popup/navy Analyst; retain mono/density/radii | Operator direction and unified instrument identity | Accepted; implementation pending |
| 2026-10-09 | Red Sitting/green Standing/gray Away; separate urgency; Lucide actions | Operator direction, stable meanings/quiet chrome | Accepted; implementation pending |
| 2026-10-09 | Timeline first, connected donut tabs replace bottom mini bars | Operator direction, selected-day association | Accepted; ADR 013 presentation superseded, date authority retained |
| 2026-10-09 | Full local day + Unknown/Future; shared compact/detail semantics | Operator direction, truthful observation coverage | Accepted; horizon/timezone evidence pending |
| 2026-10-22 | Zero-duration snapshot ticks (2px by6px at strip top) replace guessed sample spans/full-height point stripes | A reading proves an instant, not observed duration; Unknown coverage and existing posture colors stay truthful | Accepted bounded representation; independent actual-host review pending |
| 2026-10-22 | Move communication profile selection and editing to the Notifications tab; remove the three legacy notification toggles | The toggles are absent from `AppConfig` and do not affect policy; the existing profile selector, JSON editor, duplicate and reset actions preserve configuration through the active communication profile | Accepted for E027-T12; existing settings chrome and controls retained |
| 2026-10-09 | Day heading then Range, timeline and one ring per day; remove main height/Day at a Glance | Operator correction supersedes extra panels; seven-day default within requested 5–7 | Accepted correction; stories/UX/plan precede revised implementation |
| 2026-10-09 | Preserve semantic/amber/clinical preference and apply selected skin to popup and Analyst | Shared state identity without silently changing stored preference | Implementation interpretation; visual verification pending |
| 2026-10-09 | Signed points and recorded event definitions replace score/100 and assumed goal semantics | Source evidence disproves fixed score scale and human-transition interpretation | Data truth accepted |
| 2026-10-09 | Keep score trajectory and break-credit histogram in expandable details after correcting meaning/data | Explicit operator selection after explanations | Accepted; main day timeline stays primary |

Implementer supplies source and tests; coordinator independently accepts. Required evidence: shared tokens/font, connected tabs, equal interval/ring totals, honest states, named icons, fixtures for DST/midnight/gaps/live/unknown enums. When authorized preview exists, verify popup 420×240, Analyst 1280×800 and narrow resizing; keyboard/focus/overflow/reduced motion and computed contrast. Until then live QA stays NOT_CHECKED. This author does not certify implementation. Instruction-file link to this contract is pending coordinator-owned work outside this write scope. [Private review](.plan/reports/2026-10-09-design-review.md).

## Official references

[Carbon colors](https://www.carbondesignsystem.com/building-blocks/foundations/color/overview) and [layering](https://www.carbondesignsystem.com/building-blocks/foundations/color/guidelines) inform neutral layers/semantic roles; [Carbon tabs](https://www.carbondesignsystem.com/building-blocks/core/components/tabs/guidelines) informs selected-tab/panel matching and horizontal scrolling. Below-panel donut tabs are MoveUp's adaptation. [Lucide React](https://lucide.dev/guide/react/) supports named imports. [WAI tabs](https://www.w3.org/WAI/ARIA/apg/patterns/tabs/) and [WCAG 2.2](https://www.w3.org/TR/WCAG22/) inform keyboard/contrast targets. Checked 2026-10-09; exact tokens are operator-accepted MoveUp choices, not a Carbon compliance claim.
