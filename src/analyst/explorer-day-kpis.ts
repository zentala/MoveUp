/**
 * explorer-day-kpis.ts — Pure rollup of one calendar day into the three KPI
 * values the Analyst donuts render.
 *
 * Snapshots carry cumulative per-day counters, so the last snapshot of a day
 * holds that day's totals. Sessions carry the posture changes. A precomputed
 * `DailyKpi` row (fixtures today, a backend rollup later) is used only as a
 * fallback for the two fields raw data can fail to produce: `positionChanges`
 * (the wire `SessionRow` coercion leaves it at 0) and `score` (absent when the
 * day has no snapshots at all).
 */
import type { DailyKpi, SessionRow, SnapshotRow } from "@/test/analyst-fixtures";

export interface DayKpis {
  dateLocal: string;
  /** False when the day has neither snapshots nor sessions. */
  hasData: boolean;
  sittingSecs: number | null;
  standingSecs: number | null;
  /** Sitting + standing. Away/walking time is not "active" here. */
  activeSecs: number | null;
  /** Standing share of known raw sitting + standing time, 0..1. */
  standingPct: number | null;
  positionChanges: number | null;
  /** Signed recorded profile points; null means unavailable. */
  score: number | null;
}

function emptyDay(dateLocal: string): DayKpis {
  return {
    dateLocal,
    hasData: false,
    sittingSecs: null,
    standingSecs: null,
    activeSecs: null,
    standingPct: null,
    positionChanges: null,
    score: null,
  };
}

/** Snapshots whose timestamp falls on `dateLocal`, in timestamp order. */
function snapshotsOfDay(rows: SnapshotRow[], dateLocal: string): SnapshotRow[] {
  return rows
    .filter((r) => {
      return r.localDate === dateLocal;
    })
    .sort((a, b) => a.ts.localeCompare(b.ts));
}

function recordedSessionChanges(sessions: SessionRow[]): number | null {
  if (sessions.length === 0 || sessions.some((session) => session.positionChanges === null)) return null;
  return sessions.reduce((sum, session) => sum + (session.positionChanges ?? 0), 0);
}

/**
 * Rolls the raw Explorer data up into one day's KPI triple.
 *
 * @param snapshots All snapshots in the loaded range.
 * @param sessions All sessions in the loaded range.
 * @param dateLocal The selected day, `YYYY-MM-DD`.
 * @param dailyKpis Optional precomputed rollup; supplies `positionChanges`
 *   and `score` when the raw data yields none.
 */
export function aggregateDayKpis(
  snapshots: SnapshotRow[],
  sessions: SessionRow[],
  dateLocal: string,
  dailyKpis?: DailyKpi[],
): DayKpis {
  const daySnaps = snapshotsOfDay(snapshots, dateLocal);
  const daySessions = sessions.filter((s) => s.dateLocal === dateLocal);
  const rollup = dailyKpis?.find((k) => k.dateLocal === dateLocal);

  if (daySnaps.length === 0 && daySessions.length === 0) {
    if (!rollup) return emptyDay(dateLocal);
    return {
      ...emptyDay(dateLocal),
      hasData: true,
      standingPct: rollup.standingPct / 100,
      positionChanges: rollup.positionChanges,
      score: rollup.score,
    };
  }

  if (daySnaps.length === 0) {
    return {
      ...emptyDay(dateLocal),
      hasData: true,
      standingPct: rollup ? rollup.standingPct / 100 : null,
      positionChanges: recordedSessionChanges(daySessions) ?? rollup?.positionChanges ?? null,
      score: rollup?.score ?? null,
    };
  }

  const last = daySnaps[daySnaps.length - 1];
  const sittingSecs = last?.sittingSecsTotal ?? null;
  const standingSecs = last?.standingSecs ?? null;
  const activeSecs = sittingSecs === null || standingSecs === null
    ? null
    : sittingSecs + standingSecs;
  const standingPct = sittingSecs !== null && standingSecs !== null && sittingSecs + standingSecs > 0
    ? standingSecs / (sittingSecs + standingSecs)
    : null;

  const positionChanges = last?.positionChanges ?? (
    recordedSessionChanges(daySessions) ?? rollup?.positionChanges ?? null
  );

  const score = last ? last.score : (rollup?.score ?? null);

  return {
    dateLocal,
    hasData: true,
    sittingSecs,
    standingSecs,
    activeSecs,
    standingPct,
    positionChanges,
    score,
  };
}
