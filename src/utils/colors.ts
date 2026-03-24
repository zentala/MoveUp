/**
 * colors.ts — Canonical color palette for desk session progress.
 *
 * Source of truth: `src-tauri/src/colors.rs`.
 * Both Rust (overlay bar) and TypeScript (popup ProgressBar, widgets) must
 * use the SAME hex values and thresholds defined here.
 *
 * If you change colors here, update `colors.rs` to match.
 */

/** Progress ratio below which the bar is green. */
export const THRESHOLD_YELLOW = 0.60;

/** Progress ratio at or above which the bar turns red. */
export const THRESHOLD_RED = 0.85;

/** Sitting session progress colors — matches colors.rs exactly. */
export const SITTING_GREEN = "#4caf50";
export const SITTING_YELLOW = "#ffc107";
export const SITTING_RED = "#f44336";

/** Standing gradient endpoints — matches colors.rs color_for_standing(). */
export const STANDING_START = "#DAA520"; // goldenrod at 0%
export const STANDING_END = "#FFD720"; // bright gold at 100%

/** Away state color. */
export const AWAY_GRAY = "#808080";

/**
 * Returns the sitting progress bar color for a given ratio (0.0-1.0).
 *
 * Thresholds (same as Rust `color_for_progress`):
 * - < 60%: green (#4caf50)
 * - 60-85%: yellow (#ffc107)
 * - >= 85%: red (#f44336)
 */
export function sittingColorForRatio(ratio: number): string {
  if (ratio >= THRESHOLD_RED) return SITTING_RED;
  if (ratio >= THRESHOLD_YELLOW) return SITTING_YELLOW;
  return SITTING_GREEN;
}
