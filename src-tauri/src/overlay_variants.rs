//! overlay_variants.rs — Shared variant rendering for GDI (opaque) and pixel (layered) backends.
//!
//! Two entry points:
//! - `render_variant_gdi()` — used by OPAQUE backend (WM_PAINT / BeginPaint / FillRect)
//! - `render_variant_pixels()` — used by LAYERED backend (DIBSection / UpdateLayeredWindow)

/// Render the progress bar using GDI into an existing HDC (OPAQUE backend).
///
/// Called from `wnd_proc` inside `WM_PAINT` after the black background is filled.
/// `hdc` is the paint DC from `BeginPaint`. The caller is responsible for `EndPaint`.
#[cfg(target_os = "windows")]
pub(crate) fn render_variant_gdi(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    bar_width: i32,
    window_height: i32,
    color_rgb: (u8, u8, u8),
    frame_count: u32,
    overlay_variant: u8,
) {
    use windows::Win32::Foundation::*;
    use windows::Win32::Graphics::Gdi::*;

    match overlay_variant {
        0 => {
            // Solid fill (default behavior)
            let color = COLORREF(
                (color_rgb.0 as u32)
                    | ((color_rgb.1 as u32) << 8)
                    | ((color_rgb.2 as u32) << 16),
            );
            let brush = unsafe { CreateSolidBrush(color) };
            if bar_width > 0 && !brush.is_invalid() {
                let bar_rect = RECT {
                    left: 0,
                    top: 0,
                    right: bar_width,
                    bottom: window_height,
                };
                unsafe {
                    let _ = FillRect(hdc, &bar_rect, brush);
                }
            }
            if !brush.is_invalid() {
                unsafe {
                    let _ = DeleteObject(brush.into());
                }
            }
        }
        1 => {
            // Gradient: dark to bright, left to right
            for x in 0..bar_width {
                let factor = x as f32 / bar_width.max(1) as f32;
                let r = (color_rgb.0 as f32 * factor) as u8;
                let g = (color_rgb.1 as f32 * factor) as u8;
                let b = (color_rgb.2 as f32 * factor) as u8;
                let col_color = COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16));
                let brush = unsafe { CreateSolidBrush(col_color) };
                if !brush.is_invalid() {
                    let col_rect = RECT {
                        left: x,
                        top: 0,
                        right: x + 1,
                        bottom: window_height,
                    };
                    unsafe {
                        let _ = FillRect(hdc, &col_rect, brush);
                        let _ = DeleteObject(brush.into());
                    }
                }
            }
        }
        2 => {
            // Pulsing: modulate brightness using frame_count
            let pulse = (frame_count as f32 * 0.1).sin() * 0.3 + 0.7; // 0.4 to 1.0
            let r = (color_rgb.0 as f32 * pulse) as u8;
            let g = (color_rgb.1 as f32 * pulse) as u8;
            let b = (color_rgb.2 as f32 * pulse) as u8;
            let pulsed_color =
                COLORREF((r as u32) | ((g as u32) << 8) | ((b as u32) << 16));
            let brush = unsafe { CreateSolidBrush(pulsed_color) };
            if bar_width > 0 && !brush.is_invalid() {
                let bar_rect = RECT {
                    left: 0,
                    top: 0,
                    right: bar_width,
                    bottom: window_height,
                };
                unsafe {
                    let _ = FillRect(hdc, &bar_rect, brush);
                }
            }
            if !brush.is_invalid() {
                unsafe {
                    let _ = DeleteObject(brush.into());
                }
            }
        }
        _ => {} // Unknown variant, render nothing extra
    }
}

/// Render the progress bar into a 32-bit ARGB pixel buffer (LAYERED backend).
///
/// Called from `draw_layered_frame`. `bits_ptr` is the DIBSection pixel array.
/// All pixels are in pre-multiplied BGRA format required by `UpdateLayeredWindow`.
#[cfg(target_os = "windows")]
pub(crate) unsafe fn render_variant_pixels(
    bits_ptr: *mut u32,
    buf_width: i32,
    buf_height: i32,
    bar_width: i32,
    color_rgb: (u8, u8, u8),
    frame_count: u32,
    overlay_variant: u8,
) {
    let alpha = 200u32;

    match overlay_variant {
        0 => {
            // Solid fill (default)
            let r = ((color_rgb.0 as u32) * alpha / 255) as u32;
            let g = ((color_rgb.1 as u32) * alpha / 255) as u32;
            let b = ((color_rgb.2 as u32) * alpha / 255) as u32;
            let pixel = (alpha << 24) | (r << 16) | (g << 8) | b;
            for y in 0..buf_height {
                for x in 0..bar_width {
                    let idx = (y * buf_width + x) as usize;
                    *bits_ptr.add(idx) = pixel;
                }
            }
        }
        1 => {
            // Gradient: dark to bright, left to right
            for y in 0..buf_height {
                for x in 0..bar_width {
                    let factor = x as f32 / bar_width.max(1) as f32;
                    let r = ((color_rgb.0 as f32 * factor) as u32 * alpha / 255) as u32;
                    let g = ((color_rgb.1 as f32 * factor) as u32 * alpha / 255) as u32;
                    let b = ((color_rgb.2 as f32 * factor) as u32 * alpha / 255) as u32;
                    let pixel = (alpha << 24) | (r << 16) | (g << 8) | b;
                    let idx = (y * buf_width + x) as usize;
                    *bits_ptr.add(idx) = pixel;
                }
            }
        }
        2 => {
            // Pulsing: modulate brightness by frame_count
            let pulse = (frame_count as f32 * 0.1).sin() * 0.3 + 0.7; // 0.4 to 1.0
            let r = ((color_rgb.0 as f32 * pulse) as u32 * alpha / 255) as u32;
            let g = ((color_rgb.1 as f32 * pulse) as u32 * alpha / 255) as u32;
            let b = ((color_rgb.2 as f32 * pulse) as u32 * alpha / 255) as u32;
            let pixel = (alpha << 24) | (r << 16) | (g << 8) | b;
            for y in 0..buf_height {
                for x in 0..bar_width {
                    let idx = (y * buf_width + x) as usize;
                    *bits_ptr.add(idx) = pixel;
                }
            }
        }
        _ => {} // Unknown variant, leave transparent
    }
}
