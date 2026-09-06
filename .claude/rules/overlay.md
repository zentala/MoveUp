# Overlay Progress Bar

Native WinAPI window (4px × full screen width) at top of screen showing sitting session progress. NOT a Tauri WebviewWindow — rendered via GDI/UpdateLayeredWindow in a background thread. Code: `src-tauri/src/overlay_renderer.rs`.

**Full developer docs:** `.arch/overlay/DEVELOPER-GUIDE.md`

## Three independent config axes (env vars, read at startup)

| Axis | Env var | Values | Default |
|------|---------|--------|---------|
| Data source | `OVERLAY_DATA` | `demo`, `live`, `mock` | `live` (ALL builds) |
| Render mode | `OVERLAY_MODE` | `opaque`, `layered` | `opaque` |
| Visual style | `OVERLAY_VARIANT` | `0` solid, `1` gradient, `2` pulsing | `0` |
| Bar height | `OVERLAY_HEIGHT` | `1`–`20` px | `4` |

Any combination is valid. All are independent.

## Data sources (`DataSource` enum in `overlay_renderer.rs`)

- **Live** — real sensor data. `tray_controller.rs` calls `overlay.update(progress, color)`. **Default in all builds.**
- **Demo** — cycling animation 0%→25%→50%→75%→100% every 25s. Bar always visible. Requires `OVERLAY_DATA=demo`.
- **Mock** — simulated 40-min sit + 10-min stand compressed to ~3 min.

## pnpm scripts

```bash
pnpm tauri:dev                    # live mode (default, real sensor)
pnpm tauri:dev:demo               # cycling demo animation
pnpm tauri:dev:mock               # simulated sit/stand
powershell -ExecutionPolicy Bypass -File scripts/tauri-dev.ps1 -Force  # live + auto-kill previous instance
```

All variants go through `scripts/tauri-dev.ps1` which handles process guard + env vars.

## Pre-dev process guard

`scripts/tauri-dev.ps1` detects if `desk.exe` is running. On Windows, `cargo` cannot replace a running `.exe`.
- **[k] Kill** old process (default, auto after 10s) | **[s] Skip** — abort build
- `-Force` switch = auto-kill without prompt

## Data flow (Live mode)

```
serial.rs ──desk:distance──→ tray_controller.rs ──→ overlay.update(progress, color)
serial.rs ──desk:state-changed──→ tray_controller.rs ──→ overlay.show() / hide()
```

## Render modes

- **OPAQUE** (default): GDI `BeginPaint`/`FillRect`, black background, reliable
- **LAYERED** (experimental): `UpdateLayeredWindow`, transparent background, complex

## Key rules

- Don't remove working render code without proven replacement
- Test OPAQUE mode — that's what users see
- Bar shows minimum 1px when visible (even at 0%)
- Arrow cursor on overlay window (not loading cursor)
- 101 Rust unit tests cover overlay logic

## Docs & source files

| File | Purpose |
|------|---------|
| `.arch/overlay/DEVELOPER-GUIDE.md` | **START HERE** — full reference |
| `.arch/overlay/KNOWLEDGE-BASE.md` | Architecture, root causes, pitfalls |
| `overlay_renderer.rs` | WinAPI window, DataSource, rendering |
| `tray_controller.rs` | Sensor events → overlay + tray updates |
| `colors.rs` | `color_for_progress()` — green→yellow→red |
