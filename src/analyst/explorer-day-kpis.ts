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

/** Daily target used to turn a raw posture-change count into an arc fill. */
export const POSITION_CHANGE_GOAL = 8;

/** Highest daily score the ergonomic profiles award. */
export const SCORE_MAX = 100;

export interface DayKpis {
  dateLocal: string;
  /** False when the day has neither snapshots nor sessions. */
  hasData: boolean;
  sittingSecs: number;
  standingSecs: number;
  /** Sitting + standing. Away/walking time is not "active" here. */
  activeSecs: number;
  /** Standing share of active time, 0..1. */
  standingPct: number;
  positionChanges: number;
  /** Daily score, 0..SCORE_MAX. */
  score: number;
}

function emptyDay(dateLocal: string): DayKpis {
  return {
    dateLocal,
    hasData: false,
    sittingSecs: 0,
    standingSecs: 0,
    activeSecs: 0,
    standingPct: 0,
    positionChanges: 0,
    score: 0,
  };
}

/** Snapshots whose timestamp falls on `dateLocal`, in timestamp order. */
function snapshotsOfDay(rows: SnapshotRow[], dateLocal: string): SnapshotRow[] {
  return rows
    .filter((r) => r.ts.slice(0, 10) === dateLocal)
    .sort((a, b) => a.ts.localeCompare(b.ts));
}

/** Number of state transitions across an ordered snapshot list. */
function countStateChanges(rows: SnapshotRow[]): number {
  let changes = 0;
  for (let i = 1; i < rows.length; i++) {
    if (rows[i].state !== rows[i - 1].state) changes += 1;
  }
  return changes;
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

  const last = daySnaps[daySnaps.length - 1];
  const sittingSecs = last?.sittingSecs ?? 0;
  const standingSecs = last?.standingSecs ?? 0;
  const activeSecs = sittingSecs + standingSecs;

  const sessionChanges = daySessions.reduce((sum, s) => sum + s.positionChanges, 0);
  const positionChanges =
    sessionChanges > 0
      ? sessionChanges
      : countStateChanges(daySnaps) || (rollup?.positionChanges ?? 0);

  const score = last ? Math.round(last.score) : (rollup?.score ?? 0);

  return {
    dateLocal,
    hasData: true,
    sittingSecs,
    standingSecs,
    activeSecs,
    standingPct: activeSecs > 0 ? standingSecs / activeSecs : 0,
    positionChanges,
    score: Math.min(score, SCORE_MAX),
  };
}
