# Overlay Rendering Modes — OPAQUE vs LAYERED

**Source:** `src-tauri/src/overlay_renderer.rs`
**Selection:** env var `OVERLAY_MODE` (default: `opaque`, set `layered` for transparent)

---

## OPAQUE Mode (default, stable)

**Entry point:** `run_event_loop_opaque()` -> `wnd_proc()`

### How it works
1. Registers `WNDCLASSA` with `hbrBackground = BLACK_BRUSH`
2. Creates window with `CreateWindowExA`
3. Sets 16ms timer (~60fps) that calls `InvalidateRect` on each tick
4. `WM_PAINT` handler uses `BeginPaint`/`EndPaint` pipeline:
   - Fills entire client rect with black via `CreateSolidBrush(COLORREF(0))` + `FillRect`
   - Draws colored progress bar from `(0,0)` to `(bar_width, window_height)` via `FillRect`

### Window styles
- **Extended:** `WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`
- **Base:** `WS_POPUP | WS_VISIBLE`

### Rendering pipeline
```
WM_TIMER -> InvalidateRect -> WM_PAINT -> BeginPaint -> FillRect (bg) -> FillRect (bar) -> EndPaint
```

### Storage
- 1 slot in `cbWndExtra` (state pointer)

---

## LAYERED Mode (experimental, transparent)

**Entry point:** `run_event_loop_layered()` -> `wnd_proc_layered()` + `draw_layered_frame()`

### How it works
1. Registers `WNDCLASSA` with `hbrBackground = HBRUSH::default()` (null, no OS background)
2. Creates window with `CreateWindowExA` — **without** `WS_VISIBLE` initially
3. Allocates a 32-bit ARGB `DIBSection` (top-down bitmap, `biHeight` negative) matching screen width x 4px
4. Creates memory DC (`CreateCompatibleDC`), selects bitmap into it
5. Caches screen DC (`GetDC(None)`) for the lifetime of the window
6. Draws initial frame before showing window
7. On each 16ms timer tick, `draw_layered_frame()` executes:
   - Clears entire pixel buffer to 0 (fully transparent)
   - Writes BGRA pixels directly into the DIBSection memory for the bar region
   - Calls `UpdateLayeredWindow` with `ULW_ALPHA` and `BLENDFUNCTION { AlphaFormat: AC_SRC_ALPHA }`

### Pre-multiplied alpha
Colors are pre-multiplied before writing to the pixel buffer:
```rust
let alpha = 200u32;  // semi-transparent (not fully opaque)
let r = (color_rgb.0 as u32) * alpha / 255;
let g = (color_rgb.1 as u32) * alpha / 255;
let b = (color_rgb.2 as u32) * alpha / 255;
let pixel = (alpha << 24) | (r << 16) | (g << 8) | b;
```
The bar renders at alpha 200/255 (~78% opacity). Non-bar pixels are alpha 0 (fully transparent).

### Window styles
- **Extended:** `WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`
- **Base:** `WS_POPUP` (no `WS_VISIBLE` — shown after first frame is drawn)

### Rendering pipeline
```
WM_TIMER -> lock state -> write pixels to DIBSection -> UpdateLayeredWindow (composites to screen)
```
No `InvalidateRect`, no `WM_PAINT`. Layered windows bypass the standard paint pipeline entirely.

### Storage
- 2 slots in `cbWndExtra` (16 bytes): state pointer + `LayeredBufferState` pointer

### Cleanup
Explicit resource release in `WM_DESTROY`: restore old bitmap, `DeleteObject(hbm)`, `DeleteDC(hdc_mem)`, `ReleaseDC(hdc_screen)`.

---

## Technical Comparison

| Aspect | OPAQUE | LAYERED |
|--------|--------|---------|
| **Rendering API** | GDI `BeginPaint`/`FillRect` | `UpdateLayeredWindow` + DIBSection |
| **Transparency** | None (black background) | Per-pixel alpha (alpha=200) |
| **Background** | Solid black (`BLACK_BRUSH`) | Fully transparent (alpha=0) |
| **Redraw trigger** | `InvalidateRect` -> `WM_PAINT` | Direct call in `WM_TIMER` |
| **Pixel format** | GDI brushes (COLORREF, RGB) | Raw 32-bit BGRA, pre-multiplied alpha |
| **Memory overhead** | Minimal (GDI brushes only) | DIBSection buffer (screen_width * 4px * 4 bytes) |
| **CPU per frame** | 2x `FillRect` calls | `memset` full buffer + pixel loop + `UpdateLayeredWindow` |
| **API complexity** | Low (standard GDI) | High (DIBSection, memory DC, alpha blending) |
| **Resource cleanup** | Brushes per-frame | DIBSection + memory DC + screen DC (lifetime of window) |
| **Window extra data** | 1 pointer (8 bytes) | 2 pointers (16 bytes) |
| **Initial visibility** | `WS_VISIBLE` on create | Hidden until first frame drawn |
| **Windows compat** | All versions | Windows 2000+ (`WS_EX_LAYERED`) |
| **WM_PAINT used** | Yes | No |

---

## Production Recommendation

**Use OPAQUE mode for production.** Reasons:

1. **Reliability.** OPAQUE uses the standard Windows paint pipeline (`WM_PAINT`). It has been tested and proven visible. LAYERED mode bypasses this pipeline and has more failure modes (DIBSection allocation, alpha pre-multiplication errors, `UpdateLayeredWindow` failures).

2. **Simplicity.** OPAQUE requires ~120 lines of rendering code. LAYERED requires ~250 lines plus a separate `LayeredBufferState` struct, manual pixel manipulation, and explicit resource lifecycle management.

3. **Performance.** OPAQUE delegates compositing to the OS via GDI. LAYERED requires clearing and rewriting the entire pixel buffer every frame, plus a full `UpdateLayeredWindow` call. For a 4px-tall bar this difference is negligible, but OPAQUE has zero unnecessary work.

4. **Black background is acceptable.** The bar is 4px tall at the top of the screen. A black background on a 4px strip is barely noticeable and does not interfere with the user experience. The trade-off of transparency (LAYERED) does not justify the added complexity.

5. **Debugging.** OPAQUE mode issues are diagnosable with standard GDI debugging. LAYERED mode issues (invisible window, wrong alpha, failed `UpdateLayeredWindow`) are harder to diagnose — as proven by the v1 debugging session.

**When LAYERED makes sense:** If the bar height increases beyond 4px (e.g., a thicker progress indicator or animated effects), the black background becomes visually distracting. At that point, revisit LAYERED mode with thorough testing.
