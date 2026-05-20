/**
 * timeline-segments.ts — Pure helpers for rendering the TimelineDetail strip.
 *
 * Kept separate from `timeline-utils.ts` so it can depend on `chartColors`
 * (UI palette) without polluting the pure math module that's reused by
 * the DateNavigator. Stays out of TimelineDetail.tsx to keep that file
 * under the 250-line cap.
 */
import type { SnapshotRow } from "@/test/analyst-fixtures";
import { chartColors } from "./chart-utils";
import { inferSnapshotInterval, msToPx, type TimelineScale } from "./timeline-utils";

const DAY_MS = 86_400_000;

const STATE_FILL: Record<string, string> = {
  Sitting: chartColors.sitting,
  Standing: chartColors.standing,
  Walking: chartColors.walking,
  Away: chartColors.away,
};

export interface RenderedSegment {
  x: number;
  w: number;
  fill: string;
}

/**
 * Convert snapshots to renderable segments. Each snapshot represents observed
 * state from its timestamp until the next snapshot's timestamp (or +1 median
 * interval after if it's the last row). This produces a continuous strip
 * regardless of sampling density (1-minute production data or 20-min
 * fixtures).
 */
export function buildSegments(
  snapshots: SnapshotRow[],
  scale: TimelineScale,
): RenderedSegment[] {
  if (snapshots.length === 0) return [];
  const intervalMs = inferSnapshotInterval(snapshots) * 1000;
  const out: RenderedSegment[] = [];
  for (let i = 0; i < snapshots.length; i++) {
    const row = snapshots[i];
    const t = new Date(row.ts).getTime();
    const tNext =
      i + 1 < snapshots.length
        ? new Date(snapshots[i + 1].ts).getTime()
        : t + intervalMs;
    const startMs = Math.max(t, scale.startMs);
    const endMs = Math.min(tNext, scale.endMs);
    if (endMs <= startMs) continue;
    const x = msToPx(startMs, scale);
    const w = Math.max(1, msToPx(endMs, scale) - x);
    out.push({ x, w, fill: STATE_FILL[row.state] ?? chartColors.away });
  }
  return out;
}

/** Generate midnight epoch-ms timestamps that fall inside the scale. */
export function dayBoundaries(scale: TimelineScale): number[] {
  const out: number[] = [];
  const start = new Date(scale.startMs);
  start.setHours(0, 0, 0, 0);
  for (let t = start.getTime(); t <= scale.endMs; t += DAY_MS) {
    if (t >= scale.startMs) out.push(t);
  }
  return out;
}

/**
 * One hour tick (00:00 boundaries are skipped — they belong to day-dividers).
 */
export interface HourTick {
  ms: number;
  hour: number; // 0..23
  /** True if this tick deserves a text label (vs. just a hash mark). */
  isLabel: boolean;
}

/**
 * Adaptive hour ticks across the visible scale.
 *
 * Density depends on `pxPerMinute` so that ticks stay readable from zoomed-in
 * 7-day views (lots of px per hour) down to 30-day views (few px per hour):
 *
 * | px/hour    | tick step | label step |
 * |------------|-----------|------------|
 * | >= 40      | 1h        | 2h         |
 * | 20..40     | 2h        | 6h         |
 * | 10..20     | 6h        | 12h        |
 * | < 10       | 12h       | 12h        |
 *
 * 00:00 is always omitted (already drawn as a day-divider).
 */
export function enumerateHourTicks(scale: TimelineScale): HourTick[] {
  const pxPerHour = scale.pxPerMinute * 60;
  let tickStep = 12;
  let labelStep = 12;
  if (pxPerHour >= 40) {
    tickStep = 1;
    labelStep = 2;
  } else if (pxPerHour >= 20) {
    tickStep = 2;
    labelStep = 6;
  } else if (pxPerHour >= 10) {
    tickStep = 6;
    labelStep = 12;
  }
  const out: HourTick[] = [];
  for (const dayStart of dayBoundaries(scale)) {
    for (let h = tickStep; h < 24; h += tickStep) {
      const ms = dayStart + h * 3_600_000;
      if (ms < scale.startMs || ms > scale.endMs) continue;
      out.push({ ms, hour: h, isLabel: h % labelStep === 0 });
    }
  }
  return out;
}
