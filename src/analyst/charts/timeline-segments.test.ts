/**
 * timeline-segments.test.ts — unit tests for the continuous-strip helpers.
 */
import { describe, expect, it } from "vitest";
import type { SnapshotRow } from "@/test/analyst-fixtures";
import { buildSegments, dayBoundaries, enumerateHourTicks } from "./timeline-segments";
import { buildTimelineScale } from "./timeline-utils";

function snap(ts: string, state: SnapshotRow["state"]): SnapshotRow {
  return {
    ts,
    state,
    deskHeightCm: 75,
    sittingSecs: 0,
    standingSecs: 0,
    breakSecs: 0,
    idleSecs: 0,
    score: 50,
  };
}

describe("buildSegments", () => {
  const scale = buildTimelineScale("2026-05-17", "2026-05-17", 0, 1440);

  it("returns empty array when no rows", () => {
    expect(buildSegments([], scale)).toEqual([]);
  });

  it("each segment runs from its timestamp to the next snapshot (continuous)", () => {
    const rows = [
      snap("2026-05-17T08:00:00.000", "Sitting"),
      snap("2026-05-17T08:20:00.000", "Standing"),
      snap("2026-05-17T08:40:00.000", "Sitting"),
    ];
    const segs = buildSegments(rows, scale);
    expect(segs.length).toBe(3);
    // Segments touch — end of seg 0 == start of seg 1.
    expect(segs[0].x + segs[0].w).toBeCloseTo(segs[1].x, 1);
    expect(segs[1].x + segs[1].w).toBeCloseTo(segs[2].x, 1);
  });

  it("last segment uses inferred median interval as its width", () => {
    const rows = [
      snap("2026-05-17T08:00:00.000", "Sitting"),
      snap("2026-05-17T08:20:00.000", "Standing"),
    ];
    const segs = buildSegments(rows, scale);
    expect(segs.length).toBe(2);
    // Last seg width should equal one 20-min interval mapped to px.
    const oneInterval = scale.pxPerMinute * 20;
    expect(segs[1].w).toBeCloseTo(oneInterval, 0);
  });

  it("clips segments to scale boundaries", () => {
    // Snapshot from yesterday should be clipped on the left, future on the right.
    const rows = [
      snap("2026-05-16T18:00:00.000", "Sitting"), // before scale.startMs
      snap("2026-05-18T03:00:00.000", "Standing"), // after scale.endMs
    ];
    const segs = buildSegments(rows, scale);
    // Both clipped — first to fit at left edge, second falls outside scale.
    for (const seg of segs) {
      expect(seg.x).toBeGreaterThanOrEqual(0);
      expect(seg.x + seg.w).toBeLessThanOrEqual(scale.widthPx + 0.001);
    }
  });
});

describe("enumerateHourTicks", () => {
  it("never includes 00:00 (those are day-dividers)", () => {
    const scale = buildTimelineScale("2026-05-17", "2026-05-17", 0, 1440);
    const ticks = enumerateHourTicks(scale);
    expect(ticks.some((t) => t.hour === 0)).toBe(false);
  });

  it("hourly density when zoomed in (>= 40 px/hour)", () => {
    // 24h * 50px/hour = 1200px width → 50 px/hour
    const scale = buildTimelineScale("2026-05-17", "2026-05-17", 0, 1200);
    const ticks = enumerateHourTicks(scale);
    // Hours 1..23 = 23 ticks, labels at even hours = 11 labels (2,4,...,22).
    expect(ticks.length).toBe(23);
    expect(ticks.filter((t) => t.isLabel).length).toBe(11);
  });

  it("6h density on a typical 14-day view (~10-20 px/hour)", () => {
    // 14d * 24h = 336h; 4000px / 336h ~ 11.9 px/h → tier 10..20 → step=6
    const scale = buildTimelineScale("2026-05-04", "2026-05-17", 0, 4000);
    const ticks = enumerateHourTicks(scale);
    // Per day: hours 6,12,18 = 3 ticks; 14 days = 42 ticks. Labels every 12 → 12,24-omit → hour 12 only.
    expect(ticks.length).toBe(42);
    expect(ticks.every((t) => [6, 12, 18].includes(t.hour))).toBe(true);
    expect(ticks.filter((t) => t.isLabel).every((t) => t.hour === 12)).toBe(true);
  });

  it("falls back to 12h step on very compressed views (< 10 px/hour)", () => {
    // 30d * 24h = 720h; 1000px / 720h ~ 1.4 px/h → tier <10 → step=12
    const scale = buildTimelineScale("2026-04-18", "2026-05-17", 0, 1000);
    const ticks = enumerateHourTicks(scale);
    // Only hour=12 per day, 30 days → 30 ticks.
    expect(ticks.length).toBe(30);
    expect(ticks.every((t) => t.hour === 12 && t.isLabel)).toBe(true);
  });
});

describe("dayBoundaries", () => {
  it("emits midnight ms inside the scale range", () => {
    const scale = buildTimelineScale("2026-05-15", "2026-05-17", 0, 1000);
    const out = dayBoundaries(scale);
    // 3 day boundaries (15, 16, 17), plus maybe day-after start = 18.
    expect(out.length).toBeGreaterThanOrEqual(3);
    for (const t of out) {
      const d = new Date(t);
      expect(d.getHours()).toBe(0);
      expect(d.getMinutes()).toBe(0);
    }
  });
});
