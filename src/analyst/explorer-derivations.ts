/**
 * explorer-derivations.ts — Pure helpers used by `ExplorerTab` to massage
 * snapshot data into chart-ready inputs.
 *
 * - `downsampleSnapshots` — average-by-bucket compression so the height
 *   timeline stays smooth even with a full week of minute-level data.
 * - `deriveDailyKpis` — last-value-per-day rollup of snapshot fields into
 *   the `DailyKpi` shape consumed by `KpiTrend`.
 */
import type { DailyKpi, SessionRow, SnapshotRow } from "@/test/analyst-fixtures";
import type { DeskState } from "@/types";

interface WireSessionRow {
  start: string;
  end: string | null;
  state: string;
  duration_secs: number | null;
}

function toDeskState(s: string): DeskState {
  const lower = s.toLowerCase();
  if (lower === "standing") return "Standing";
  if (lower === "walking") return "Walking";
  if (lower === "away") return "Away";
  return "Sitting";
}

/**
 * Derives a `breakCredit` enum from raw session duration. Mirrors the proportional
 * rule from ADR 008 only approximately — the `SessionRow` shape does not (yet)
 * carry the real `break_credit` field. Tracked in epic IMPROVEMENTS.md.
 */
export function deriveBreakCredit(durationSecs: number): SessionRow["breakCredit"] {
  if (durationSecs >= 120) return "full";
  if (durationSecs >= 60) return "partial";
  return "none";
}

/**
 * Coerces a wire `SessionRow` (sparse, from `get_sessions_range`) into the
 * chart-facing `SessionRow` shape used by Explorer charts.
 */
export function coerceSessionRow(row: WireSessionRow, idx: number): SessionRow {
  const duration = row.duration_secs ?? 0;
  const state = toDeskState(row.state);
  const isSit = state === "Sitting";
  return {
    id: idx,
    startedAt: row.start,
    endedAt: row.end ?? row.start,
    state,
    durationSecs: duration,
    sittingSecs: isSit ? duration : 0,
    standingSecs: isSit ? 0 : duration,
    positionChanges: 0,
    breakCredit: deriveBreakCredit(duration),
    dateLocal: row.start.slice(0, 10),
  };
}

export type { WireSessionRow };

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
