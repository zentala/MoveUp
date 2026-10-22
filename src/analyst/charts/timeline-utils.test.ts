/**
 * timeline-utils.test.ts — unit tests for the Daily Timeline math helpers.
 */
import { describe, expect, it } from "vitest";
import type { SnapshotRow } from "@/test/analyst-fixtures";
import {
  aggregateDayTotals,
  analystTimelineIntervals,
  buildTimelineScaleForWindow,
  buildTimelineScale,
  dayCenterPx,
  dayOfMonthLabel,
  easeInOut,
  enumerateDays,
  isWeekend,
  localIsoDate,
  msToPx,
  scrollDurationMs,
  shiftDay,
  shortDayName,
} from "./timeline-utils";

function snap(ts: string, state: SnapshotRow["state"], localDate = ts.slice(0, 10)): SnapshotRow {
  return {
    ts,
    localDate,
    state,
    deskHeightCm: 75,
    sittingSecs: 0,
    standingSecs: 0,
    breakSecs: 0,
    idleSecs: 0,
    score: 50,
  };
}

describe("aggregateDayTotals", () => {
  it("does not turn point snapshots into elapsed posture totals", () => {
    const rows = [
      snap("2026-05-17T08:00:00.000Z", "Sitting"),
      snap("2026-05-17T08:01:00.000Z", "Sitting"),
      snap("2026-05-17T08:02:00.000Z", "Standing"),
      snap("2026-05-17T08:03:00.000Z", "Walking"),
      snap("2026-05-17T08:04:00.000Z", "Away"),
      snap("2026-05-18T08:00:00.000Z", "Sitting"),
    ];
    expect(aggregateDayTotals(rows)).toEqual(new Map());
  });

  it("does not derive duration from producer-local point samples or unknown rows", () => {
    const totals = aggregateDayTotals([
      snap("2026-10-22T22:30:00Z", null, "2026-10-23"),
      snap("2026-10-22T22:31:00Z", "Standing", "2026-10-23"),
      { ...snap("2026-10-23T00:00:00Z", "Sitting"), localDate: null },
      { ...snap("2026-10-23T00:01:00Z", "Away"), localDate: undefined },
    ]);

    expect(totals).toEqual(new Map());
  });
});

describe("analystTimelineIntervals", () => {
  it("uses only qualified source endpoints and ignores cumulative credited totals", () => {
    const rows = [
      { id: 1, startedAt: "2026-05-17T08:00:00Z", endedAt: "2026-05-17T08:10:00Z", state: "Standing" as const, durationSecs: 600, sittingSecs: 0, standingSecs: 600, positionChanges: null, breakCredit: null, historyKind: "individual", dateLocal: "2026-05-17" },
      { id: 2, startedAt: "2026-05-17T08:10:00Z", endedAt: "2026-05-17T08:20:00Z", state: "Standing" as const, durationSecs: 600, sittingSecs: 0, standingSecs: null, positionChanges: null, breakCredit: "full" as const, historyKind: null, dateLocal: "2026-05-17" },
    ];
    expect(analystTimelineIntervals(rows).map(({ startMs, endMs, state, provenance }) => [
      startMs, endMs, state, provenance,
    ])).toEqual([[Date.parse(rows[0].startedAt), Date.parse(rows[0].endedAt!), "Standing", "session"]]);
  });
});

describe("enumerateDays", () => {
  it("returns inclusive sequential days", () => {
    expect(enumerateDays("2026-05-15", "2026-05-17")).toEqual([
      "2026-05-15",
      "2026-05-16",
      "2026-05-17",
    ]);
  });

  it("returns single element when from == to", () => {
    expect(enumerateDays("2026-05-17", "2026-05-17")).toEqual(["2026-05-17"]);
  });

  it("returns empty for invalid input", () => {
    expect(enumerateDays("nope", "also-bad")).toEqual([]);
  });
});

describe("isWeekend / shortDayName / dayOfMonthLabel", () => {
  it("flags Saturday and Sunday", () => {
    // 2026-05-16 is a Saturday, 2026-05-17 Sunday.
    expect(isWeekend("2026-05-16")).toBe(true);
    expect(isWeekend("2026-05-17")).toBe(true);
    expect(isWeekend("2026-05-18")).toBe(false);
  });
  it("renders short labels", () => {
    expect(shortDayName("2026-05-18")).toBe("Mon");
    expect(dayOfMonthLabel("2026-05-04")).toBe("04");
  });
});

describe("buildTimelineScale + msToPx + dayCenterPx", () => {
  it("uniform pixel scale across the padded range", () => {
    const scale = buildTimelineScale("2026-05-17", "2026-05-17", 6, 1440);
    // Range = 24h + 12h padding = 36h = 2160 minutes. widthPx 1440 → pxPerMin = 2/3.
    expect(scale.pxPerMinute).toBeCloseTo(1440 / 2160, 6);
    // Center of May 17 is 12:00 = 6h padding + 12h = 18h from start → 18/36 = 50%.
    const cx = dayCenterPx("2026-05-17", scale);
    expect(cx).toBeCloseTo(720, 0);
  });

  it("msToPx maps the start to 0", () => {
    const scale = buildTimelineScale("2026-05-17", "2026-05-17", 0, 1440);
    expect(msToPx(scale.startMs, scale)).toBe(0);
    expect(msToPx(scale.endMs, scale)).toBeCloseTo(1440, 0);
  });
});

describe("easeInOut", () => {
  it("clamps endpoints", () => {
    expect(easeInOut(0)).toBe(0);
    expect(easeInOut(1)).toBe(1);
    expect(easeInOut(-1)).toBe(0);
    expect(easeInOut(2)).toBe(1);
  });
  it("passes through 0.5 with itself", () => {
    expect(easeInOut(0.5)).toBeCloseTo(0.5, 6);
  });
});

describe("scrollDurationMs", () => {
  it("clamps to [240, 900]", () => {
    expect(scrollDurationMs(0)).toBe(240);
    expect(scrollDurationMs(100)).toBe(300);
    expect(scrollDurationMs(10_000)).toBe(900);
  });
  it("ignores sign", () => {
    expect(scrollDurationMs(-500)).toBe(scrollDurationMs(500));
  });
});

describe("buildTimelineScaleForWindow", () => {
  it("maps actual source bounds without consulting the browser timezone", () => {
    const scale = buildTimelineScaleForWindow({ startMs: 10_000, endMs: 1_810_000 }, 180);
    expect(scale.startMs).toBe(10_000);
    expect(scale.endMs).toBe(1_810_000);
    expect(scale.pxPerMinute).toBe(6);
    expect(() => buildTimelineScaleForWindow({ startMs: 2, endMs: 2 }, 100)).toThrow(/source window/);
  });
});

describe("shiftDay / localIsoDate", () => {
  it("shifts by positive and negative offsets", () => {
    expect(shiftDay("2026-05-17", 1)).toBe("2026-05-18");
    expect(shiftDay("2026-05-17", -1)).toBe("2026-05-16");
    expect(shiftDay("2026-05-01", -1)).toBe("2026-04-30");
  });
  it("localIsoDate formats local components", () => {
    const d = new Date(2026, 4, 17); // May (0-indexed)
    expect(localIsoDate(d)).toBe("2026-05-17");
  });
});
