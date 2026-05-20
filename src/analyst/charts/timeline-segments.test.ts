/**
 * timeline-segments.test.ts — unit tests for the continuous-strip helpers.
 */
import { describe, expect, it } from "vitest";
import type { SnapshotRow } from "@/test/analyst-fixtures";
import { buildSegments, dayBoundaries } from "./timeline-segments";
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
