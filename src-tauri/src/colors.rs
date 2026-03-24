//! colors.rs — Shared progress color mapping.
//!
//! Converts a progress ratio (0.0–1.0+) to colors used by the tray icon,
//! overlay bar, and UI components.

/// 0–60 % of session limit → green.
pub const THRESHOLD_YELLOW: f32 = 0.60;
/// 60–85 % → yellow.
pub const THRESHOLD_RED: f32 = 0.85;

/// Returns (r, g, b, css_hex) for a given progress ratio.
///
/// Thresholds:
/// - < 60%: green (#4caf50)
/// - 60–85%: yellow (#ffc107)
/// - ≥ 85%: red (#f44336)
pub fn color_for_progress(progress: f32) -> (u8, u8, u8, &'static str) {
    if progress < THRESHOLD_YELLOW {
        (76, 175, 80, "#4caf50") // green
    } else if progress < THRESHOLD_RED {
        (255, 193, 7, "#ffc107") // yellow
    } else {
        (244, 67, 54, "#f44336") // red
    }
}

/// Dark goldenrod base color for completed standing laps: `#B8860B`.
pub const STANDING_LAP_BASE: (u8, u8, u8) = (0xB8, 0x86, 0x0B);

/// Returns (r, g, b) for a standing progress ratio (0.0 → 1.0).
///
/// Interpolates from goldenrod `#DAA520` to bright gold `#FFD720`.
pub fn color_for_standing(progress: f32) -> (u8, u8, u8) {
    let t = progress.clamp(0.0, 1.0);
    let r = (0xDA as f32 + (0xFF - 0xDA) as f32 * t) as u8; // 218→255
    let g = (0xA5 as f32 + (0xD7 - 0xA5) as f32 * t) as u8; // 165→215
    let b = 0x20u8; // constant: 32
    (r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_green_below_60_percent() {
        let (r, g, b, css) = color_for_progress(0.3);
        assert_eq!((r, g, b), (76, 175, 80));
        assert_eq!(css, "#4caf50");
    }

    #[test]
    fn color_yellow_between_60_and_85() {
        let (r, g, b, css) = color_for_progress(0.7);
        assert_eq!((r, g, b), (255, 193, 7));
        assert_eq!(css, "#ffc107");
    }

    #[test]
    fn color_red_above_85_percent() {
        let (r, g, b, css) = color_for_progress(0.9);
        assert_eq!((r, g, b), (244, 67, 54));
        assert_eq!(css, "#f44336");
    }

    #[test]
    fn standing_color_at_zero_is_goldenrod() {
        assert_eq!(color_for_standing(0.0), (218, 165, 32));
    }

    #[test]
    fn standing_color_at_one_is_bright_gold() {
        assert_eq!(color_for_standing(1.0), (255, 215, 32));
    }

    #[test]
    fn standing_color_at_half_is_mid_gold() {
        let (r, g, b) = color_for_standing(0.5);
        assert!(r > 218 && r < 255, "r={r} should be between 218 and 255");
        assert!(g > 165 && g < 215, "g={g} should be between 165 and 215");
        assert_eq!(b, 32);
    }
}
