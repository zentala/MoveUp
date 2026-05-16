/**
 * explorer-derivations.test.ts — Unit tests for derivation helpers.
 */
import { describe, it, expect } from "vitest";
import { downsampleSnapshots, deriveDailyKpis } from "./explorer-derivations";
import type { SnapshotRow } from "@/test/analyst-fixtures";

function snap(ts: string, h: number, sit = 0, stand = 0, score = 0): SnapshotRow {
  return {
    ts,
    state: "Sitting",
    deskHeightCm: h,
    sittingSecs: sit,
    standingSecs: stand,
    breakSecs: 0,
    idleSecs: 0,
    score,
  };
}

describe("downsampleSnapshots", () => {
  it("returns input unchanged when below threshold", () => {
    const rows = [snap("a", 80), snap("b", 90)];
    expect(downsampleSnapshots(rows, 1000)).toEqual(rows);
  });

  it("compresses to roughly the target bucket count", () => {
    const rows = Array.from({ length: 2400 }, (_, i) =>
      snap(`t${i}`, 80 + (i % 10)),
    );
    const out = downsampleSnapshots(rows, 1000);
    expect(out.length).toBeLessThanOrEqual(1000);
    expect(out.length).toBeGreaterThan(500);
  });
});

describe("deriveDailyKpis", () => {
  it("produces one row per day with the last snapshot as the source", () => {
    const rows: SnapshotRow[] = [
      snap("2026-05-15T10:00:00Z", 80, 600, 200, 30),
      snap("2026-05-15T18:00:00Z", 80, 900, 300, 50),
      snap("2026-05-16T09:00:00Z", 80, 0, 400, 10),
    ];
    const kpis = deriveDailyKpis(rows);
    expect(kpis).toHaveLength(2);
    expect(kpis[0].dateLocal).toBe("2026-05-15");
    expect(kpis[0].standingPct).toBe(25);
    expect(kpis[0].score).toBe(50);
    expect(kpis[1].dateLocal).toBe("2026-05-16");
    expect(kpis[1].standingPct).toBe(100);
  });
});
