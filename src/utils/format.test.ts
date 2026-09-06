/**
 * format.test.ts — covers the shared formatting helpers, including the six
 * moved here from component files in E018-T06.
 */
import { describe, it, expect } from "vitest";
import {
  colorSchemeFor,
  formatDate,
  formatDuration,
  formatDurationShort,
  formatIdleTime,
  formatRangeLabel,
  formatRefreshedAt,
  formatTime,
  weekdayName,
} from "./format";

describe("colorSchemeFor", () => {
  it("maps each state", () => {
    expect(colorSchemeFor("Sitting")).toBe("sitting");
    expect(colorSchemeFor("Standing")).toBe("standing");
    expect(colorSchemeFor("Away")).toBe("gray");
    expect(colorSchemeFor("Walking")).toBe("gray");
  });
});

describe("formatDuration", () => {
  it("formats with and without hours", () => {
    expect(formatDuration(3872)).toBe("1:04:32");
    expect(formatDuration(1927)).toBe("32:07");
  });

  it("clamps zero and negative input", () => {
    expect(formatDuration(0)).toBe("00:00");
    expect(formatDuration(-5)).toBe("00:00");
  });
});

describe("formatDurationShort", () => {
  it("picks the largest useful unit", () => {
    expect(formatDurationShort(8040)).toBe("2h14m");
    expect(formatDurationShort(2700)).toBe("45m");
    expect(formatDurationShort(32)).toBe("32s");
  });

  it("renders zero as seconds", () => {
    expect(formatDurationShort(0)).toBe("0s");
  });
});

describe("formatDate", () => {
  it("renders a day as DD Mon YYYY", () => {
    expect(formatDate("2026-09-06")).toBe("06 Sep 2026");
    expect(formatDate("2026-01-31")).toBe("31 Jan 2026");
  });

  it("does not throw on an invalid date string", () => {
    expect(() => formatDate("not-a-date")).not.toThrow();
  });
});

describe("weekdayName", () => {
  it("renders the full weekday", () => {
    expect(weekdayName("2026-09-06")).toBe("Sunday");
    expect(weekdayName("2026-09-07")).toBe("Monday");
  });

  it("does not throw on an invalid date string", () => {
    expect(() => weekdayName("")).not.toThrow();
  });
});

describe("formatRefreshedAt", () => {
  it("renders local HH:MM:SS with zero padding", () => {
    const epoch = new Date(2026, 8, 6, 9, 5, 7).getTime();
    expect(formatRefreshedAt(epoch)).toBe("09:05:07");
  });

  it("returns null for null and for 0", () => {
    expect(formatRefreshedAt(null)).toBeNull();
    expect(formatRefreshedAt(0)).toBeNull();
  });

  it("does not throw on NaN input", () => {
    expect(() => formatRefreshedAt(Number.NaN)).not.toThrow();
  });
});

describe("formatRangeLabel", () => {
  it("collapses a same-month range", () => {
    expect(formatRangeLabel("2026-05-10", "2026-05-17")).toBe("May 10–17");
  });

  it("spells out a cross-month range", () => {
    expect(formatRangeLabel("2026-05-28", "2026-06-03")).toBe("May 28–Jun 3");
  });

  it("falls back to an arrow for malformed input", () => {
    expect(formatRangeLabel("2026-05", "2026-06-03")).toBe("2026-05 → 2026-06-03");
    expect(formatRangeLabel("", "")).toBe(" → ");
  });

  it("renders an unknown month as ?", () => {
    expect(formatRangeLabel("2026-99-01", "2026-99-02")).toBe("? 1–2");
  });
});

describe("formatIdleTime", () => {
  it("renders minutes and seconds", () => {
    expect(formatIdleTime(95)).toBe("1m 35s");
    expect(formatIdleTime(30)).toBe("0m 30s");
  });

  it("renders zero", () => {
    expect(formatIdleTime(0)).toBe("0m 0s");
  });
});

describe("formatTime", () => {
  it("renders local HH:MM with zero padding", () => {
    const iso = new Date(2026, 8, 6, 8, 4).toISOString();
    expect(formatTime(iso)).toBe("08:04");
  });

  it("does not throw on an invalid ISO string", () => {
    expect(() => formatTime("nonsense")).not.toThrow();
    expect(() => formatTime("")).not.toThrow();
  });
});
