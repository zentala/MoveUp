//! colors.rs — Shared progress color mapping.
//!
//! Converts a progress ratio (0.0–1.0+) to colors used by the tray icon,
//! overlay bar, and UI components.

/// 0–60 % of session limit → neutral (below warning threshold).
pub const THRESHOLD_YELLOW: f32 = 0.60;
/// 60–85 % → yellow.
pub const THRESHOLD_RED: f32 = 0.85;

/// Neutral color for sitting below the warning threshold.
pub const NEUTRAL: (u8, u8, u8) = (0x2c, 0x29, 0x20);

/// Neutral dark base for completed standing laps (was goldenrod #B8860B).
pub const STANDING_LAP_BASE: (u8, u8, u8) = (0x24, 0x20, 0x18);

/// Returns (r, g, b) for a standing progress ratio.
///
/// Previously returned a gold gradient; now returns neutral to avoid green/gold signals.
pub fn color_for_standing(_progress: f32) -> (u8, u8, u8) {
    NEUTRAL
}

/// Returns (r, g, b, css_hex) for a given progress ratio.
///
/// Thresholds:
/// - < 60%: neutral (#2c2920)
/// - 60–85%: yellow (#ffc107)
/// - ≥ 85%: red (#f44336)
pub fn color_for_progress(progress: f32) -> (u8, u8, u8, &'static str) {
    if progress < THRESHOLD_YELLOW {
        (NEUTRAL.0, NEUTRAL.1, NEUTRAL.2, "#2c2920") // neutral
    } else if progress < THRESHOLD_RED {
        (255, 193, 7, "#ffc107") // yellow
    } else {
        (244, 67, 54, "#f44336") // red
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_neutral_below_60_percent() {
        let (r, g, b, css) = color_for_progress(0.3);
        assert_eq!((r, g, b), (0x2c, 0x29, 0x20));
        assert_eq!(css, "#2c2920");
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
    fn color_at_zero_is_neutral() {
        let (r, g, b, css) = color_for_progress(0.0);
        assert_eq!((r, g, b), NEUTRAL);
        assert_eq!(css, "#2c2920");
    }

    #[test]
    fn color_at_60_percent_is_yellow() {
        let (_, _, _, css) = color_for_progress(0.60);
        assert_eq!(css, "#ffc107");
    }
}
