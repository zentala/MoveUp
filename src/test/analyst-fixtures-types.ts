/**
 * analyst-fixtures-types.ts — Types for Analyst mockup fixtures.
 */
import type { DeskState, BreakCredit } from "@/types";

/** One data source row in the catalog. */
export interface CatalogSource {
  id: string;
  name: string;
  kind: "sqlite" | "file" | "json" | "store" | "serial" | "signal" | "events";
  location: string;
  fields: { name: string; type: string }[];
  retention: string;
  rowCount: number;
  bytes: number;
}

/** One snapshot row (downsampled minute-level data). */
export interface SnapshotRow {
  ts: string;
  state: DeskState;
  deskHeightCm: number;
  sittingSecs: number;
  standingSecs: number;
  breakSecs: number;
  idleSecs: number;
  score: number;
}

/** One event row from events.log. */
export interface EventRow {
  ts: string;
  type: "STATE" | "CREDIT" | "NOTIF" | "DEVICE" | "ALERT" | "RESET" | "START" | "AUTOSTART";
  message: string;
}

/** One session row from SQLite. */
export interface SessionRow {
  id: number;
  startedAt: string;
  endedAt: string;
  state: DeskState;
  durationSecs: number;
  sittingSecs: number;
  standingSecs: number;
  positionChanges: number;
  breakCredit: BreakCredit;
  dateLocal: string;
}

/** Daily KPI rollup. */
export interface DailyKpi {
  dateLocal: string;
  standingPct: number;
  positionChanges: number;
  longestSessionSecs: number;
  score: number;
}

/** Time constants. */
export const DAY_MS = 86_400_000;
export const HOUR_MS = 3_600_000;

/** Deterministic pseudo-random for stable mock data. */
export function seeded(seed: number): () => number {
  let s = seed;
  return () => {
    s = (s * 1664525 + 1013904223) >>> 0;
    return s / 0xffffffff;
  };
}

/** ISO date string YYYY-MM-DD for `daysAgo` days back from `today`. */
export function dateLocal(today: Date, daysAgo: number): string {
  const d = new Date(today.getTime() - daysAgo * DAY_MS);
  return d.toISOString().slice(0, 10);
}
