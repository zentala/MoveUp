import { describe, it, expect } from "vitest";
import type { DailyKpi, SessionRow, SnapshotRow } from "@/test/analyst-fixtures";
import { aggregateDayKpis } from "./explorer-day-kpis";

const DAY = "2026-05-16";

function snap(over: Partial<SnapshotRow> & { ts: string }): SnapshotRow {
  return {
    localDate: over.localDate ?? over.ts.slice(0, 10),
    state: "Sitting",
    deskHeightCm: 72,
    sittingSecs: 0,
    standingSecs: 0,
    breakSecs: 0,
    idleSecs: 0,
    score: 0,
    sittingSecsTotal: over.sittingSecs ?? 0,
    positionChanges: 0,
    ...over,
  };
}

function session(over: Partial<SessionRow> & { id: number }): SessionRow {
  return {
    startedAt: `${DAY}T09:00:00.000Z`,
    endedAt: `${DAY}T09:30:00.000Z`,
    state: "Sitting",
    durationSecs: 1800,
    sittingSecs: 1800,
    standingSecs: 0,
    positionChanges: 0,
    breakCredit: "none",
    dateLocal: DAY,
    ...over,
  };
}

describe("aggregateDayKpis", () => {
  it("returns an empty, flagged day when nothing is recorded", () => {
    const result = aggregateDayKpis([], [], DAY);
    expect(result).toEqual({
      dateLocal: DAY,
      hasData: false,
      sittingSecs: null,
      standingSecs: null,
      activeSecs: null,
      standingPct: null,
      positionChanges: null,
      score: null,
    });
  });

  it("reads the day's totals from its last snapshot", () => {
    const rows = [
      snap({ ts: `${DAY}T09:00:00.000Z`, sittingSecs: 600, standingSecs: 0, score: 10 }),
      snap({ ts: `${DAY}T17:00:00.000Z`, sittingSecs: 3600, standingSecs: 1200, score: 42 }),
      snap({ ts: "2026-05-15T23:00:00.000Z", sittingSecs: 99_999, standingSecs: 0, score: 99 }),
    ];
    const result = aggregateDayKpis(rows, [], DAY);
    expect(result.hasData).toBe(true);
    expect(result.sittingSecs).toBe(3600);
    expect(result.standingSecs).toBe(1200);
    expect(result.activeSecs).toBe(4800);
    expect(result.standingPct).toBeCloseTo(0.25, 5);
    expect(result.score).toBe(42);
  });

  it("reports 0% standing for an all-sit day", () => {
    const rows = [snap({ ts: `${DAY}T17:00:00.000Z`, sittingSecs: 7200, standingSecs: 0 })];
    const result = aggregateDayKpis(rows, [], DAY);
    expect(result.standingPct).toBe(0);
    expect(result.hasData).toBe(true);
  });

  it("reports 100% standing for an all-stand day", () => {
    const rows = [snap({ ts: `${DAY}T17:00:00.000Z`, sittingSecs: 0, standingSecs: 7200 })];
    expect(aggregateDayKpis(rows, [], DAY).standingPct).toBe(1);
  });

  it("guards against dividing by zero active time", () => {
    const rows = [snap({ ts: `${DAY}T08:00:00.000Z`, state: "Away" })];
    const result = aggregateDayKpis(rows, [], DAY);
    expect(result.standingPct).toBeNull();
    expect(result.hasData).toBe(true);
  });

  it("sums posture changes across the day's sessions", () => {
    const sessions = [
      session({ id: 1, positionChanges: 2 }),
      session({ id: 2, positionChanges: 3 }),
      session({ id: 3, positionChanges: 9, dateLocal: "2026-05-15" }),
    ];
    expect(aggregateDayKpis([], sessions, DAY).positionChanges).toBe(5);
  });

  it("uses the recorded snapshot event counter instead of inferred state changes", () => {
    const rows = [
      snap({ ts: `${DAY}T09:00:00.000Z`, state: "Sitting", positionChanges: 7 }),
      snap({ ts: `${DAY}T10:00:00.000Z`, state: "Standing", positionChanges: 7 }),
      snap({ ts: `${DAY}T11:00:00.000Z`, state: "Standing", positionChanges: 7 }),
      snap({ ts: `${DAY}T12:00:00.000Z`, state: "Sitting", positionChanges: 7 }),
    ];
    const result = aggregateDayKpis(rows, [session({ id: 1 })], DAY);
    expect(result.positionChanges).toBe(7);
  });

  it("falls back to the precomputed rollup when raw data yields no changes", () => {
    const rows = [snap({ ts: `${DAY}T09:00:00.000Z`, state: "Sitting", score: 30, positionChanges: null })];
    const rollup: DailyKpi[] = [
      { dateLocal: DAY, standingPct: 40, positionChanges: 7, longestSessionSecs: 900, score: 55 },
    ];
    const result = aggregateDayKpis(rows, [], DAY, rollup);
    expect(result.positionChanges).toBe(7);
    // snapshots win for score — they are the measured value
    expect(result.score).toBe(30);
  });

  it("uses the rollup outright when the day has no snapshots or sessions", () => {
    const rollup: DailyKpi[] = [
      { dateLocal: DAY, standingPct: 62, positionChanges: 6, longestSessionSecs: 900, score: 71 },
    ];
    const result = aggregateDayKpis([], [], DAY, rollup);
    expect(result.hasData).toBe(true);
    expect(result.standingPct).toBeCloseTo(0.62, 5);
    expect(result.positionChanges).toBe(6);
    expect(result.score).toBe(71);
    expect(result.standingSecs).toBeNull();
  });

  it("ignores a rollup for a different day", () => {
    const rollup: DailyKpi[] = [
      { dateLocal: "2026-05-15", standingPct: 62, positionChanges: 6, longestSessionSecs: 900, score: 71 },
    ];
    expect(aggregateDayKpis([], [], DAY, rollup).hasData).toBe(false);
  });

  it("preserves signed scores without a 0-to-100 clamp", () => {
    const rows = [snap({ ts: `${DAY}T17:00:00.000Z`, score: -480 })];
    expect(aggregateDayKpis(rows, [], DAY).score).toBe(-480);
  });

  it("keeps unavailable raw counters and scores unknown", () => {
    const result = aggregateDayKpis([
      snap({
        ts: `${DAY}T17:00:00.000Z`,
        sittingSecs: 1200,
        sittingSecsTotal: null,
        score: null,
        positionChanges: null,
      }),
    ], [], DAY);
    expect(result.hasData).toBe(true);
    expect(result.sittingSecs).toBeNull();
    expect(result.activeSecs).toBeNull();
    expect(result.standingPct).toBeNull();
    expect(result.score).toBeNull();
    expect(result.positionChanges).toBeNull();
  });

  it("does not infer producer date or raw sitting from legacy snapshot fields", () => {
    const row: SnapshotRow = {
      ts: `${DAY}T17:00:00.000Z`,
      state: "Sitting",
      deskHeightCm: 72,
      sittingSecs: 1200,
      standingSecs: 0,
      breakSecs: 0,
      idleSecs: 0,
      score: 4,
    };
    const unknownDate = aggregateDayKpis([row], [], DAY);
    expect(unknownDate.hasData).toBe(false);
    expect(unknownDate.sittingSecs).toBeNull();

    const knownDate = aggregateDayKpis([{ ...row, localDate: DAY }], [], DAY);
    expect(knownDate.hasData).toBe(true);
    expect(knownDate.sittingSecs).toBeNull();
    expect(knownDate.activeSecs).toBeNull();
  });
});
