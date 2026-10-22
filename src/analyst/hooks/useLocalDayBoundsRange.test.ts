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
    expect(result.current.data).toEqual(rows);
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
