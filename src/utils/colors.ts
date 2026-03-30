/**
 * colors.ts — Canonical color palette for desk session progress.
 *
 * Source of truth: `src-tauri/src/colors.rs`.
 * Both Rust (overlay bar) and TypeScript (popup ProgressBar, widgets) must
 * use the SAME hex values and thresholds defined here.
 *
 * If you change colors here, update `colors.rs` to match.
 */

/** Progress ratio below which the bar is neutral (no warning). */
export const THRESHOLD_YELLOW = 0.60;

/** Progress ratio at or above which the bar turns red. */
export const THRESHOLD_RED = 0.85;

/** Neutral color for sitting below warning threshold — matches colors.rs NEUTRAL. */
export const NEUTRAL = "#2c2920";

/** Sitting session progress colors — matches colors.rs exactly. */
export const SITTING_YELLOW = "#ffc107";
export const SITTING_RED = "#f44336";

/** Neutral dark base for completed standing laps — matches colors.rs STANDING_LAP_BASE. */
export const STANDING_LAP_BASE = "#242018";

/** Away state color. */
export const AWAY_GRAY = "#808080";

/**
 * Returns the state dot / label color for a given desk state.
 *
 * - Sitting: progress-based (neutral/yellow/red)
 * - Standing: beam amber (handled by CSS --beam variable)
 * - Away/Walking: gray
 */
export function stateColor(state: string, limitRatio = 0): string {
  if (state === "Sitting") return sittingColorForRatio(limitRatio);
  if (state === "Standing") return AWAY_GRAY;
  return AWAY_GRAY;
}

/**
 * Returns the sitting progress bar color for a given ratio (0.0-1.0).
 *
 * Thresholds (same as Rust `color_for_progress`):
 * - < 60%: neutral (#2c2920)
 * - 60-85%: yellow (#ffc107)
 * - >= 85%: red (#f44336)
 */
export function sittingColorForRatio(ratio: number): string {
  if (ratio >= THRESHOLD_RED) return SITTING_RED;
  if (ratio >= THRESHOLD_YELLOW) return SITTING_YELLOW;
  return NEUTRAL;
}
