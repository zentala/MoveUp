# T026 — App icon design + tray base icon

**Status:** open
**Priority:** P3
**Branch:** feat/T026-app-icon

---

## Context

The app currently uses placeholder icons. An SVG source has been added:
`src-tauri/icons/icon-source.svg`

**Important note:** The current `icon-source.svg` is a raster PNG embedded in an SVG wrapper (data:image/png;base64). It is NOT a true vector SVG — you cannot change its colors via CSS/attributes. It can be used as visual reference for the icon style.

---

## Current Icon Files

```
src-tauri/icons/
├── 32x32.png           ← placeholder (used as app icon)
├── 128x128.png         ← placeholder
├── icon.ico            ← placeholder (Windows installer)
├── icon-source.svg     ← source image (raster embedded, see note above)
└── tray/
    ├── tray-ok.png     ← green square (sitting, <60%)
    ├── tray-warn.png   ← amber square (sitting, 60-85%)
    ├── tray-alert.png  ← red square (sitting, >85%)
    └── tray-idle.png   ← gray square (standing/away) ← WRONG: should show standing positively
```

All tray icons are currently plain colored 32x32 squares generated in Rust — they are fallbacks when PNG files don't load correctly. The PNG files in `tray/` are loaded first.

---

## Design Spec

### App icon (installer, window title, taskbar)

Show an **adjustable desk** silhouette. The desk's height position conveys the product.

**Concept A — Dual-height desk (recommended):**
Two desk silhouettes at different heights side by side (or morphed), showing low=sitting and high=standing.

**Concept B — Single desk + up arrow:**
A desk surface with an upward arrow, suggesting "raise your desk."

**Search terms to find SVG base:**
- Noun Project: `standing desk`, `adjustable desk`, `sit stand desk`
- SVG Repo: `desk height adjustable`
- Flaticon: `standing desk minimal`

**Design requirements:**
- Works at 16×16 px (tray) and 256×256 (installer)
- Single color (can be white or black base)
- Clean lines, no gradients at small size
- Style: minimal, modern, geometric

### Tray icon

**Two-part design:**
1. **White base icon** (desk silhouette, ~22×22px centered in 32×32 transparent background)
2. **Colored dot** (4–5px diameter, bottom-right corner, ~2px padding from edge)

**Dot colors per app state:**

| State | Dot color | Hex |
|-------|-----------|-----|
| Sitting <60% | Green | `#00C864` |
| Sitting 60-85% | Amber | `#C89600` |
| Sitting >85% / alert | Red | `#C80000` |
| Standing | Gold | `#DAA520` |
| Away / disconnected | Gray | `#808080` |

**White base rationale:** Works on both dark and light Windows taskbars. Colored square (current) looks unprofessional and doesn't adapt to themes.

---

## Implementation

### Step 1 — Create icon assets

Option A — Use Tauri CLI (recommended):
```bash
# From apps/desk directory
# Place a 1024×1024 PNG source in src-tauri/icons/
pnpm tauri icon src-tauri/icons/icon-source-hires.png
# Generates all required sizes automatically
```

Option B — Manual export:
- Export from SVG/PNG at: 16×16, 32×32, 128×128, 256×256, 512×512
- Generate ICO: use https://convertico.com or `magick convert *.png icon.ico`
- Place in `src-tauri/icons/`

### Step 2 — Update tray icon generation in `tray.rs`

Currently `generate_tray_icon()` in `tray.rs` generates solid colored squares. Change to:

```rust
fn generate_tray_icon(state: DeskState, progress_ratio: f32) -> Option<Image<'static>> {
    const SIZE: u32 = 32;
    let pixels = (SIZE * SIZE) as usize;
    let mut rgba = vec![0u8; pixels * 4]; // transparent background

    // Draw white desk base icon (simplified: white rectangle as desk surface)
    // TODO: replace with actual desk silhouette pixels from icon asset
    draw_desk_silhouette(&mut rgba, SIZE);

    // Determine dot color
    let (dr, dg, db) = match state {
        DeskState::Sitting => {
            if progress_ratio < 0.60 { (0, 200, 100) }      // green
            else if progress_ratio < 0.85 { (200, 150, 0) }  // amber
            else { (200, 0, 0) }                              // red
        }
        DeskState::Standing => (218, 165, 32), // gold #DAA520
        _ => (128, 128, 128),                  // gray
    };

    // Draw dot at bottom-right (4×4 px, 2px padding from edge)
    draw_dot(&mut rgba, SIZE, 26, 26, 4, dr, dg, db);

    let rgba_static = Box::leak(rgba.into_boxed_slice());
    Some(Image::new(rgba_static, SIZE, SIZE))
}
```

`draw_dot(buffer, size, x, y, radius, r, g, b)` — fills a circle area with given color.

### Step 3 — Update `tray_icon.rs`

The `icon_for_state_and_progress()` function currently returns strings for PNG filenames. When T016 is implemented (white base + dot), this logic moves to the generator. For now, add `"tray-standing"` as a new icon name for Standing state:

```rust
// tray_icon.rs
_ => "tray-standing", // Standing, Walking (was "tray-idle")
```

And add `src-tauri/icons/tray/tray-standing.png` — white desk + gold dot.

### Step 4 — New PNG tray files

Create these PNG files (32×32, white-on-transparent):
- `tray-ok.png` — white desk + green dot
- `tray-warn.png` — white desk + amber dot
- `tray-alert.png` — white desk + red dot
- `tray-standing.png` — white desk + gold dot ← NEW
- `tray-idle.png` — white desk + gray dot (disconnected/away)

Can be created in Figma, Inkscape, or any image editor.

---

## Files to Modify/Create

| File | Change |
|------|--------|
| `src-tauri/icons/` | Add new PNG assets (all sizes) |
| `src-tauri/icons/tray/*.png` | Replace placeholder squares with white desk + dot |
| `src-tauri/src/tray.rs` | Update `generate_tray_icon()` fallback generator |
| `src-tauri/src/tray_icon.rs` | Add `"tray-standing"` for Standing state |

---

## Dependency: T016

T026 creates the icon assets. T016 is the code change to the tray icon logic (colored dot per state, gold when standing). These should be done together or T026 first, then T016.

---

## Acceptance Criteria

- [ ] App has a desk-themed icon (not placeholder)
- [ ] Tray icon shows white desk silhouette (not solid color block)
- [ ] Tray dot is green/amber/red when sitting (by progress)
- [ ] Tray dot is gold when standing
- [ ] Tray dot is gray when away/disconnected
- [ ] Icons work on both Windows dark and light themes
- [ ] `pnpm tauri:build` succeeds with new icons
