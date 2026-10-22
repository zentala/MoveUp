import { describe, expect, it } from "vitest";
import {
  dayComposition,
  intervalPosition,
  localDayWindow,
  normalizeIntervals,
  observationMarkers,
  recordedTimelineIntervals,
  sessionIntervals,
  timelineGaps,
  type TimelineInterval,
} from "./intervals";

const ms = (value: string) => Date.parse(value);
const start = ms("2026-10-25T00:00:00+02:00");
const fallDay = { startMs: start, endMs: ms("2026-10-26T00:00:00+01:00") };

describe("shared chronological intervals", () => {
  it("keeps recorded elapsed spans separate from credited load and separates Unknown from Future", () => {
    const rows = [
      { ts: "2026-10-25T00:00:00+02:00", state: "sitting", observationQuality: "known" },
      { ts: "2026-10-25T00:01:00+02:00", state: null, observationQuality: null },
      { ts: "2026-10-25T00:03:00+02:00", state: "standing", observationQuality: "known" },
    ];
    const intervals = sessionIntervals([
      { start: "2026-10-25T00:00:00+02:00", end: "2026-10-25T00:01:00+02:00", state: "Sitting", history_kind: "individual" },
      { start: "2026-10-25T00:03:00+02:00", end: "2026-10-25T00:04:00+02:00", state: "Standing", history_kind: "individual" },
    ]);
    expect(observationMarkers(rows)).toHaveLength(3);
    const result = dayComposition(intervals, fallDay, ms("2026-10-25T00:04:00+02:00"));
    expect(result.dayMs).toBe(90_000_000);
    expect(result.totals.Sitting).toBe(60_000);
    expect(result.totals.Standing).toBe(60_000);
    expect(result.totals.Unknown).toBe(120_000);
    expect(result.totals.Future).toBe(89_760_000);
    expect(Object.values(result.totals).reduce((sum, value) => sum + value, 0)).toBe(result.dayMs);
  });

  it("never turns a sample or nominal cadence into a duration interval", () => {
    const rows = [
      { ts: "2026-10-09T10:00:00Z", state: "sitting", observationQuality: "known" },
      { ts: "2026-10-09T10:00:30Z", state: null, observationQuality: null },
      { ts: "2026-10-09T10:10:00Z", state: "standing", observationQuality: "known" },
    ];
    expect(observationMarkers(rows).map(({ state, atMs }) => [state, atMs])).toEqual([
      ["Sitting", ms("2026-10-09T10:00:00Z")],
      [null, ms("2026-10-09T10:00:30Z")],
      ["Standing", ms("2026-10-09T10:10:00Z")],
    ]);
    expect(recordedTimelineIntervals([])).toEqual([]);
  });

  it("uses producer quality/state while preserving physical connectivity as separate evidence", () => {
    const rows = [
      { ts: "2026-10-09T10:00:00Z", state: "standing", observationQuality: "known", connected: false },
      { ts: "2026-10-09T10:01:00Z", state: "walking", observationQuality: null, connected: true },
      { ts: "2026-10-09T10:02:00Z", state: "away", observationQuality: "known", connected: true },
    ];
    const markers = observationMarkers(rows);
    expect(markers.map(({ state }) => state)).toEqual(["Standing", null, "Away"]);
    expect(markers[0].connected).toBe(false);
    expect(markers[0].observationQuality).toBe("known");
    expect(markers[1].observationQuality).toBeNull();
  });

  it("uses actual individual session endpoints and leaves legacy break summaries unqualified", () => {
    const rows = [
      { start: "2026-10-09T09:00:00Z", end: "2026-10-09T09:10:00Z", state: "Sitting", history_kind: "individual" },
      { start: "2026-10-09T09:10:00Z", end: "2026-10-09T09:20:00Z", state: "Sitting", history_kind: null },
      { start: "2026-10-09T09:20:00Z", end: "2026-10-09T09:30:00Z", state: "Sitting", history_kind: "legacy" },
      { start: "2026-10-09T09:10:00Z", end: "2026-10-09T09:30:00Z", state: "Standing", history_kind: null },
      { start: "2026-10-09T09:30:00Z", end: "2026-10-09T09:40:00Z", state: "Standing", history_kind: "individual" },
    ];
    expect(sessionIntervals(rows).map(({ state, startMs, endMs }) => [state, (endMs - startMs) / 1000])).toEqual([
      ["Sitting", 600], ["Standing", 600],
    ]);
  });

  it("rejects conflicts and malformed intervals instead of inventing plausible history", () => {
    const interval: TimelineInterval = { startMs: 10, endMs: 20, state: "Sitting" };
    expect(() => normalizeIntervals([interval, { ...interval, state: "Standing" }])).toThrow(/overlapping/);
    expect(() => normalizeIntervals([{ ...interval, endMs: Number.NaN }])).toThrow(/endpoints/);
  });

  it("computes explicit Unknown/Future coverage and preserves tiny proportions", () => {
    const window = { startMs: 0, endMs: 90_000_000 };
    const tiny = { startMs: 100, endMs: 110, state: "Standing" as const };
    expect(intervalPosition(tiny, window).widthPct).toBeCloseTo(10 / 90_000_000 * 100, 14);
    expect(timelineGaps([tiny], window, 45_000_000).map(({ kind, endMs, startMs }) => [kind, endMs - startMs])).toEqual([
      ["Unknown", 100], ["Unknown", 44_999_890], ["Future", 45_000_000],
    ]);
  });

  it("uses source offsets for 23-hour and 25-hour local-day bounds", () => {
    expect(localDayWindow("2026-03-29", 3600, 7200).endMs - localDayWindow("2026-03-29", 3600, 7200).startMs).toBe(82_800_000);
    expect(localDayWindow("2026-10-25", 7200, 3600).endMs - localDayWindow("2026-10-25", 7200, 3600).startMs).toBe(90_000_000);
  });

  it("requires both source boundary offsets instead of repeating one sample offset", () => {
    expect(() => {
      // @ts-expect-error The producer must supply the source offset at both boundaries.
      localDayWindow("2026-03-29", 3600);
    }).toThrow();
  });
});
