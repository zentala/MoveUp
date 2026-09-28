# MoveUp icon studio

This is a reproducible SVG workspace within MoveUp. Its local instructions are
in `AGENTS.md` and its plan is in `.plan/`. None of its output is wired into the application yet. The live Windows tray
icon is generated in `MoveUp/src-tauri/src/tray_icon.rs`; the app installer icon
is a separate asset.

Run `pnpm icons:generate` from the MoveUp root. The command generates every SVG
under `ws/icon-studio/icons/` and refreshes `preview-manifest.js`. Open
`ws/icon-studio/preview.html` to compare the proposals, or inspect `preview.png`
for a static overview. `manifest.mjs` owns the icon names shared by the generator
and preview; generated SVGs removed from the manifest are removed on the next run.
Run `node --test ws/icon-studio/generator-io.test.mjs` to check that contract.

## Families

- `healthy-balance/`: seven monochrome rounded tray states from the old Healthy
  Balance design brief. Source geometry is in `generate.mjs`.
- `moveup/`: seven tray states using MoveUp's instrument-panel desk silhouette
  and compact status markers. Source geometry is also in `generate.mjs`.
- `reference-line/`: fourteen app icon studies drawn by hand after the three
  June 8, 2026 reference boards. Their editable paths are in
  `reference-line.mjs`. They retain the references' principal **shapes**:
  thin outlined desk, two legs and feet, arrows, human figures, pulse marks,
  and sparse blue/green/amber status accents.

The `reference-line` drawings are approximations, not recovered source artwork.
The reference PNGs contain no embedded prompt, vector paths, or editable layers.
The drawings should be compared against the boards icon by icon before adoption.
At small tray sizes, simplify a selected app icon rather than shrinking detailed
figures blindly.

## Reference images and geometry

`references/` contains unchanged copies of the three supplied boards:

- `desk-and-activity.png` — desk variants 01–12 and activity/status icons 13–30.
- `extended-system.png` — status, suggestions, body, rhythm, progress, brand,
  and utility symbols.
- `brand-book.png` — logo, iconography overview, palette, and UI examples.

The files came from `\\nas\Projects\desk\ChatGPT Image 8 cze 2026, *.png`.
Their original ChatGPT conversation may hold the textual instructions; the PNG
metadata does not. `ASK_ORIGINAL_CHAT.md` gives a request for recovering the
actual prompt when available and reconstructing one SVG per chosen icon.

## Selection and integration

1. Choose names from `preview.html` and note what is wrong in each shape.
2. Edit `reference-line.mjs` (or add another family) and rerun the generator.
3. Review SVGs at their intended app size and at 16 px if used in the tray.
4. Map accepted names to actual MoveUp runtime states and rendering paths.

No family is approved by its presence here. The old Healthy Balance product
specification is archived at `MoveUp/.plan/archive/healthy-balance-spec/` and is
not an implementation contract for the current Tauri app.
