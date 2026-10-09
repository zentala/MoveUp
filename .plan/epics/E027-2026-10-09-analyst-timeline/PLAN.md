# E027: Trustworthy timelines and a unified Analyst design

## Outcome

Popup and Analyst show the same chronological desk-state history at different scales. Analyst loads useful current-day data immediately, exposes exact interval details, and uses the popup's compact monospace instrument style with charcoal surfaces.

This serves the owner's vision of a configurable desk ergonomics application whose measurements and break credit are easy to understand. The current direction is captured in `../../VISION.md`.

## Scope

- Reconcile root `DESIGN.md` with the approved charcoal appearance, quiet badge borders and Lucide line icons.
- Correct credited-load versus actual-interval confusion without changing engine credit, scoring or persisted history.
- Share timeline rendering, interval normalization and state colors between compact and detail views; retain mode-specific navigation.
- Diagnose and fix confirmed initial-load/current-day/standing-visibility defects.
- Show selected-day heading, immediately Range/refresh, large timeline and one daily donut tab per day directly below in one section. Start with seven days, reserving loading geometry immediately. Remove main Desk Height Timeline and Day at a Glance.
- Explain day composition against the complete local calendar day, with unobserved time explicit.
- Review and exercise relevant popup, Analyst and settings interactions; report other findings separately.
- Keep recorded signed points and change-event provenance truthful; do not invent a 0–100 scale, goal or noise filter. Secondary trajectory/histogram choices await operator discussion; range-wide KPI trends sit outside the selected-day context.

## Correction prerequisite and retained evidence

The operator rejected the previous browser preview's extra panels. Revised UI implementation is held until Astra user stories, metric investigation, UX audit/internet skill review and this corrected plan exist. Those artifacts now exist privately in the owning reports directory; they remain untracked because they contain private operational context. The superseded UI candidate `de3ff28` is preserved, unaccepted and unintegrated. Compatible timeline `f8e77d0` and data `45d3fac` candidates remain subject to independent acceptance. Round 1 confirmed uncertainty/presentation defects; see `review-rounds.md`. Do not restart a second queue.

Publication hold: supervision explicitly prohibits pushing current main until owner-safe preparation addresses private paths in earlier unpublished history. Preserve audit references and local commits. Notify supervision before planned push or overlapping source integration; no force-push or unilateral rewriting of foreign work.

## Execution

Total: **39 points**. Route: direct bounded workers supervised in Herdr, followed by independent source verification. Six tasks: design contract (5), data/loading correction (5), shared timeline (8), connected donut navigation/theme/icons (8), browser and integration verification (5), mixed-break history recording (8). The history task was added after source and read-only runtime evidence showed mixed breaks persisted as one final state. No AO threshold is reached. Writes are isolated by explicit file ownership and worktrees for implementation.

## Acceptance

1. A 46-second current sitting interval with 1855 seconds credited load renders 46 seconds on the timeline; the load timer still uses 1855.
2. Completed Away intervals remain visible with exact local start/end/duration; compact expansion supports click and keyboard when nonempty.
3. Standing has the same distinguishable green in both modes; missing observations are never silently classified as Away.
4. Today is selected on first open when in range; loading, empty and error states are distinguishable. Changes with identical row counts still update charts.
5. Timeline and daily donut tabs form one visual/interaction composition with accessible selected state and keyboard navigation.
6. Donut denominators use local day boundaries, including DST; observed state percentages and unknown remainder are stated accurately.
7. Analyst uses canonical typography, surface, semantic-color and icon contracts. No new independent palette.
8. Regression tests, TypeScript checks and applicable existing gates pass. Independent source review and rendered browser evidence are recorded; native checks are marked separately.

## Decisions and ADRs

- ADR 008: proportional credit remains unchanged and separate from actual timeline duration.
- ADR 012: retain the separate Analyst window.
- ADR 013: retain one date authority; replace grouped bars with connected daily donut tabs as newly requested.
- ADR 015: keep ergonomics engine semantics unchanged; presentation adapters own history normalization.
- ADR 016: reuse existing Recharts support where appropriate, without a second chart library.
- Root `DESIGN.md` is the visual authority; record the new target before implementing it.
- Applicable ecosystem patterns: disposable bounded scout for diagnosis; explicit loading/error states ensure silence cannot pass for success. Follow existing repository/component ownership and public-artifact boundaries.

## Data impact

Read-only history queries; no migration, correction, restore or deletion of live data. Review reports with personal runtime details stay untracked locally.
