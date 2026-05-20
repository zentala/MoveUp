/**
 * timeline-utils.test.ts — unit tests for the Daily Timeline math helpers.
 */
import { describe, expect, it } from "vitest";
import type { SnapshotRow } from "@/test/analyst-fixtures";
import {
  aggregateDayTotals,
  buildTimelineScale,
  dayCenterPx,
  dayOfMonthLabel,
  easeInOut,
  enumerateDays,
  inferSnapshotInterval,
  isWeekend,
  localIsoDate,
  msToPx,
  scrollDurationMs,
  shiftDay,
  shortDayName,
} from "./timeline-utils";

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

describe("aggregateDayTotals", () => {
  it("buckets snapshots by local date and counts per-state minutes", () => {
    const rows = [
      snap("2026-05-17T08:00:00.000Z", "Sitting"),
      snap("2026-05-17T08:01:00.000Z", "Sitting"),
      snap("2026-05-17T08:02:00.000Z", "Standing"),
      snap("2026-05-17T08:03:00.000Z", "Walking"),
      snap("2026-05-17T08:04:00.000Z", "Away"),
      snap("2026-05-18T08:00:00.000Z", "Sitting"),
    ];
    const totals = aggregateDayTotals(rows);
    const may17 = totals.get("2026-05-17");
    expect(may17).toBeDefined();
    expect(may17?.sit).toBe(120);
    expect(may17?.stand).toBe(60);
    expect(may17?.walk).toBe(60);
    expect(may17?.away).toBe(60);
    expect(may17?.total).toBe(300);
    expect(totals.get("2026-05-18")?.sit).toBe(60);
  });

  it("respects custom secsPerSnapshot", () => {
    const totals = aggregateDayTotals([snap("2026-05-17T00:00:00Z", "Sitting")], 30);
    expect(totals.get("2026-05-17")?.sit).toBe(30);
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

describe("inferSnapshotInterval", () => {
  it("returns 60s default for empty / single row", () => {
    expect(inferSnapshotInterval([])).toBe(60);
    expect(inferSnapshotInterval([snap("2026-05-17T08:00:00Z", "Sitting")])).toBe(60);
  });
  it("detects 20-min regular spacing", () => {
    const rows: SnapshotRow[] = [];
    const start = new Date("2026-05-17T08:00:00Z").getTime();
    for (let i = 0; i < 10; i++) {
      rows.push(snap(new Date(start + i * 20 * 60_000).toISOString(), "Sitting"));
    }
    expect(inferSnapshotInterval(rows)).toBe(1200);
  });
  it("uses median to ignore outliers", () => {
    const rows = [
      snap("2026-05-17T08:00:00Z", "Sitting"),
      snap("2026-05-17T08:05:00Z", "Sitting"), // 5m
      snap("2026-05-17T08:10:00Z", "Sitting"), // 5m
      snap("2026-05-17T08:15:00Z", "Sitting"), // 5m
      // 30m gap counted (still <1h cap)
      snap("2026-05-17T08:45:00Z", "Sitting"),
    ];
    expect(inferSnapshotInterval(rows)).toBe(300);
  });
  it("caps at 1h and floors at 60s", () => {
    const tooBig = [
      snap("2026-05-17T08:00:00Z", "Sitting"),
      snap("2026-05-17T12:00:00Z", "Sitting"), // 4h gap filtered (≥3600 ignored)
    ];
    expect(inferSnapshotInterval(tooBig)).toBe(60); // no gaps in range → fallback
    const tooSmall = [
      snap("2026-05-17T08:00:00Z", "Sitting"),
      snap("2026-05-17T08:00:30Z", "Sitting"),
    ];
    expect(inferSnapshotInterval(tooSmall)).toBe(60); // clamped up
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
