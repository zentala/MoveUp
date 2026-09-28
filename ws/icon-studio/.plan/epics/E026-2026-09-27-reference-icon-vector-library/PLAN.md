---
formatVersion: 1
type: epic
epic: E026
created: 2026-09-27
status: planning
readiness: drafting
points: "?"
agent: general-purpose
blocked-by: "Inventory and crop review must establish the unique icon count before final sizing and dispatch."
---

# E026: Vector library from the June 8 reference boards

## TLDR

Recover the *shapes*, line grammar and sparse colour accents of the icons in
three user-supplied Healthy Balance boards as editable, individual SVGs in
the `MoveUp/ws/icon-studio` workspace. Start with an audited crop inventory and a four-icon proof. Use scripts
for repeatable preprocessing and line extraction, then correct geometry against
the original crops. Keep the current hand-authored icon studies separate and
leave the running app and tray icon unchanged until the user selects assets.

## Goal

Build a reviewable SVG source library for the distinct icons visible in all
three boards. The user can identify each icon by its source board and printed
number, compare the SVG with its original crop, and choose which shapes to use
in MoveUp later. The workspace owns this plan at `ws/icon-studio/.plan/`;
application integration requires a separate decision and task.

## Sources and current state

- `ws/icon-studio/references/desk-and-activity.png` (1448×1086): numbered
  desk variants 01–12 and activity/status icons 13–30.
- `ws/icon-studio/references/extended-system.png` (1536×1024): numbered
  system, suggestion, body, rhythm, progress, brand and utility sections.
- `ws/icon-studio/references/brand-book.png` (1536×1024): logo, icon library,
  palette and examples; some motifs repeat the first two boards.
- `ws/icon-studio/reference-line.mjs` currently generates 14 **approximate**
  studies, not recovered geometry. `ws/icon-studio/generate.mjs` already
  exports SVGs and `preview.html` renders a contact sheet. These changes are
  local and uncommitted at planning time; confirm their Git status before
  execution.
- The older Electron product pack is archived at
  `.plan/archive/healthy-balance-spec/`. Current MoveUp is Tauri/React/Rust.
- Inkscape was not found on PATH or in the checked Windows program locations on
  2026-09-27. Its CLI can automate SVG actions and ordinary object tracing, but
  the availability of headless *centerline* tracing on the target installation
  is unverified. Do not make that action a hard prerequisite without a probe.

The PNG files have no editable SVG paths or prompt metadata. Their labels and
layout are usable as indexing evidence, not as automatic crop coordinates.

## Scope

1. Inventory every visible glyph position across the three boards, including
   numbered slots and unnumbered brand-book or UI-example motifs. Assign each
   position a stable ID and a disposition: `export`, `duplicate`, `exclude` or
   `unreadable`. Record board, section, printed number when present, label,
   bounding box, source hash, confidence and review verdict. Duplicates point
   to their canonical glyph; exclusions and unreadable positions have a reason.
   Typography, colour swatches, photographs and UI chrome can be excluded, but
   embedded icon glyphs still need their own inventory position. Count all
   positions by board and section so that every visible position is accounted
   for, even when it does not produce an SVG.
2. Generate one crop per `export` position and an overlay marking every
   inventoried position and disposition on each original board. Review the
   overlay against the whole board, including unnumbered areas, and correct
   missing or ambiguous boxes before batch tracing. Computer vision/OCR may
   propose boxes, but does not approve them.
3. Prove extraction on desk 01, raise 02, seated desk 10 from the first board
   and one coloured symbol from the extended board. Compare outline tracing,
   centerline tracing and shape-guided reconstruction against the source crops.
4. Build deterministic SVG output with editable paths, a consistent viewBox,
   no embedded raster image, labels or filters, and separate semantic accent
   colours. Keep extracted geometry separate from the existing approximate
   families in `ws/icon-studio/icons/`.
5. Vectorize the accepted inventory in batches and produce a contact sheet with
   source, mask, uncorrected trace and corrected SVG. Register the source crop
   and SVG on the same canvas with a recorded aspect-preserving scale and
   translation; show a blended overlay as well as side-by-side renders at native
   crop size and 48/24/16 px. Review the silhouette, intentional gaps, line
   joins, endpoints, proportions and colour-accent boundaries per icon.
6. Provide a selection manifest for the user. Do not replace MoveUp runtime
   icons or redesign the public website in this epic.

## Approaches

| Approach | Effort | Risk | Reuse | Advantages | Limits |
|---|---|---|---|---|---|
| A. Manual paths only | Large | Medium | Existing `reference-line.mjs` and generator | Clean editable geometry and deliberate gaps | Slow at this icon count; easy to drift from source |
| B. Automatic bitmap trace only | Medium | High | Inkscape/Potrace or VTracer | Fast reproducible first pass | Often returns filled outlines, preserves image artefacts and loses semantic stroke structure |
| C. Hybrid crop → mask → centerline → geometric correction | Large | Medium | Existing icon studio; Inkscape visual review; `scikit-image`/OpenCV if the pilot supports them | Repeatable batch processing with source-shape review | Requires crop manifest and human correction |

**Recommended:** C, subject to the four-icon proof. A remains the fallback for
icons where line extraction breaks junctions or intentional gaps. No new
dependency is accepted before that proof; record the selected tool and its
version in the implementation handoff.

## Crop and vector contract

- Use the visible board section and printed ID as the stable source identity.
  Give unnumbered positions a stable board/section/local ID. The inventory is
  a coverage ledger for all visible glyph positions, not only the icons chosen
  for export. A full-board overlay must mark exports, duplicates, exclusions
  and unreadable positions differently; the reviewer compares it with the
  original board and signs off each section's count and dispositions.
  The first pass may detect candidate boxes from grid positions and connected
  regions; OCR assists labels only. Each proposed rectangle gets an overlay on
  the full board and a crop preview. Missing, overlapping or label-containing
  crops fail review; uncertain boundaries stay `unknown`. Store label/number
  rectangles if an automated overlap check is desired; otherwise record a
  visual label-exclusion verdict for each crop. Crop bounds alone cannot prove
  that a label is absent.
- Separate white strokes and blue/green/amber/red accents before tracing.
  Thresholding and small-object removal can suppress speckles, but must not
  silently close intentional gaps. Keep source and mask next to the SVG.
- Ordinary Potrace/VTracer region traces are comparator outputs; an editable
  line icon needs centerline paths or hand-authored geometric primitives.
  Snap repeated desk tops, legs, feet, circles and arrowheads to a common grid
  after tracing, while checking against the crop. Never claim pixel-perfect
  source recovery from an AI raster.
- Output one standalone SVG per unique glyph and a manifest link back to its
  source. Render at 48, 24 and 16 px; 16 px is a legibility check, not a demand
  that every detailed app icon become a tray asset.
- Preserve the registration transform used by each source/SVG comparison.
  Never stretch either image independently to fit the review cell: that can
  disguise proportion errors. Human review decides shape fidelity; pixel
  similarity can flag differences but cannot automatically approve a trace.

## Acceptance criteria and evidence

| Criterion | Proof |
|---|---|
| Every visible glyph position on the three boards is accounted for | Per-board/section ledger of `export`, `duplicate`, `exclude` and `unreadable` positions, with reasons/targets and a full-board disposition overlay manually checked against each original |
| Every `export` position has one crop and one SVG | Generator check compares exported manifest IDs, crop files and SVG files; zero matches, missing IDs and stale extra outputs fail |
| Extraction preserves intended shapes and colour roles | Four-icon pilot review, then registered source/mask/trace/final comparison and per-icon rubric verdict for every batch |
| SVG library is editable and reproducible | XML validation; reject `<image>`, `<text>`, filters and external URLs; generate twice and compare hashes |
| Icons remain readable at intended sizes | Render comparison at 48/24/16 px; record any icon that needs a separate simplified 16 px form |
| No app behaviour changes without selection | Git diff shows no runtime tray, UI component or website integration in this epic |

## Test strategy

- `ws/icon-studio/tests/manifest.test.*`: unique IDs, valid board hash and
  rectangle bounds, valid dispositions and reasons, duplicate links resolve,
  per-board/section disposition totals match the signed coverage ledger.
  If label/number rectangles are recorded, assert that `export` crop boxes do
  not overlap them; otherwise label exclusion requires a visual verdict.
- `ws/icon-studio/tests/export.test.*`: one crop and SVG per `export` ID,
  valid XML, allowed elements and colours only, no embedded bitmap or external
  resource, stable output hashes across two runs, and no stale output IDs.
- `ws/icon-studio/tests/render-check.*`: browser or Inkscape raster export at
  48/24/16 px with nonempty alpha bounds. The pilot deliberately checks one
  desk, one arrow, one human and one coloured icon; a blank vector fails.
- Visual acceptance is a bounded human check, not a unit test: inspect the
  registered source/SVG overlay and separate renders. Record the reviewer,
  registration transform, label-exclusion verdict, rubric notes for silhouette,
  gaps, joins/endpoints, proportions and accents, plus `accepted`, `revise` or
  `unreadable` for every exported icon. A section cannot close with unreviewed
  or silently skipped positions; `revise` requires a new reviewed result.

## Decisions and ADRs

- `DESIGN.md` describes the current MoveUp instrument-panel style. This epic
  preserves the *reference-board* geometry as a selectable library and does
  not overwrite current tokens.
- `.arch/ADR/011-unified-sit-stand-walk-cycle.md` reserves tray/overlay
  escalation for the sitting limit; adding new runtime tray states requires a
  separate product decision. `.arch/ADR/010-notification-escalating-silence.md`
  constrains future signal use, not offline vector extraction.
- `.plan/decisions.jsonl` contains no existing decision selecting an icon
  family or a vectorization engine (checked 2026-09-27). If the pilot selects a
  new durable library/dependency, record that decision before integration.
- `harness/how-we-build/patterns.md` §11 (source files as truth) applies to the
  manifest and SVG sources; §12 (silence cannot pass as success) applies to
  nonzero per-section counts and explicit review status.

## Architecture impact

Only `ws/icon-studio/` and its build command change during the epic. No
runtime API, database, Tauri tray path, or external service changes. A later
adoption task must name actual app surfaces and add native-size verification.

## Vision impact

None — this creates optional design assets and does not change the current
MoveUp product promise or shipped behaviour. The repository has no
`PURPOSE.md` or `.plan/VISION.md` at planning time; do not invent product goals
from the archived Healthy Balance spec.

## Out of scope

- Recovering the original image-generation prompt or original vector files from
  PNG metadata; none was found.
- Automatically accepting all computer-vision crop boxes or trace output.
- Installing Inkscape or changing machine-wide configuration merely to make
  the pilot run.
- Shipping a new tray icon, changing application code, or redesigning the site.

## Planning status

The full icon count, category batch sizes and tool choice are deliberately
unknown until the inventory and four-icon proof. The proposed execution route
is AO because the full library exceeds 13 points, but dispatch remains closed
until task estimates, write sets, tool availability and review criteria are
reconciled in `HANDOFF.md` and task files. The AO base must be a committed ref
containing the workspace plan, scripts and source boards; an uncommitted working
tree is not available to a new worktree. Verify remote ancestry and reconcile
the planning-time local `main` lag behind `origin/main` before selecting it.
No executor has been launched.
