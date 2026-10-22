import { describe, expect, it } from "vitest";
import type { SnapshotRow } from "@/test/analyst-fixtures";
import { dayComposition } from "@/timeline/intervals";
import { timelineColors } from "@/timeline/colors";
import { buildSegments, buildObservationTicks, dayBoundaries, enumerateHourTicks } from "./timeline-segments";
import { buildTimelineScale, buildTimelineScaleForWindow } from "./timeline-utils";

function snap(ts: string, state: SnapshotRow["state"], quality: string | null = "known"): SnapshotRow {
  return {
    ts, state, observationQuality: quality, connected: true,
    deskHeightCm: 75, sittingSecs: 0, standingSecs: 0, breakSecs: 0, idleSecs: 0, score: 50,
  };
}

describe("buildSegments", () => {
  const window = { startMs: Date.parse("2026-05-17T00:00:00Z"), endMs: Date.parse("2026-05-18T00:00:00Z") };
  const scale = buildTimelineScale("2026-05-17", "2026-05-17", 0, 1440);

  it("keeps exact recorded spans and represents uncovered elapsed time as Unknown", () => {
    const startMs = Date.parse("2026-05-17T08:00:00Z");
    const endMs = Date.parse("2026-05-17T08:20:00Z");
    const result = buildSegments([
      { startMs, endMs, state: "Standing", provenance: "session" },
    ], window, scale, Date.parse("2026-05-17T12:00:00Z"));
    expect(result.intervals).toHaveLength(1);
    expect(result.intervals[0]).toMatchObject({ startMs, endMs, state: "Standing", width: 20 });
    expect(result.gaps).toEqual(expect.arrayContaining([
      expect.objectContaining({ kind: "Unknown", startMs: window.startMs, endMs: startMs }),
      expect.objectContaining({ kind: "Unknown", startMs: endMs, endMs: Date.parse("2026-05-17T12:00:00Z") }),
      expect.objectContaining({ kind: "Future", startMs: Date.parse("2026-05-17T12:00:00Z"), endMs: window.endMs }),
    ]));
  });

  it("does not infer spans from one or more snapshots", () => {
    const ticks = buildObservationTicks([
      snap("2026-05-17T08:00:00Z", "Sitting"),
      snap("2026-05-17T08:20:00Z", "Standing"),
    ], scale, window);
    expect(ticks).toHaveLength(2);
    expect(ticks.map(({ atMs, state }) => [atMs, state])).toEqual([
      [Date.parse("2026-05-17T08:00:00Z"), "Sitting"],
      [Date.parse("2026-05-17T08:20:00Z"), "Standing"],
    ]);
    expect(ticks.every((tick) => tick.color === timelineColors.Sitting || tick.color === timelineColors.Standing)).toBe(true);
  });

  it("clips observations to the source window and uses secondary ink when quality is unknown", () => {
    const ticks = buildObservationTicks([
      snap("2026-05-16T23:59:00Z", "Sitting"),
      snap("2026-05-17T08:00:00Z", "Away", null),
      snap("2026-05-18T00:00:00Z", "Standing"),
    ], scale, window);
    expect(ticks).toHaveLength(1);
    expect(ticks[0]).toMatchObject({ state: null, color: "var(--ink-secondary, #c6c6c6)" });
  });

  it("does not paint a known observation after the supplied current time as confirmed history", () => {
    const now = Date.parse("2026-05-17T10:00:00Z");
    const ticks = buildObservationTicks([
      snap("2026-05-17T09:59:00Z", "Standing"),
      snap("2026-05-17T10:01:00Z", "Walking"),
    ], scale, window, now);
    expect(ticks.map(({ atMs, state }) => [atMs, state])).toEqual([
      [Date.parse("2026-05-17T09:59:00Z"), "Standing"],
    ]);
  });

  it("preserves fractional proportions without a visual minimum", () => {
    const startMs = Date.parse("2026-05-17T08:00:00Z");
    const endMs = startMs + 10;
    const result = buildSegments([{ startMs, endMs, state: "Standing" }], window, scale, window.endMs);
    expect(result.intervals[0].width).toBeCloseTo(10 / 60_000, 10);
  });

  it("renders only elapsed portions and keeps the same Future coverage as the day ring", () => {
    const now = Date.parse("2026-05-17T12:00:00Z");
    const recorded = [
      { startMs: Date.parse("2026-05-17T09:00:00Z"), endMs: Date.parse("2026-05-17T10:00:00Z"), state: "Sitting" as const },
      { startMs: Date.parse("2026-05-17T11:50:00Z"), endMs: Date.parse("2026-05-17T12:20:00Z"), state: "Standing" as const },
      { startMs: Date.parse("2026-05-17T16:00:00Z"), endMs: Date.parse("2026-05-17T17:00:00Z"), state: "Walking" as const },
    ];

    const rendered = buildSegments(recorded, window, scale, now);
    const ring = dayComposition(recorded, window, now);

    expect(rendered.intervals.map(({ startMs, endMs, state }) => [startMs, endMs, state])).toEqual([
      [recorded[0].startMs, recorded[0].endMs, "Sitting"],
      [recorded[1].startMs, now, "Standing"],
    ]);
    expect(rendered.gaps.find(({ kind }) => kind === "Future")).toMatchObject({ startMs: now, endMs: window.endMs });
    expect(ring.totals.Future).toBe(window.endMs - now);
    expect(rendered.gaps.filter(({ kind }) => kind === "Future").reduce((sum, gap) => sum + gap.endMs - gap.startMs, 0))
      .toBe(ring.totals.Future);
    expect(recorded[1].endMs).toBe(Date.parse("2026-05-17T12:20:00Z"));
    expect(recorded[2].startMs).toBe(Date.parse("2026-05-17T16:00:00Z"));
  });

  it("shows a future local day entirely as Future, matching its ring composition", () => {
    const futureWindow = {
      startMs: Date.parse("2026-05-18T00:00:00Z"),
      endMs: Date.parse("2026-05-19T00:00:00Z"),
    };
    const now = Date.parse("2026-05-17T12:00:00Z");
    const rendered = buildSegments([], futureWindow, buildTimelineScaleForWindow(futureWindow, 1440), now);
    const ring = dayComposition([], futureWindow, now);

    expect(rendered.intervals).toEqual([]);
    expect(rendered.gaps).toEqual([{ ...futureWindow, kind: "Future", x: 0, width: 1440 }]);
    expect(ring.totals.Future).toBe(ring.dayMs);
  });
});

describe("legacy axis annotations", () => {
  it("keeps day dividers and readable hour ticks across a visible range", () => {
    const scale = buildTimelineScale("2026-05-15", "2026-05-17", 0, 4000);
    expect(dayBoundaries(scale).length).toBeGreaterThanOrEqual(3);
    const ticks = enumerateHourTicks(scale);
    expect(ticks.length).toBeGreaterThan(0);
    expect(ticks.some((tick) => tick.hour === 0)).toBe(false);
  });
});
