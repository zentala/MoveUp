# T016 — Tray icon redesign: white base + colored dot

**Status:** open
**Priority:** P2
**Branch:** feat/T016-tray-icon
**Depends on:** T026 (icon assets) — can implement dot logic without final assets using colored squares as placeholder

---

## Goal

Replace the current solid-colored-square tray icon with a **white desk silhouette base + small colored dot** in the corner. The dot communicates app state at a glance without being ugly.

**Critically:** When standing, the dot should be **gold** (not gray). Currently standing = gray = same as "disconnected" = zero feedback. Standing is positive and should be celebrated visually.

---

## Current Behavior (to replace)

In `src-tauri/src/tray_icon.rs`, `icon_for_state_and_progress()`:
```rust
pub fn icon_for_state_and_progress(state: DeskState, ratio: f32) -> &'static str {
    match state {
        DeskState::Sitting => {
            if ratio >= 0.85 { "tray-alert" }      // red square
            else if ratio >= 0.60 { "tray-warn" }  // amber square
            else { "tray-ok" }                     // green square
        }
        _ => "tray-idle", // gray square — Standing, Walking, Away ALL SAME
    }
}
```

In `src-tauri/src/tray.rs`, `generate_tray_icon()` (fallback when PNG not found):
```rust
// Generates solid 32x32 squares — green/amber/red for sitting, gray for everything else
```

---

## New Behavior

### New icon name for Standing
```rust
// tray_icon.rs
pub fn icon_for_state_and_progress(state: DeskState, ratio: f32) -> &'static str {
    match state {
        DeskState::Sitting => {
            if ratio >= 0.85 { "tray-alert" }
            else if ratio >= 0.60 { "tray-warn" }
            else { "tray-ok" }
        }
        DeskState::Standing => "tray-standing",  // ← NEW: gold dot
        _ => "tray-idle",  // gray dot (Walking, Away, disconnected)
    }
}
```

### New PNG files needed in `src-tauri/icons/tray/`

| File | Base | Dot color |
|------|------|-----------|
| `tray-ok.png` | white desk | green `#00C864` |
| `tray-warn.png` | white desk | amber `#C89600` |
| `tray-alert.png` | white desk | red `#C80000` |
| `tray-standing.png` | white desk | gold `#DAA520` ← NEW |
| `tray-idle.png` | white desk | gray `#808080` |

All: 32×32 PNG, transparent background, white desk silhouette (~22×22px centered), 4-5px dot at bottom-right corner (2px from edge).

### Updated fallback generator in `tray.rs`

```rust
fn generate_tray_icon(state: DeskState, progress_ratio: f32) -> Option<Image<'static>> {
    const SIZE: u32 = 32;
    let pixels = (SIZE * SIZE) as usize;
    let mut rgba = vec![0u8; pixels * 4]; // transparent background

    // Draw white desk silhouette (simplified: white horizontal bar representing desk surface)
    // Desk surface: y=10..14, x=4..28 (white)
    // Left leg: x=6..8, y=14..22 (white)
    // Right leg: x=24..26, y=14..22 (white)
    for y in 10u32..14 {
        for x in 4u32..28 {
            let idx = ((y * SIZE + x) as usize) * 4;
            rgba[idx] = 255; rgba[idx+1] = 255; rgba[idx+2] = 255; rgba[idx+3] = 255;
        }
    }
    for y in 14u32..22 {
        for x in [6u32..8, 24..26].iter().flatten() {
            let idx = ((y * SIZE + *x) as usize) * 4;
            rgba[idx] = 255; rgba[idx+1] = 255; rgba[idx+2] = 255; rgba[idx+3] = 255;
        }
    }

    // Dot color
    let (dr, dg, db) = match state {
        DeskState::Sitting => {
            if progress_ratio >= 0.85 { (200, 0, 0) }
            else if progress_ratio >= 0.60 { (200, 150, 0) }
            else { (0, 200, 100) }
        }
        DeskState::Standing => (218, 165, 32), // gold #DAA520
        _ => (128, 128, 128),                  // gray
    };

    // 4×4 dot at bottom-right (x=26..30, y=26..30)
    for y in 26u32..30 {
        for x in 26u32..30 {
            let idx = ((y * SIZE + x) as usize) * 4;
            rgba[idx] = dr; rgba[idx+1] = dg; rgba[idx+2] = db; rgba[idx+3] = 255;
        }
    }

    let rgba_static = Box::leak(rgba.into_boxed_slice());
    Some(Image::new(rgba_static, SIZE, SIZE))
}
```

---

## Key Files

| File | Change |
|------|--------|
| `src-tauri/src/tray_icon.rs` | Add `DeskState::Standing => "tray-standing"` |
| `src-tauri/src/tray.rs` | Update `generate_tray_icon()` — white desk + dot |
| `src-tauri/icons/tray/tray-standing.png` | New asset (white desk + gold dot) |
| `src-tauri/icons/tray/tray-ok.png` | Replace with white desk + green dot |
| `src-tauri/icons/tray/tray-warn.png` | Replace with white desk + amber dot |
| `src-tauri/icons/tray/tray-alert.png` | Replace with white desk + red dot |
| `src-tauri/icons/tray/tray-idle.png` | Replace with white desk + gray dot |

## `update_tray()` in `tray.rs` — how it's called

```rust
// Called from tray_controller.rs on every state change:
pub fn update_tray(app: &AppHandle, label: &str, state: DeskState, progress_ratio: f32)
// → sets tooltip + loads PNG from tray/ folder
// → falls back to generate_tray_icon() if PNG not found
```

---

## Tests (~6)

In `tray_icon.rs` tests (update existing 8 tests):
- `sitting_below_60` → `"tray-ok"`
- `sitting_60_to_85` → `"tray-warn"`
- `sitting_above_85` → `"tray-alert"`
- `standing` → `"tray-standing"` ← NEW (was `"tray-idle"`)
- `walking` → `"tray-idle"` (unchanged)
- `away` → `"tray-idle"` (unchanged)

---

## Acceptance Criteria

- [ ] Standing → tray icon shows gold dot (not gray)
- [ ] Sitting progression → green/amber/red dot
- [ ] Away/disconnected → gray dot
- [ ] White desk silhouette visible (not solid color block)
- [ ] `cargo test` passes (update tray_icon.rs tests for Standing)
