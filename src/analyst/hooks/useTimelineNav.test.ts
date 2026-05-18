/**
 * useTimelineNav.test.ts — selectedDay state + 02:00 flip behaviour.
 */
import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { liveSelectedDay, useTimelineNav } from "./useTimelineNav";

describe("liveSelectedDay", () => {
  it("before 02:00 returns yesterday", () => {
    const at = new Date("2026-05-17T01:30:00");
    expect(liveSelectedDay(at, 2)).toBe("2026-05-16");
  });
  it("at and after 02:00 returns today", () => {
    const at = new Date("2026-05-17T02:00:00");
    expect(liveSelectedDay(at, 2)).toBe("2026-05-17");
  });
});

describe("useTimelineNav", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it("clamps selectedDay to range", () => {
    const fixedNow = new Date("2026-05-17T12:00:00").getTime();
    const { result } = renderHook(() =>
      useTimelineNav({
        rangeFrom: "2026-05-10",
        rangeTo: "2026-05-17",
        nowMs: () => fixedNow,
      }),
    );
    expect(result.current.selectedDay).toBe("2026-05-17");
    expect(result.current.isLive).toBe(true);
  });

  it("next/prev exit live mode and clamp at edges", () => {
    const fixedNow = new Date("2026-05-17T12:00:00").getTime();
    const { result } = renderHook(() =>
      useTimelineNav({
        rangeFrom: "2026-05-15",
        rangeTo: "2026-05-17",
        nowMs: () => fixedNow,
      }),
    );
    act(() => result.current.prev());
    expect(result.current.selectedDay).toBe("2026-05-16");
    expect(result.current.isLive).toBe(false);
    act(() => result.current.next());
    act(() => result.current.next()); // already at end
    expect(result.current.selectedDay).toBe("2026-05-17");
  });

  it("goTo clamps and marks manual", () => {
    const fixedNow = new Date("2026-05-17T12:00:00").getTime();
    const { result } = renderHook(() =>
      useTimelineNav({
        rangeFrom: "2026-05-10",
        rangeTo: "2026-05-17",
        nowMs: () => fixedNow,
      }),
    );
    act(() => result.current.goTo("2026-05-12"));
    expect(result.current.selectedDay).toBe("2026-05-12");
    expect(result.current.isLive).toBe(false);
  });

  it("goLive resets manualSince and snaps back to today", () => {
    const fixedNow = new Date("2026-05-17T12:00:00").getTime();
    const { result } = renderHook(() =>
      useTimelineNav({
        rangeFrom: "2026-05-10",
        rangeTo: "2026-05-17",
        nowMs: () => fixedNow,
      }),
    );
    act(() => result.current.prev());
    expect(result.current.isLive).toBe(false);
    act(() => result.current.goLive());
    expect(result.current.selectedDay).toBe("2026-05-17");
    expect(result.current.isLive).toBe(true);
  });
});
