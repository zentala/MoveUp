import type { DeskState } from "@/types";
import type { LiveObservation } from "@/generated/LiveObservation";

export interface TimelineWindow { startMs: number; endMs: number }
export type TimelineLabel = DeskState | "Unknown" | "Future";
export interface TimelineInterval extends TimelineWindow {
  state: DeskState;
  current?: boolean;
  provenance?: "session" | "snapshot" | "live";
}
export interface TimelineGap extends TimelineWindow { kind: "Unknown" | "Future" }

export interface StateObservation {
  ts: string;
  state: string | null;
  connected?: boolean | null;
  observationQuality?: string | null;
  localDate?: string | null;
  timezoneOffsetSecs?: number | null;
}
export interface TimelineObservationMarker {
  atMs: number;
  state: DeskState | null;
  provenance: "snapshot";
  observationQuality: string | null;
  connected: boolean | null;
  localDate: string | null;
  timezoneOffsetSecs: number | null;
}

export interface SessionIntervalEntry {
  start: string;
  end: string | null;
  state: string;
  history_kind?: string | null;
}

const KNOWN_STATES = new Set<DeskState>(["Sitting", "Standing", "Walking", "Away"]);
const DAY_MS = 86_400_000;

function deskState(value: string | null | undefined): DeskState | null {
  if (!value) return null;
  const match = value.toLowerCase();
  if (match === "sitting") return "Sitting";
  if (match === "standing") return "Standing";
  if (match === "walking") return "Walking";
  if (match === "away") return "Away";
  return null;
}

export class TimelineDataError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "TimelineDataError";
  }
}

/** Validate a source calendar date and use the producer's offsets at both boundaries. */
export function localDayWindow(
  dateLocal: string,
  offsetAtStartSecs: number,
  offsetAtEndSecs: number,
): TimelineWindow {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(dateLocal);
  if (!match || !Number.isFinite(offsetAtStartSecs) || !Number.isFinite(offsetAtEndSecs)) {
    throw new TimelineDataError("Invalid local calendar date or source offset");
  }
  const [, year, month, day] = match;
  const startUtc = Date.UTC(Number(year), Number(month) - 1, Number(day));
  const check = new Date(startUtc);
  if (check.toISOString().slice(0, 10) !== dateLocal) {
    throw new TimelineDataError("Invalid local calendar date");
  }
  const endUtc = startUtc + DAY_MS;
  const startMs = startUtc - offsetAtStartSecs * 1000;
  const endMs = endUtc - offsetAtEndSecs * 1000;
  if (endMs <= startMs) throw new TimelineDataError("Invalid local calendar bounds");
  return { startMs, endMs };
}

/** Validate, clip and sort half-open intervals; conflicting evidence is an error. */
export function normalizeIntervals(
  intervals: readonly TimelineInterval[],
  window?: TimelineWindow,
): TimelineInterval[] {
  const normalized = intervals.flatMap((interval) => {
    if (!Number.isFinite(interval.startMs) || !Number.isFinite(interval.endMs) || interval.endMs <= interval.startMs) {
      throw new TimelineDataError("Invalid interval endpoints");
    }
    if (!KNOWN_STATES.has(interval.state)) return [];
    const clipped = {
      ...interval,
      startMs: Math.max(interval.startMs, window?.startMs ?? Number.NEGATIVE_INFINITY),
      endMs: Math.min(interval.endMs, window?.endMs ?? Number.POSITIVE_INFINITY),
    };
    return clipped.endMs > clipped.startMs ? [clipped] : [];
  }).sort((a, b) => a.startMs - b.startMs || a.endMs - b.endMs);

  const result: TimelineInterval[] = [];
  for (const interval of normalized) {
    const previous = result[result.length - 1];
    if (previous && previous.startMs === interval.startMs && previous.endMs === interval.endMs &&
      previous.state === interval.state && previous.provenance === interval.provenance) continue;
    if (previous && previous.endMs > interval.startMs) {
      throw new TimelineDataError(`Conflicting overlapping intervals: ${previous.state} / ${interval.state}`);
    }
    result.push(interval);
  }
  return result;
}

/** Only individually recorded endpoints establish a chronological state interval. */
export function sessionIntervals(rows: readonly SessionIntervalEntry[]): TimelineInterval[] {
  const intervals = rows.flatMap((row) => {
    if (row.end === null || row.history_kind !== "individual") return [];
    const state = deskState(row.state);
    if (state === null) return [];
    return [{
      startMs: Date.parse(row.start),
      endMs: Date.parse(row.end),
      state,
      provenance: "session" as const,
    }];
  });
  return normalizeIntervals(intervals);
}

/**
 * Build bounded sample spans. The caller must supply the producer cadence bound;
 * unsupported, stale or disconnected rows still cap the preceding observation.
 */
/** Snapshot cadence describes sampling frequency, not a posture interval. */
export function observationMarkers(rows: readonly StateObservation[]): TimelineObservationMarker[] {
  const ordered = rows.map((row) => ({ ...row, atMs: Date.parse(row.ts) }))
    .sort((a, b) => a.atMs - b.atMs);
  if (ordered.some((row) => !Number.isFinite(row.atMs))) {
    throw new TimelineDataError("Invalid snapshot timestamp");
  }
  const markers: TimelineObservationMarker[] = [];
  for (const row of ordered) {
    const previous = markers[markers.length - 1];
    const state = row.observationQuality === "known"
      ? deskState(row.state)
      : null;
    const marker: TimelineObservationMarker = {
      atMs: row.atMs,
      state,
      provenance: "snapshot",
      observationQuality: row.observationQuality ?? null,
      connected: row.connected ?? null,
      localDate: row.localDate ?? null,
      timezoneOffsetSecs: row.timezoneOffsetSecs ?? null,
    };
    if (previous?.atMs === marker.atMs) {
      if (previous.state !== marker.state || previous.observationQuality !== marker.observationQuality ||
        previous.connected !== marker.connected || previous.localDate !== marker.localDate ||
        previous.timezoneOffsetSecs !== marker.timezoneOffsetSecs) {
        throw new TimelineDataError("Conflicting snapshots at the same timestamp");
      }
      continue;
    }
    markers.push(marker);
  }
  return markers;
}

/** Recorded individual history wins over approximate samples only on its actual span. */
export function preferRecordedIntervals(
  recorded: readonly TimelineInterval[],
  approximate: readonly TimelineInterval[],
): TimelineInterval[] {
  const primary = normalizeIntervals(recorded);
  let remainder = normalizeIntervals(approximate);
  for (const exact of primary) {
    remainder = remainder.flatMap((sample) => {
      if (sample.endMs <= exact.startMs || sample.startMs >= exact.endMs) return [sample];
      return [
        ...(sample.startMs < exact.startMs ? [{ ...sample, endMs: exact.startMs }] : []),
        ...(sample.endMs > exact.endMs ? [{ ...sample, startMs: exact.endMs }] : []),
      ];
    });
  }
  return normalizeIntervals([...primary, ...remainder]);
}

export function recordedTimelineIntervals(sessions: readonly SessionIntervalEntry[]): TimelineInterval[] {
  return sessionIntervals(sessions);
}

/** Retain observed prefixes until an individually recorded row replaces them. */
export function retainLiveObservationIntervals(
  retained: readonly LiveObservation[],
  current: LiveObservation | null | undefined,
  sessions: readonly SessionIntervalEntry[],
): LiveObservation[] {
  const bySpan = new Map<string, LiveObservation>();
  for (const observation of [...retained, ...(current ? [current] : [])]) {
    const span = observation.span;
    if (!span) continue;
    const key = `${span.state}\u0000${span.started_at}`;
    const previous = bySpan.get(key);
    if (!previous || Date.parse(span.observed_through) >= Date.parse(previous.span!.observed_through)) {
      bySpan.set(key, observation);
    }
  }

  const recorded = sessionIntervals(sessions);
  return [...bySpan.values()].filter((observation) => {
    const span = observation.span!;
    const startMs = Date.parse(span.started_at);
    const endMs = Date.parse(span.observed_through);
    return !recorded.some((row) => row.state === span.state && row.startMs <= startMs && row.endMs >= endMs);
  });
}

/** Merge individually recorded spans with the producer's confirmed live prefix. */
export function mergeTimelineIntervals(
  sessions: readonly SessionIntervalEntry[],
  observation: LiveObservation | null | undefined,
  window?: TimelineWindow,
  nowMs = Date.now(),
  retained: readonly LiveObservation[] = [],
): TimelineInterval[] {
  const recorded = normalizeIntervals(sessionIntervals(sessions), window);
  const bySpan = new Map<string, TimelineInterval>();
  for (const candidate of [...retained, ...(observation ? [observation] : [])]) {
    const span = candidate.span;
    if (!span) continue;
    const startMs = Date.parse(span.started_at);
    const endMs = Date.parse(span.observed_through);
    const snapshotMs = Date.parse(candidate.snapshot_at);
    const lastObservationMs = candidate.last_observation_at == null
      ? Number.NaN
      : Date.parse(candidate.last_observation_at);
    const isCurrent = candidate === observation;
    const supportedQuality = candidate.quality === "known" && candidate.reason === null ||
      candidate.quality === null && ["stale", "pending_confirmation"].includes(candidate.reason ?? "");
    if (!supportedQuality || !Number.isFinite(startMs) || !Number.isFinite(endMs) ||
        !Number.isFinite(snapshotMs) || !Number.isFinite(lastObservationMs) ||
        endMs <= startMs || endMs > lastObservationMs || lastObservationMs > snapshotMs ||
        (isCurrent && (snapshotMs > nowMs || endMs > nowMs)) || !KNOWN_STATES.has(span.state)) {
      throw new TimelineDataError("Invalid or unsupported live observation span");
    }
    const key = `${span.state}\u0000${span.started_at}`;
    const previous = bySpan.get(key);
    if (!previous || endMs > previous.endMs || (isCurrent && endMs === previous.endMs)) {
      bySpan.set(key, {
        startMs,
        endMs,
        state: span.state,
        current: isCurrent,
        provenance: "live",
      });
    }
  }
  const live = normalizeIntervals([...bySpan.values()], window);
  return preferRecordedIntervals(recorded, live);
}

/** Every elapsed hole remains Unknown; time after now remains a separate Future span. */
export function timelineGaps(
  intervals: readonly TimelineInterval[],
  window: TimelineWindow,
  nowMs = Date.now(),
): TimelineGap[] {
  if (window.endMs <= window.startMs) throw new TimelineDataError("Invalid timeline window");
  const elapsedEnd = Math.min(window.endMs, Math.max(window.startMs, nowMs));
  const observed = normalizeIntervals(intervals, { startMs: window.startMs, endMs: elapsedEnd });
  const gaps: TimelineGap[] = [];
  let cursor = window.startMs;
  for (const interval of observed) {
    if (interval.startMs > cursor) gaps.push({ startMs: cursor, endMs: interval.startMs, kind: "Unknown" });
    cursor = interval.endMs;
  }
  if (cursor < elapsedEnd) gaps.push({ startMs: cursor, endMs: elapsedEnd, kind: "Unknown" });
  if (elapsedEnd < window.endMs) gaps.push({ startMs: elapsedEnd, endMs: window.endMs, kind: "Future" });
  return gaps;
}

export function dayComposition(
  intervals: readonly TimelineInterval[],
  window: TimelineWindow,
  nowMs = Date.now(),
): { dayMs: number; totals: Record<TimelineLabel, number> } {
  const totals: Record<TimelineLabel, number> = {
    Sitting: 0, Standing: 0, Walking: 0, Away: 0, Unknown: 0, Future: 0,
  };
  const elapsed = normalizeIntervals(intervals, {
    startMs: window.startMs,
    endMs: Math.min(window.endMs, Math.max(window.startMs, nowMs)),
  });
  for (const interval of elapsed) totals[interval.state] += interval.endMs - interval.startMs;
  for (const gap of timelineGaps(elapsed, window, nowMs)) totals[gap.kind] += gap.endMs - gap.startMs;
  return { dayMs: window.endMs - window.startMs, totals };
}

export function intervalPosition(interval: TimelineWindow, window: TimelineWindow): { leftPct: number; widthPct: number } {
  const duration = window.endMs - window.startMs;
  if (duration <= 0) return { leftPct: 0, widthPct: 0 };
  return {
    leftPct: ((interval.startMs - window.startMs) / duration) * 100,
    widthPct: ((interval.endMs - interval.startMs) / duration) * 100,
  };
}
