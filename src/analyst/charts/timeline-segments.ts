/** Pure geometry for the Analyst's interval spans and observation-point ticks. */
import type { SnapshotRow } from "@/test/analyst-fixtures";
import { timelineColors } from "@/timeline/colors";
import {
  intervalPosition,
  normalizeIntervals,
  observationMarkers,
  timelineGaps,
  type TimelineGap,
  type TimelineInterval,
  type TimelineObservationMarker,
  type TimelineWindow,
} from "@/timeline/intervals";
import { msToPx, type TimelineScale } from "./timeline-utils";

const DAY_MS = 86_400_000;

export interface RenderedInterval extends TimelineInterval {
  x: number;
  width: number;
}

export interface RenderedGap extends TimelineGap {
  x: number;
  width: number;
}

export interface ObservationTick extends TimelineObservationMarker {
  x: number;
  color: string;
}

function renderSpan<T extends TimelineInterval | TimelineGap>(
  span: T,
  window: TimelineWindow,
  scale: TimelineScale,
): T & { x: number; width: number } {
  const position = intervalPosition(span, window);
  return {
    ...span,
    x: msToPx(span.startMs, scale),
    width: position.widthPct * scale.widthPx / 100,
  };
}

/** Use only recorded interval endpoints. A point sample never becomes a span. */
export function buildSegments(
  intervals: readonly TimelineInterval[],
  window: TimelineWindow,
  scale: TimelineScale,
  nowMs: number,
): { intervals: RenderedInterval[]; gaps: RenderedGap[] } {
  const exact = normalizeIntervals(intervals, window);
  return {
    intervals: exact.map((interval) => renderSpan(interval, window, scale)),
    gaps: timelineGaps(exact, window, nowMs).map((gap) => renderSpan(gap, window, scale)),
  };
}

/** Observations are 2px × 6px top-edge cues, never filled posture durations. */
export function buildObservationTicks(
  snapshots: SnapshotRow[],
  scale: TimelineScale,
  window: TimelineWindow,
  nowMs = Date.now(),
): ObservationTick[] {
  return observationMarkers(snapshots.map((row) => ({
    ts: row.ts,
    state: row.state,
    connected: row.connected,
    observationQuality: row.observationQuality,
    localDate: row.localDate,
    timezoneOffsetSecs: row.timezoneOffsetSecs,
  })))
    .filter((marker) => marker.atMs >= window.startMs && marker.atMs < window.endMs && marker.atMs <= nowMs)
    .map((marker) => ({
      ...marker,
      x: msToPx(marker.atMs, scale),
      color: marker.state ? timelineColors[marker.state] : "var(--ink-secondary, #c6c6c6)",
    }));
}

/** Local midnight dividers for the legacy date-only scale. */
export function dayBoundaries(scale: TimelineScale): number[] {
  const out: number[] = [];
  const start = new Date(scale.startMs);
  start.setHours(0, 0, 0, 0);
  for (let at = start.getTime(); at <= scale.endMs; at += DAY_MS) {
    if (at >= scale.startMs) out.push(at);
  }
  return out;
}

export interface HourTick {
  ms: number;
  hour: number;
  isLabel: boolean;
}

/** Local-hour context marks; elapsed posture still comes only from intervals. */
export function enumerateHourTicks(scale: TimelineScale): HourTick[] {
  const pxPerHour = scale.pxPerMinute * 60;
  const tickStep = pxPerHour >= 40 ? 1 : pxPerHour >= 20 ? 2 : pxPerHour >= 10 ? 6 : 12;
  const labelStep = pxPerHour >= 40 ? 2 : pxPerHour >= 20 ? 6 : 12;
  const ticks: HourTick[] = [];
  for (const midnight of dayBoundaries(scale)) {
    for (let hour = tickStep; hour < 24; hour += tickStep) {
      const ms = midnight + hour * 3_600_000;
      if (ms >= scale.startMs && ms <= scale.endMs) ticks.push({ ms, hour, isLabel: hour % labelStep === 0 });
    }
  }
  return ticks;
}
