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
  leftPct: number;
  label: string;
}

/** Formats hour:minute as "H:MM". */
function timeLabel(date: Date): string {
  return `${date.getHours()}:${String(date.getMinutes()).padStart(2, "0")}`;
}

/**
 * Computes hour markers for a timeline spanning firstStart to now.
 *
 * Returns: start time at 0%, full hour boundaries in between, current time at ~97%.
 * Handles midnight crossing (sessions starting before midnight, now after).
 */
export function computeHourMarkers(firstStartIso: string): HourMarker[] {
  const firstStart = new Date(firstStartIso);
  const now = new Date();
  const spanMs = now.getTime() - firstStart.getTime();
  if (spanMs <= 0) return [];

  const markers: HourMarker[] = [];

  // Start time label at left edge
  markers.push({ leftPct: 0, label: timeLabel(firstStart) });

  // Full hour boundaries — walk hour by hour to handle midnight crossing
  const cursor = new Date(firstStart);
  cursor.setMinutes(0, 0, 0);
  cursor.setHours(cursor.getHours() + 1);

  while (cursor.getTime() < now.getTime()) {
    const offsetMs = cursor.getTime() - firstStart.getTime();
    const leftPct = (offsetMs / spanMs) * 100;
    if (leftPct > MARKER_MIN_PCT && leftPct < MARKER_MAX_PCT) {
      markers.push({ leftPct, label: `${cursor.getHours()}:00` });
    }
    cursor.setHours(cursor.getHours() + 1);
  }

  // Current time label at right edge
  markers.push({ leftPct: NOW_LABEL_PCT, label: timeLabel(now) });

  return markers;
}
