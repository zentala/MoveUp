/**
 * explorer-derivations.ts — Pure helpers used by `ExplorerTab` to massage
 * snapshot data into chart-ready inputs.
 *
 * - `downsampleSnapshots` — average-by-bucket compression so the height
 *   timeline stays smooth even with a full week of minute-level data.
 * - `deriveDailyKpis` — last-value-per-day rollup of snapshot fields into
 *   the `DailyKpi` shape consumed by `KpiTrend`.
 */
import type { DailyKpi, SnapshotRow } from "@/test/analyst-fixtures";

/**
 * Downsamples a snapshot list into `target` evenly-sized buckets. Within each
 * bucket, the timestamp of the last point is kept and `deskHeightCm` is
 * averaged. Returns the original list when below the threshold.
 */
export function downsampleSnapshots(
  rows: SnapshotRow[],
  target: number,
): SnapshotRow[] {
  if (rows.length <= target || target <= 0) return rows;
  const bucketSize = Math.ceil(rows.length / target);
  const out: SnapshotRow[] = [];
  for (let i = 0; i < rows.length; i += bucketSize) {
    const slice = rows.slice(i, i + bucketSize);
    const avg =
      slice.reduce((sum, r) => sum + r.deskHeightCm, 0) / slice.length;
    const last = slice[slice.length - 1];
    out.push({ ...last, deskHeightCm: avg });
  }
  return out;
}

/**
 * Reduces a snapshot list to one `DailyKpi` row per local date by taking the
 * last snapshot of each day as the day's "current" value. Standing percentage
 * is computed from the cumulative sitting/standing counters.
 */
export function deriveDailyKpis(rows: SnapshotRow[]): DailyKpi[] {
  const byDay = new Map<string, SnapshotRow>();
  for (const r of rows) {
    const day = r.ts.slice(0, 10);
    const cur = byDay.get(day);
    if (!cur || r.ts > cur.ts) byDay.set(day, r);
  }
  return Array.from(byDay.entries())
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([dateLocal, snap]) => {
      const total = snap.sittingSecs + snap.standingSecs;
      const standingPct = total > 0
        ? Math.round((snap.standingSecs / total) * 100)
        : 0;
      return {
        dateLocal,
        standingPct,
        positionChanges: 0,
        longestSessionSecs: snap.standingSecs,
        score: Math.round(snap.score),
      };
    });
}
