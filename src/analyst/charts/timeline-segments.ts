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
