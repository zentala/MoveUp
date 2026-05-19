/**
 * timeline-utils.ts — Pure math for the Daily Timeline feature.
 *
 * Two related helpers:
 *   - DateNavigator: per-day state hours aggregation (sum minutes per state).
 *   - TimelineDetail: pixel-per-minute scale for one continuous strip
 *     spanning [range.from - sidePadHours, range.to + sidePadHours].
 *
 * No DOM access, no React — easy to test under Vitest.
 */
import type { SnapshotRow } from "@/test/analyst-fixtures";
import type { DeskState } from "@/types";

/** Seconds-per-state totals for a single day. */
export interface DayStateTotals {
  dateLocal: string;
  /** Total seconds in each state for that day. */
  sit: number;
  stand: number;
  walk: number;
  away: number;
  /** Convenience: total Sit + Stand + Walk + Away in seconds. */
  total: number;
}

/** A YYYY-MM-DD ↦ DayStateTotals map keyed by local date. */
export type DayTotalsMap = Map<string, DayStateTotals>;

const MINUTE_SECS = 60;

function emptyTotals(date: string): DayStateTotals {
  return { dateLocal: date, sit: 0, stand: 0, walk: 0, away: 0, total: 0 };
}

function addToTotals(t: DayStateTotals, state: DeskState, secs: number): void {
  switch (state) {
    case "Sitting":
      t.sit += secs;
      break;
    case "Standing":
      t.stand += secs;
      break;
    case "Walking":
      t.walk += secs;
      break;
    case "Away":
      t.away += secs;
      break;
  }
  t.total += secs;
}

/** Median seconds between consecutive snapshots, capped to [60s, 1h]. */
export function inferSnapshotInterval(rows: SnapshotRow[]): number {
  if (rows.length < 2) return MINUTE_SECS;
  const gaps: number[] = [];
  for (let i = 1; i < rows.length; i++) {
    const dt =
      (new Date(rows[i].ts).getTime() - new Date(rows[i - 1].ts).getTime()) / 1000;
    if (dt > 0 && dt < 3600) gaps.push(dt);
  }
  if (gaps.length === 0) return MINUTE_SECS;
  gaps.sort((a, b) => a - b);
  const median = gaps[Math.floor(gaps.length / 2)];
  return Math.max(MINUTE_SECS, Math.min(3600, median));
}

/**
 * Aggregate snapshots into per-day state totals. Each snapshot is treated as
 * occupying `secsPerSnapshot` seconds; if omitted, the interval is inferred
 * from the median gap between consecutive rows.
 */
export function aggregateDayTotals(
  rows: SnapshotRow[],
  secsPerSnapshot?: number,
): DayTotalsMap {
  const interval = secsPerSnapshot ?? inferSnapshotInterval(rows);
  const map: DayTotalsMap = new Map();
  for (const r of rows) {
    const date = r.ts.slice(0, 10);
    let bucket = map.get(date);
    if (!bucket) {
      bucket = emptyTotals(date);
      map.set(date, bucket);
    }
    addToTotals(bucket, r.state, interval);
  }
  return map;
}

/** Format a Date as `YYYY-MM-DD` in the LOCAL timezone (not UTC). */
export function localIsoDate(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

/** List every YYYY-MM-DD in [from, to] inclusive, in ascending order. */
export function enumerateDays(from: string, to: string): string[] {
  const start = new Date(`${from}T00:00:00`);
  const end = new Date(`${to}T00:00:00`);
  if (Number.isNaN(start.getTime()) || Number.isNaN(end.getTime())) return [];
  const out: string[] = [];
  for (let d = start; d.getTime() <= end.getTime(); d.setDate(d.getDate() + 1)) {
    out.push(localIsoDate(d));
  }
  return out;
}

/** True for Saturdays + Sundays. Local-timezone interpretation. */
export function isWeekend(dateLocal: string): boolean {
  const d = new Date(`${dateLocal}T00:00:00`);
  const dow = d.getDay();
  return dow === 0 || dow === 6;
}

/** Short weekday name (`Mon` / `Tue` / …) for the local date. */
export function shortDayName(dateLocal: string): string {
  const d = new Date(`${dateLocal}T00:00:00`);
  return d.toLocaleDateString("en-US", { weekday: "short" });
}

/** Two-digit day-of-month (`04`, `17`). */
export function dayOfMonthLabel(dateLocal: string): string {
  return dateLocal.slice(8, 10);
}

// ---------- Timeline detail scale ----------

/** Pixel scale describing how a continuous range maps to a strip width. */
export interface TimelineScale {
  /** Inclusive start of the continuous strip in epoch ms. */
  startMs: number;
  /** Exclusive end of the continuous strip in epoch ms. */
  endMs: number;
  /** Pixels per minute. */
  pxPerMinute: number;
  /** Total pixel width. */
  widthPx: number;
}

const MS_PER_MIN = 60_000;
const MS_PER_HOUR = 3_600_000;

/**
 * Build a continuous pixel scale that pads the selected range with
 * `sidePadHours` of context on each side, then maps to `widthPx`.
 */
export function buildTimelineScale(
  rangeFrom: string,
  rangeTo: string,
  sidePadHours: number,
  widthPx: number,
): TimelineScale {
  const start = new Date(`${rangeFrom}T00:00:00`);
  const endDay = new Date(`${rangeTo}T00:00:00`);
  endDay.setDate(endDay.getDate() + 1);
  const startMs = start.getTime() - sidePadHours * MS_PER_HOUR;
  const endMs = endDay.getTime() + sidePadHours * MS_PER_HOUR;
  const totalMinutes = (endMs - startMs) / MS_PER_MIN;
  const pxPerMinute = totalMinutes > 0 ? widthPx / totalMinutes : 0;
  return { startMs, endMs, pxPerMinute, widthPx };
}

/** Convert epoch ms → pixel offset within the strip. */
export function msToPx(ms: number, scale: TimelineScale): number {
  return ((ms - scale.startMs) / MS_PER_MIN) * scale.pxPerMinute;
}

/**
 * Pixel offset of the centre of `dateLocal`'s 12:00 (noon). Useful when
 * smooth-scrolling the strip so the selected day is centred in the viewport.
 */
export function dayCenterPx(dateLocal: string, scale: TimelineScale): number {
  const noon = new Date(`${dateLocal}T12:00:00`).getTime();
  return msToPx(noon, scale);
}

// ---------- Smooth scroll easing ----------

/** Cubic ease-in-out. t in [0,1] → eased value in [0,1]. */
export function easeInOut(t: number): number {
  if (t <= 0) return 0;
  if (t >= 1) return 1;
  return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
}

/**
 * Duration (ms) for a smooth-scroll animation, scaled by jump magnitude.
 * Snap close jumps fast; big jumps stay readable. Capped to feel snappy.
 */
export function scrollDurationMs(deltaPx: number): number {
  const abs = Math.abs(deltaPx);
  // 1 px → 0.6ms, capped between 240 and 900.
  return Math.min(900, Math.max(240, 240 + abs * 0.6));
}
