/**
 * timeline.ts — shared timeline marker computation.
 *
 * Generates hour markers for proportional timeline bars.
 * Handles midnight crossing correctly.
 */

/** Percentage thresholds to avoid label overlap with start/end markers. */
const MARKER_MIN_PCT = 3;
const MARKER_MAX_PCT = 94;

/** Position percentage for the "now" label at right edge. */
export const NOW_LABEL_PCT = 97;

export interface HourMarker {
  atMs: number;
  leftPct: number;
  label: string;
}

/** Formats hour:minute as "H:MM". */
function timeLabel(date: Date): string {
  return `${date.getHours()}:${String(date.getMinutes()).padStart(2, "0")}`;
}

/**
 * Computes hour markers for the exact timeline window rendered by the caller.
 *
 * Returns: start time at 0%, full hour boundaries in between, current time at ~97%.
 * Handles midnight crossing (sessions starting before midnight, now after).
 */
export function computeHourMarkers(window: { startMs: number; endMs: number }): HourMarker[] {
  const firstStart = new Date(window.startMs);
  const end = new Date(window.endMs);
  const spanMs = window.endMs - window.startMs;
  if (spanMs <= 0) return [];

  const markers: HourMarker[] = [];

  // Start time label at left edge
  markers.push({ atMs: firstStart.getTime(), leftPct: 0, label: timeLabel(firstStart) });

  // Scan real minute instants for local hour boundaries. This preserves both
  // sides of repeated hours and avoids assuming timezone shifts are one hour.
  let cursorMs = Math.floor(firstStart.getTime() / 60_000) * 60_000 + 60_000;
  while (cursorMs < window.endMs) {
    const cursor = new Date(cursorMs);
    if (cursor.getMinutes() === 0) {
      const offsetMs = cursorMs - firstStart.getTime();
      const leftPct = (offsetMs / spanMs) * 100;
      if (leftPct > MARKER_MIN_PCT && leftPct < MARKER_MAX_PCT) {
        markers.push({ atMs: cursorMs, leftPct, label: `${cursor.getHours()}:${String(cursor.getMinutes()).padStart(2, "0")}` });
      }
    }
    cursorMs += 60_000;
  }

  // Current time label at right edge
  markers.push({ atMs: window.endMs, leftPct: NOW_LABEL_PCT, label: timeLabel(end) });

  const labelCounts = new Map<string, number>();
  for (const marker of markers) labelCounts.set(marker.label, (labelCounts.get(marker.label) ?? 0) + 1);
  return markers.map((marker) => {
    if ((labelCounts.get(marker.label) ?? 0) < 2) return marker;
    const offsetMinutes = -new Date(marker.atMs).getTimezoneOffset();
    const sign = offsetMinutes >= 0 ? "+" : "-";
    const absoluteOffset = Math.abs(offsetMinutes);
    const offset = `UTC${sign}${String(Math.floor(absoluteOffset / 60)).padStart(2, "0")}:${String(absoluteOffset % 60).padStart(2, "0")}`;
    return { ...marker, label: `${marker.label} ${offset}` };
  });
}
