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
import type { SnapshotRow as SnapshotWireRow } from "@/generated/SnapshotRow";
import type { SessionRow as SessionWireRow } from "@/generated/SessionRow";
import type { DeskState } from "@/types";
import {
  normalizeIntervals,
  timelineGaps,
  type TimelineInterval,
  type TimelineWindow,
} from "@/timeline/intervals";

function toDeskState(s: string | null | undefined): DeskState | null {
  if (!s) return null;
  const lower = s.toLowerCase();
  if (lower === "standing") return "Standing";
  if (lower === "walking") return "Walking";
  if (lower === "away") return "Away";
  if (lower === "sitting") return "Sitting";
  return null;
}

/** Adapts the real generated Rust row without filling unknown wire values. */
export function coerceSnapshotRow(row: SnapshotWireRow): SnapshotRow {
  const quality = row.observation_quality;
  return {
    ts: row.ts,
    state: quality === "known" ? toDeskState(row.state) : null,
    deskHeightCm: row.desk_height_cm,
    sittingSecs: row.sitting_seconds,
    standingSecs: row.standing_seconds,
    breakSecs: row.break_seconds,
    idleSecs: row.idle_secs,
    score: row.daily_score,
    sittingSecsTotal: row.sitting_seconds_total,
    connected: row.connected,
    observationQuality: quality,
    localDate: row.local_date,
    timezoneOffsetSecs: row.timezone_offset_secs,
    positionChanges: row.position_changes,
  };
}

/**
 * Coerces a wire `SessionRow` (sparse, from `get_sessions_range`) into the
 * chart-facing `SessionRow` shape used by Explorer charts.
 */
export function coerceSessionRow(row: SessionWireRow, idx: number): SessionRow {
  const state = toDeskState(row.state);
  const duration = row.duration_secs;
  const knownPosture = row.history_kind === "individual" && state !== null && duration !== null;
  const localStart = /^(\d{4}-\d{2}-\d{2})T.*[+-]\d{2}:\d{2}$/.exec(row.start);
  return {
    id: idx,
    startedAt: row.start,
    endedAt: row.end,
    state,
    durationSecs: duration,
    sittingSecs: knownPosture ? (state === "Sitting" ? duration : 0) : null,
    standingSecs: knownPosture ? (state === "Standing" ? duration : 0) : null,
    positionChanges: null,
    breakCredit: row.break_credit,
    historyKind: row.history_kind ?? null,
    dateLocal: localStart?.[1] ?? null,
  };
}

export type WireSessionRow = SessionWireRow;

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
 * Return the longest proven contiguous Standing run in a source day window.
 * Every part of the window must be covered by individual session endpoints;
 * legacy summaries or any Unknown hole make the maximum unknowable.
 */
export function deriveLongestContiguousStandingSecs(
  sessions: readonly SessionRow[],
  window: TimelineWindow | null | undefined,
): number | null {
  if (!window || window.endMs <= window.startMs) return null;
  const individual: TimelineInterval[] = sessions.flatMap((row) => {
    if (row.historyKind !== "individual" || !row.state || !row.endedAt) return [];
    return [{
      startMs: Date.parse(row.startedAt),
      endMs: Date.parse(row.endedAt),
      state: row.state,
      provenance: "session",
    }];
  });
  const intervals = normalizeIntervals(individual, window);
  if (timelineGaps(intervals, window, window.endMs).length > 0) return null;

  let longestMs = 0;
  let activeStandingMs = 0;
  let previousEnd: number | null = null;
  for (const interval of intervals) {
    if (interval.state === "Standing") {
      activeStandingMs = previousEnd === interval.startMs ? activeStandingMs + interval.endMs - interval.startMs
        : interval.endMs - interval.startMs;
      longestMs = Math.max(longestMs, activeStandingMs);
    } else {
      activeStandingMs = 0;
    }
    previousEnd = interval.endMs;
  }
  return longestMs / 1000;
}

/**
 * Reduces a snapshot list to one `DailyKpi` row per local date by taking the
 * last snapshot of each day as the day's "current" value. Standing percentage
 * is computed from the cumulative sitting/standing counters.
 */
export function deriveDailyKpis(
  rows: SnapshotRow[],
  sessions: readonly SessionRow[] = [],
  dayWindows: ReadonlyMap<string, TimelineWindow> = new Map(),
): DailyKpi[] {
  const byDay = new Map<string, SnapshotRow>();
  for (const r of rows) {
    const day = r.localDate;
    if (!day) continue;
    const cur = byDay.get(day);
    if (!cur || r.ts > cur.ts) byDay.set(day, r);
  }
  const out: DailyKpi[] = [];
  for (const [dateLocal, snap] of Array.from(byDay.entries()).sort(([a], [b]) => a.localeCompare(b))) {
    const sitting = snap.sittingSecsTotal;
    const standing = snap.standingSecs;
    const changes = snap.positionChanges;
    if (sitting == null || changes == null || snap.score == null) continue;
    const total = sitting + standing;
    if (total <= 0) continue;
    const dayWindow = dayWindows.get(dateLocal);
    const overlappingSessions = dayWindow
      ? sessions.filter((row) => {
        const startMs = Date.parse(row.startedAt);
        const endMs = row.endedAt === null ? Number.NaN : Date.parse(row.endedAt);
        return Number.isFinite(startMs) && Number.isFinite(endMs) &&
          startMs < dayWindow.endMs && endMs > dayWindow.startMs;
      })
      : [];
    out.push({
        dateLocal,
        standingPct: Math.round((standing / total) * 100),
        positionChanges: changes,
        longestSessionSecs: deriveLongestContiguousStandingSecs(
          overlappingSessions,
          dayWindow,
        ),
        score: snap.score,
      });
  }
  return out;
}
