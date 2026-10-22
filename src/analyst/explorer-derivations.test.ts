/**
 * explorer-derivations.test.ts — Unit tests for derivation helpers.
 */
import { describe, it, expect } from "vitest";
import type { SnapshotRow as SnapshotWireRow } from "@/generated/SnapshotRow";
import {
  coerceSnapshotRow,
  coerceSessionRow,
  deriveLongestContiguousStandingSecs,
  deriveDailyKpis,
  downsampleSnapshots,
} from "./explorer-derivations";
import type { SessionRow, SnapshotRow } from "@/test/analyst-fixtures";

function session(startedAt: string, endedAt: string, state: SessionRow["state"], historyKind: string | null = "individual"): SessionRow {
  return {
    id: 0, startedAt, endedAt, state, durationSecs: (Date.parse(endedAt) - Date.parse(startedAt)) / 1000,
    sittingSecs: state === "Sitting" ? (Date.parse(endedAt) - Date.parse(startedAt)) / 1000 : null,
    standingSecs: state === "Standing" ? (Date.parse(endedAt) - Date.parse(startedAt)) / 1000 : null,
    positionChanges: null, breakCredit: null, historyKind, dateLocal: "2026-05-15",
  };
}

function snap(ts: string, h: number, sit = 0, stand = 0, score = 0): SnapshotRow {
  return {
    ts,
    localDate: ts.slice(0, 10),
    state: "Sitting",
    deskHeightCm: h,
    sittingSecs: sit,
    standingSecs: stand,
    breakSecs: 0,
    idleSecs: 0,
    score,
    sittingSecsTotal: sit,
    positionChanges: 0,
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

describe("coerceSnapshotRow", () => {
  it("uses the generated Rust row and preserves measured values and local date", () => {
    const row: SnapshotWireRow = {
      ts: "2026-10-22T22:30:00Z",
      state: "Standing",
      sitting_seconds: 75,
      sitting_seconds_total: 126,
      standing_seconds: 42,
      break_seconds: 19,
      desk_height_cm: 81,
      idle_secs: 7,
      continuous_computer_secs: 510,
      position_changes: 9,
      daily_score: -12.5,
      connected: true,
      observation_quality: "known",
      local_date: "2026-10-23",
      timezone_offset_secs: 5400,
    };

    expect(coerceSnapshotRow(row)).toEqual({
      ts: row.ts,
      state: "Standing",
      deskHeightCm: 81,
      sittingSecs: 75,
      standingSecs: 42,
      breakSecs: 19,
      idleSecs: 7,
      score: -12.5,
      sittingSecsTotal: 126,
      connected: true,
      observationQuality: "known",
      localDate: "2026-10-23",
      timezoneOffsetSecs: 5400,
      positionChanges: 9,
    });
  });

  it("keeps legacy, unknown posture and score values null", () => {
    const row: SnapshotWireRow = {
      ts: "2026-10-22T22:30:00Z",
      state: "unrecognized",
      sitting_seconds: 75,
      sitting_seconds_total: null,
      standing_seconds: 42,
      break_seconds: 19,
      desk_height_cm: 81,
      idle_secs: 7,
      continuous_computer_secs: 510,
      position_changes: 0,
      daily_score: null,
      connected: true,
      observation_quality: null,
      local_date: null,
      timezone_offset_secs: null,
    };
    const coerced = coerceSnapshotRow(row);
    expect(coerced.state).toBeNull();
    expect(coerced.score).toBeNull();
    expect(coerced.sittingSecsTotal).toBeNull();
    expect(coerced.observationQuality).toBeNull();
    expect(coerced.connected).toBe(true);
    expect(coerced.localDate).toBeNull();
  });
});

describe("coerceSessionRow", () => {
  it("preserves generated recorded credit and exact history marker", () => {
    const row = coerceSessionRow(
      {
        start: "2026-05-16T09:00:00+02:00",
        end: "2026-05-16T09:03:00+02:00",
        state: "STANDING",
        duration_secs: 180,
        break_credit: "partial",
        history_kind: "individual",
      },
      7,
    );
    expect(row.id).toBe(7);
    expect(row.state).toBe("Standing");
    expect(row.durationSecs).toBe(180);
    expect(row.standingSecs).toBe(180);
    expect(row.sittingSecs).toBe(0);
    expect(row.breakCredit).toBe("partial");
    expect(row.historyKind).toBe("individual");
    expect(row.dateLocal).toBe("2026-05-16");
    expect(row.endedAt).toBe("2026-05-16T09:03:00+02:00");
  });

  it("keeps unknown state, duration, recorded credit and provenance unknown", () => {
    const row = coerceSessionRow(
      {
        start: "2026-05-16T09:00:00Z",
        end: null,
        state: "???",
        duration_secs: null,
        break_credit: null,
      },
      0,
    );
    expect(row.state).toBeNull();
    expect(row.durationSecs).toBeNull();
    expect(row.sittingSecs).toBeNull();
    expect(row.endedAt).toBeNull();
    expect(row.breakCredit).toBeNull();
    expect(row.historyKind).toBeNull();
    expect(row.positionChanges).toBeNull();
  });

  it("keeps legacy mixed posture duration and UTC local date unknown", () => {
    const row = coerceSessionRow(
      {
        start: "2026-05-16T23:30:00Z",
        end: "2026-05-17T00:30:00Z",
        state: "Standing",
        duration_secs: 3600,
        break_credit: "full",
      },
      1,
    );
    expect(row.durationSecs).toBe(3600);
    expect(row.breakCredit).toBe("full");
    expect(row.historyKind).toBeNull();
    expect(row.standingSecs).toBeNull();
    expect(row.sittingSecs).toBeNull();
    expect(row.dateLocal).toBeNull();
  });

  it("uses an explicit source offset for an individual session local date", () => {
    const row = coerceSessionRow(
      {
        start: "2026-05-16T23:30:00+02:00",
        end: "2026-05-17T00:00:00+02:00",
        state: "Standing",
        duration_secs: 1800,
        break_credit: "full",
        history_kind: "individual",
      },
      2,
    );
    expect(row.dateLocal).toBe("2026-05-16");
    expect(row.standingSecs).toBe(1800);
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

  it("uses the producer local date across a UTC midnight and omits unknown metrics", () => {
    const rows = [
      {
        ...snap("2026-10-22T22:30:00Z", 80, 75, 42, -12.5),
        localDate: "2026-10-23",
        sittingSecsTotal: 126,
        positionChanges: 9,
      },
      { ...snap("2026-10-22T22:31:00Z", 80, 80, 43), localDate: null },
    ];
    expect(deriveDailyKpis(rows)).toEqual([
      {
        dateLocal: "2026-10-23",
        standingPct: 25,
        positionChanges: 9,
        longestSessionSecs: null,
        score: -12.5,
      },
    ]);
  });

  it("uses two source-qualified runs' maximum, not the cumulative standing counter", () => {
    const dayWindow = {
      startMs: Date.parse("2026-05-15T09:00:00Z"),
      endMs: Date.parse("2026-05-15T09:25:00Z"),
    };
    const rows = [
      session("2026-05-15T09:00:00Z", "2026-05-15T09:10:00Z", "Standing"),
      session("2026-05-15T09:10:00Z", "2026-05-15T09:15:00Z", "Sitting"),
      session("2026-05-15T09:15:00Z", "2026-05-15T09:25:00Z", "Standing"),
    ];
    expect(deriveLongestContiguousStandingSecs(rows, dayWindow)).toBe(600);
    const day = deriveDailyKpis([snap("2026-05-15T09:25:00Z", 80, 300, 1200, 30)], rows,
      new Map([["2026-05-15", dayWindow]]));
    expect(day[0].longestSessionSecs).toBe(600);
    expect(day[0].standingPct).toBe(80);
  });

  it("includes a qualified cross-midnight Standing overlap in the explicitly supplied query-day window", () => {
    const nextDayWindow = {
      startMs: Date.parse("2026-05-16T00:00:00Z"),
      endMs: Date.parse("2026-05-16T00:10:00Z"),
    };
    const sessions = [
      session("2026-05-15T23:55:00Z", "2026-05-16T00:05:00Z", "Standing"),
      session("2026-05-16T00:05:00Z", "2026-05-16T00:10:00Z", "Sitting"),
    ];
    const snapshot = {
      ...snap("2026-05-16T00:10:00Z", 80, 900, 300, 30),
      localDate: "2026-05-16",
    };

    const day = deriveDailyKpis(
      [snapshot],
      sessions,
      new Map([["2026-05-16", nextDayWindow]]),
    );

    expect(day[0].longestSessionSecs).toBe(300);
    expect(day[0].standingPct).toBe(25);
  });

  it("distinguishes a proven zero from an Unknown day, legacy summary, or absent boundary", () => {
    const window = { startMs: Date.parse("2026-05-15T09:00:00Z"), endMs: Date.parse("2026-05-15T09:10:00Z") };
    expect(deriveLongestContiguousStandingSecs([
      session("2026-05-15T09:00:00Z", "2026-05-15T09:10:00Z", "Sitting"),
    ], window)).toBe(0);
    expect(deriveLongestContiguousStandingSecs([], window)).toBeNull();
    expect(deriveLongestContiguousStandingSecs([
      session("2026-05-15T09:00:00Z", "2026-05-15T09:10:00Z", "Sitting", null),
    ], window)).toBeNull();
    expect(deriveLongestContiguousStandingSecs([
      session("2026-05-15T09:00:00Z", "2026-05-15T09:03:00Z", "Standing"),
      session("2026-05-15T09:05:00Z", "2026-05-15T09:10:00Z", "Sitting"),
    ], window)).toBeNull();
    expect(deriveLongestContiguousStandingSecs([
      session("2026-05-15T09:00:00Z", "2026-05-15T09:10:00Z", "Standing"),
    ], null)).toBeNull();
  });
});
