/**
 * useRangeQuery.test.ts — Unit tests for the generic range-query hook.
 *
 * Verifies:
 * - debounce: invoke does not fire before `debounceMs` elapses
 * - loading → ready transition with returned data
 * - error path on rejected invoke
 * - refetch when `(from, to)` changes
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { useRangeQuery } from "./useRangeQuery";

beforeEach(() => {
  vi.useFakeTimers();
  vi.mocked(invoke).mockReset();
});

afterEach(() => {
  vi.useRealTimers();
});

describe("useRangeQuery", () => {
  it("debounces invoke and resolves to ready", async () => {
    vi.mocked(invoke).mockResolvedValueOnce([{ ts: "a" }]);
    const { result } = renderHook(() =>
      useRangeQuery<{ ts: string }>({
        command: "get_x",
        from: "2026-05-10",
        to: "2026-05-16",
        debounceMs: 250,
      }),
    );

    expect(result.current.status).toBe("loading");
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();

    await act(async () => {
      await vi.advanceTimersByTimeAsync(250);
    });
    expect(result.current.status).toBe("ready");
    if (result.current.status !== "ready") throw new Error("expected ready");
    expect(result.current.data).toEqual([{ ts: "a" }]);
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("get_x", {
      from: "2026-05-10",
      to: "2026-05-16",
    });
  });

  it("transitions to error when invoke rejects", async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error("io failure"));
    const { result } = renderHook(() =>
      useRangeQuery<unknown>({
        command: "get_x",
        from: "2026-05-10",
        to: "2026-05-16",
        debounceMs: 0,
      }),
    );

    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(result.current.status).toBe("error");
    if (result.current.status !== "error") throw new Error("expected error");
    expect(result.current.error).toBe("io failure");
  });

  it("refetches when range changes", async () => {
    vi.mocked(invoke).mockResolvedValue([]);
    const { rerender } = renderHook(
      ({ from, to }: { from: string; to: string }) =>
        useRangeQuery<unknown>({ command: "get_x", from, to, debounceMs: 0 }),
      { initialProps: { from: "2026-05-10", to: "2026-05-16" } },
    );
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("get_x", {
      from: "2026-05-10",
      to: "2026-05-16",
    });
    vi.mocked(invoke).mockClear();
    rerender({ from: "2026-05-09", to: "2026-05-15" });
    await act(async () => {
      await vi.advanceTimersByTimeAsync(0);
    });
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("get_x", {
      from: "2026-05-09",
      to: "2026-05-15",
    });
  });

  it("skip=true never invokes", async () => {
    const { result } = renderHook(() =>
      useRangeQuery<unknown>({
        command: "get_x",
        from: "2026-05-10",
        to: "2026-05-16",
        debounceMs: 0,
        skip: true,
      }),
    );
    await act(async () => {
      await vi.advanceTimersByTimeAsync(1000);
    });
    expect(result.current.status).toBe("loading");
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
  });
});
