# Overlay Progress Bar — Developer Guide

Reference for developing and testing the top-of-screen progress bar.

---

## Environment Variables

All env vars are read at app startup. Restart required after changes.

| Variable | Values | Default | Description |
|----------|--------|---------|-------------|
| `OVERLAY_DEV_MODE` | `true`/`false` | `true` in debug, `false` in release | Enables demo animation (ignores sensor data) |
| `OVERLAY_MODE` | `opaque`/`layered` | `opaque` | Render backend (see Render Modes below) |
| `OVERLAY_HEIGHT` | `1`–`20` | `4` | Bar height in pixels |
| `OVERLAY_VARIANT` | `0`/`1`/`2` | `0` | Bar style (see Variants below) |

## Data Modes

### Demo Mode (`dev_mode = true`)
- Auto-enabled in debug builds (`cargo dev` / `pnpm tauri:dev`)
- Shows cycling animation: 0% → 25% → 50% → 75% → 100%, looping every 25 seconds
- Each stage lasts 5 seconds (300 frames @ 60fps)
- Color follows progress: green → yellow → red
- Ignores `update()`, `show()`, `hide()` calls from `tray_controller.rs`
- **Use case:** Visual development without desk sensor

### Live Mode (`dev_mode = false`)
- Active in release builds by default
- Bar driven by real session data from `tray_controller.rs`
- Shows when sitting (`DeskState::Sitting`), hides otherwise
- Progress = `sitting_seconds / session_limit_secs`
- Color: green (0%) → yellow (60%) → red (85%+)
- **Use case:** Production use with desk sensor connected

### Switching between modes
```bash
# Demo mode (default in debug)
pnpm tauri:dev

# Demo mode explicitly
OVERLAY_DEV_MODE=true pnpm tauri:dev

# Live mode in debug build (requires sensor)
OVERLAY_DEV_MODE=false pnpm tauri:dev

# Release build (always live mode unless overridden)
pnpm tauri:build
```

## Render Modes

### OPAQUE (default)
- GDI `BeginPaint`/`FillRect` rendering
- Black background behind the bar
- Simple, reliable, low CPU
- Window styles: `WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW` + `WS_POPUP | WS_VISIBLE`

### LAYERED (experimental)
- `UpdateLayeredWindow` with 32-bit ARGB DIBSection
- Transparent background (bar floats over desktop)
- More complex, pre-multiplied alpha required
- ~78% opacity (200/255)

```bash
# Test LAYERED mode
OVERLAY_MODE=layered pnpm tauri:dev
```

See [MODE-COMPARISON.md](./MODE-COMPARISON.md) for detailed comparison.

## Bar Variants (Styles)

| Variant | `OVERLAY_VARIANT` | Description |
|---------|-------------------|-------------|
| Solid | `0` | Single color fill (default) |
| Gradient | `1` | Dark-to-bright, left-to-right |
| Pulsing | `2` | Brightness oscillates (sin wave) |

```bash
# Test gradient style
OVERLAY_VARIANT=1 pnpm tauri:dev

# Test pulsing style
OVERLAY_VARIANT=2 pnpm tauri:dev

# Combine: tall pulsing layered bar
OVERLAY_HEIGHT=10 OVERLAY_VARIANT=2 OVERLAY_MODE=layered pnpm tauri:dev
```

## Architecture

```
serial.rs (sensor) → session.rs (state machine) → tray_controller.rs → overlay_renderer.rs
                                                                         ↑
                                                              update(progress, color)
                                                              show() / hide()
```

- `overlay_renderer.rs` — creates native WinAPI window in background thread
- `tray_controller.rs` — listens for `desk:state-changed` events, calls overlay methods
- In dev_mode, `tray_controller.rs` calls are ignored; demo animation runs independently

## Key Files

| File | Purpose |
|------|---------|
| `src-tauri/src/overlay_renderer.rs` | WinAPI overlay window, rendering, dev mode |
| `src-tauri/src/tray_controller.rs` | Wires session events → overlay + tray |
| `src-tauri/src/colors.rs` | `color_for_progress()` — progress → RGB mapping |
| `src-tauri/src/serial.rs` | Serial port reader, sensor auto-detect |
| `src-tauri/src/session.rs` | Session state machine (sitting/standing/walking) |

## Testing

```bash
# Run overlay unit tests
cd src-tauri && cargo test overlay_renderer --lib

# Run all Rust tests
cd src-tauri && cargo test --lib

# Run auto-test script (Windows, verifies bar stages change)
bash .claude/overlay/test-infrastructure/auto-test.sh [opaque|layered]
```

## Known Issues
- OPAQUE mode has visible black background (4px, barely noticeable)
- LAYERED mode is experimental — may have rendering edge cases
- Bar not visible without sensor in live mode (by design — no data to show)
