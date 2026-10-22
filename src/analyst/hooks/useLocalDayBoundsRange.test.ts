import { act, renderHook } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { LocalDayBounds } from "@/generated/LocalDayBounds";
import { useLocalDayBoundsRange } from "./useLocalDayBoundsRange";

const row = (date: string, start: string, end: string): LocalDayBounds => ({
  date_local: date,
  start_utc: start,
  end_utc: end,
  zone_basis: "current-query-local",
  historical_source_zone: "unknown",
});

beforeEach(() => {
  vi.useFakeTimers();
  vi.mocked(invoke).mockReset();
});

afterEach(() => vi.useRealTimers());

describe("useLocalDayBoundsRange", () => {
  it("loads every returned calendar date, including empty dates", async () => {
    const rows = [
      row("2026-03-07", "2026-03-07T05:00:00Z", "2026-03-08T05:00:00Z"),
      row("2026-03-08", "2026-03-08T05:00:00Z", "2026-03-09T04:00:00Z"),
      row("2026-03-09", "2026-03-09T04:00:00Z", "2026-03-10T04:00:00Z"),
    ];
    vi.mocked(invoke).mockResolvedValueOnce(rows);
    const { result } = renderHook(() => useLocalDayBoundsRange("2026-03-07", "2026-03-09"));
    expect(result.current.status).toBe("loading");
    await act(async () => vi.advanceTimersByTimeAsync(250));
    expect(result.current.status).toBe("ready");
    if (result.current.status !== "ready") throw new Error("expected ready");
    expect(result.current.data).toEqual(rows.map((item) => ({ ...item, hour_ticks: null })));
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("get_local_day_bounds_range", {
      from: "2026-03-07", to: "2026-03-09",
    });
  });

  it("returns error rather than accepting missing or noncontiguous dates", async () => {
    vi.mocked(invoke).mockResolvedValueOnce([
      row("2026-05-16", "2026-05-16T00:00:00Z", "2026-05-17T00:00:00Z"),
      row("2026-05-18", "2026-05-18T00:00:00Z", "2026-05-19T00:00:00Z"),
    ]);
    const { result } = renderHook(() => useLocalDayBoundsRange("2026-05-16", "2026-05-17"));
    await act(async () => vi.advanceTimersByTimeAsync(250));
    expect(result.current.status).toBe("error");
    if (result.current.status !== "error") throw new Error("expected error");
    expect(result.current.error).toBe("Invalid local-day bounds response");
  });

  it.each([
    undefined,
    null,
    [{ at_utc: "bad", wall_time: "25:00", utc_offset_secs: "zero" }],
    [{ at_utc: "2026-05-16T00:00:00Z", wall_time: "01:00", utc_offset_secs: 0 }],
    [{ at_utc: "2026-05-16T09:00:59Z", wall_time: "09:00", utc_offset_secs: 0 }],
    [{ at_utc: "2026-05-16T09:00:00.500Z", wall_time: "09:00", utc_offset_secs: 0 }],
    [
      { at_utc: "2026-05-16T01:00:00Z", wall_time: "01:00", utc_offset_secs: 0 },
      { at_utc: "2026-05-16T00:00:00Z", wall_time: "00:00", utc_offset_secs: 0 },
    ],
    [{ at_utc: "2026-05-17T00:00:00Z", wall_time: "00:00", utc_offset_secs: 0 }],
    [{ at_utc: "2026-05-16T23:00:00Z", wall_time: "01:00", utc_offset_secs: 7_200 }],
    [],
  ])(
    "keeps valid bounds but marks an absent or invalid local-hour map unavailable (%j)", async (hourTicks) => {
      const payload = [{ ...row("2026-05-16", "2026-05-16T00:00:00Z", "2026-05-17T00:00:00Z"), hour_ticks: hourTicks }];
      vi.mocked(invoke).mockResolvedValueOnce(payload as unknown as LocalDayBounds[]);
      const { result } = renderHook(() => useLocalDayBoundsRange("2026-05-16", "2026-05-16"));
      await act(async () => vi.advanceTimersByTimeAsync(250));
      expect(result.current.status).toBe("ready");
      if (result.current.status !== "ready") throw new Error("expected valid calendar bounds");
      expect(result.current.data[0].start_utc).toBe("2026-05-16T00:00:00Z");
      expect(result.current.data[0].hour_ticks).toBeNull();
    },
  );

  it("preserves a valid exact hourly map, including offsets", async () => {
    const ticks = [
      { at_utc: "2026-05-16T00:00:00Z", wall_time: "02:00", utc_offset_secs: 7200 },
      { at_utc: "2026-05-16T01:00:00Z", wall_time: "03:00", utc_offset_secs: 7200 },
    ];
    vi.mocked(invoke).mockResolvedValueOnce([
      { ...row("2026-05-16", "2026-05-16T00:00:00Z", "2026-05-17T00:00:00Z"), hour_ticks: ticks },
    ] as unknown as LocalDayBounds[]);
    const { result } = renderHook(() => useLocalDayBoundsRange("2026-05-16", "2026-05-16"));
    await act(async () => vi.advanceTimersByTimeAsync(250));
    if (result.current.status !== "ready") throw new Error("expected ready");
    expect(result.current.data[0].hour_ticks).toEqual(ticks);
  });

  it.each([null, {}, [null]])("rejects malformed IPC payload %j as an error", async (payload) => {
    vi.mocked(invoke).mockResolvedValueOnce(payload as LocalDayBounds[]);
    const { result } = renderHook(() => useLocalDayBoundsRange("2026-05-16", "2026-05-16"));
    await act(async () => vi.advanceTimersByTimeAsync(250));
    expect(result.current.status).toBe("error");
    if (result.current.status !== "error") throw new Error("expected error");
    expect(result.current.error).toBe("Invalid local-day bounds response");
  });

  it("keeps failures as error and ignores a late response from an old range", async () => {
    let resolveOld!: (rows: LocalDayBounds[]) => void;
    const oldResponse = new Promise<LocalDayBounds[]>((resolve) => { resolveOld = resolve; });
    vi.mocked(invoke)
      .mockImplementationOnce(() => oldResponse as ReturnType<typeof invoke>)
      .mockRejectedValueOnce(new Error("calendar unavailable"));
    const { result, rerender } = renderHook(
      ({ from, to }: { from: string; to: string }) => useLocalDayBoundsRange(from, to),
      { initialProps: { from: "2026-05-16", to: "2026-05-16" } },
    );
    await act(async () => vi.advanceTimersByTimeAsync(250));
    rerender({ from: "2026-05-17", to: "2026-05-17" });
    await act(async () => vi.advanceTimersByTimeAsync(250));
    expect(result.current.status).toBe("error");
    await act(async () => { resolveOld([row("2026-05-16", "2026-05-16T00:00:00Z", "2026-05-17T00:00:00Z")]); });
    expect(result.current.status).toBe("error");
    if (result.current.status !== "error") throw new Error("expected error");
    expect(result.current.error).toBe("calendar unavailable");
  });
});
