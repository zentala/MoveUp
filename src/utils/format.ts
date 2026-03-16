/**
 * format.ts — utility functions for formatting time and display values.
 */

/**
 * Formats a duration in seconds to `hh:mm:ss` string.
 * @param totalSeconds - Non-negative integer number of seconds.
 * @returns Formatted string like "1:04:32" or "0:32:07".
 */
export function formatDuration(totalSeconds: number): string {
  const s = Math.max(0, Math.floor(totalSeconds));
  const hours = Math.floor(s / 3600);
  const minutes = Math.floor((s % 3600) / 60);
  const seconds = s % 60;

  const mm = String(minutes).padStart(2, "0");
  const ss = String(seconds).padStart(2, "0");

  if (hours > 0) {
    return `${hours}:${mm}:${ss}`;
  }
  return `${mm}:${ss}`;
}

/**
 * Formats a duration in seconds as a human-readable short form: "2h14m" or "45m" or "32s".
 * @param totalSeconds - Non-negative integer number of seconds.
 */
export function formatDurationShort(totalSeconds: number): string {
  const s = Math.max(0, Math.floor(totalSeconds));
  const hours = Math.floor(s / 3600);
  const minutes = Math.floor((s % 3600) / 60);
  const seconds = s % 60;

  if (hours > 0) {
    return `${hours}h${String(minutes).padStart(2, "0")}m`;
  }
  if (minutes > 0) {
    return `${minutes}m`;
  }
  return `${seconds}s`;
}
