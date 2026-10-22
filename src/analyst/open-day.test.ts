import { describe, expect, it } from "vitest";
import {
  createOpenDayRequest,
  localCalendarDay,
  parseOpenDayRequest,
  rangeForOpenDay,
  shiftCalendarDay,
} from "./open-day";

describe("selected Analyst day requests", () => {
  it("formats and shifts calendar days without UTC-prefix assumptions", () => {
    expect(localCalendarDay(new Date(2026, 9, 22, 0, 15))).toBe("2026-10-22");
    expect(shiftCalendarDay("2026-03-29", -1)).toBe("2026-03-28");
    expect(shiftCalendarDay("2026-10-25", 1)).toBe("2026-10-26");
  });

  it("accepts a fresh well-formed local calendar-day request", () => {
    const request = createOpenDayRequest("2026-10-22", 10_000, "request-1");
    expect(parseOpenDayRequest(JSON.stringify(request), 11_000)).toEqual(request);
  });

  it.each([
    ["malformed JSON", "{"],
    ["impossible date", JSON.stringify({ day: "2026-02-30", requestedAtMs: 1000, requestId: "bad" })],
    ["missing request id", JSON.stringify({ day: "2026-10-22", requestedAtMs: 1000 })],
    ["stale request", JSON.stringify({ day: "2026-10-22", requestedAtMs: 1000, requestId: "old" })],
    ["future request", JSON.stringify({ day: "2026-10-22", requestedAtMs: 50_000, requestId: "future" })],
  ])("ignores %s payloads", (_label, raw) => {
    expect(parseOpenDayRequest(raw, raw.includes('"old"') ? 200_000 : 10_000)).toBeNull();
  });

  it("keeps a range containing the requested day and shifts its span when needed", () => {
    const range = { from: "2026-10-16", to: "2026-10-22" };
    expect(rangeForOpenDay(range, "2026-10-20")).toBe(range);
    expect(rangeForOpenDay(range, "2026-10-25")).toEqual({ from: "2026-10-19", to: "2026-10-25" });
    expect(rangeForOpenDay({ from: "bad", to: "bad" }, "2026-10-25")).toEqual({ from: "2026-10-25", to: "2026-10-25" });
  });
});
