---
formatVersion: 1
type: handoff
epic: E026
status: draft
---

# E026 handoff — reference icon vector library

## Mental model

Read `PLAN.md` first. The original boards are
`MoveUp/ws/icon-studio/references/*.png`; the current hand-drawn SVGs are
approximate studies in `reference-line.mjs`, not the extraction output. Create
a new family, for example `icons/reference-trace/`, and keep every intermediate
crop/mask and review verdict addressable by the same manifest ID. Do not edit
`src-tauri/src/tray_icon.rs`, app components or the website.

The workflow answers the crop question explicitly: propose rectangles from
the numbered grid, write coordinates to the manifest, draw those rectangles
over the whole board, and visually approve each box before extraction. A model
or image-processing algorithm may suggest a box; it cannot silently define
the final icon boundary. The board labels and numbers are identities, not
pixels to include in the crop. The owning plan is
`MoveUp/ws/icon-studio/.plan/epics/E026-2026-09-27-reference-icon-vector-library/`.

## Proposed work packages

| ID | Work | Points | Importance | Evidence and dependency |
|---|---|---:|---|---|
| T01 | Source inventory, deduplication and crop-manifest overlays | ? | High | Every visible glyph position has `export`, `duplicate`, `exclude` or `unreadable` disposition; per-section ledger and full-board overlay are reviewed. Starts first. |
| T02 | Four-icon vectorization proof and Inkscape/tool probe | 8 | High | Compare centerline, region trace and manual geometry; select method. After T01 sample boxes. |
| T03 | Deterministic crop, mask, SVG and review-sheet pipeline | 5 | High | Manifest/export tests, no stale output, registered source/SVG overlay, no external SVG payload. After T02. |
| T04 | Desk and movement symbols | ? | High | Per-icon crop/vector comparison. After T03. |
| T05 | Human, activity and body symbols | ? | High | Per-icon crop/vector comparison. After T03. |
| T06 | Status, suggestions, progress and utility symbols | ? | High | Per-icon crop/vector comparison. After T03. |
| T07 | Brand-book unique motifs, dedupe and consistency pass | ? | Medium | Duplicate map and full three-board coverage. After T04–T06. |
| T08 | Final review, export package and selection manifest | 5 | High | Validated complete disposition ledger, per-icon shape rubric and user-facing contact sheet. Last. |

Unknown estimates are `?`, not zero. T01 establishes counts and changes T04–T07
into bounded task files with Fibonacci points before dispatch. If a category
exceeds 13 points, split it; a 21-point task is not allowed. The full epic is
expected to route through AO (>13 points); do not launch it while the total,
task write sets and prerequisite tool profile remain unknown.

## Order and boundaries

Wave 0: T01 and a narrow T02 sample, sequentially because crop approval is
the input to vectorization. Wave 1: T03. Wave 2: T04–T06 may be parallel only
after the category manifest has non-overlapping write sets. Wave 3: T07, then
T08. Each wave's point sum and the separate full epic total must be recorded
after T01; do not route based on only the remaining wave.

For T02, inspect the installed `inkscape --version`, `--action-list` and help
before invoking actions. The official Inkscape 1.4 notes document an
`object_trace` action for ordinary image tracing; they do not establish that
the GUI's centerline mode is scriptable in this environment. If Inkscape is
absent or that action is unavailable, compare a reproducible morphology/
skeletonization script with manual geometry. Do not install software or treat
an unobserved CLI result as a pass.

## Tests and review

Each task owns tests for its manifest IDs, crops and SVG output as described
in `PLAN.md`. No category is complete without its reviewed source/final sheet.
For every exported icon, store an aspect-preserving source/SVG registration
transform and inspect a blended overlay plus separate native-size renders.
Record silhouette, proportions, intentional gaps, joins/endpoints and accent
boundaries with an explicit `accepted`, `revise` or `unreadable` verdict.
Label/number exclusion is automated only when independently recorded label
rectangles support an overlap check; otherwise record a visual verdict.
Record the final accepted icon count, duplicate count, excluded and unreadable items,
command exit codes and visual verdicts. A missing icon or missing report is
`NOT_CHECKED`; a known failed command stays failed until its result is retained
and a bounded correction is verified.

## AO

```yaml
project: MoveUp
epic: E026
base_ref: null # Set to a verified committed ref containing the workspace assets before dispatch.
tasks: []
waves: []
```

This is a planning placeholder, not an AO-ready dispatch. After the plan and
pilot scope are reviewed, create task files and replace this block using the
current AO template, with exact write sets and executable checks. Confirm the
current host's executor permissions before any delegation. Before setting
`base_ref`, reconcile local `main` with `origin/main` (local `main` was behind at
planning time), inspect their ancestry, and verify the chosen committed ref
contains `ws/icon-studio/.plan/`, scripts and all three reference boards. An
uncommitted local workspace is not a valid AO worktree base.
