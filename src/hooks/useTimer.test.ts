/**
 * useTimer.test.ts — unit tests for the useTimer hook.
 *
 * Verifies that the counter initialises correctly and increments over time.
 */
import { renderHook, act } from "@testing-library/react";
import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { useTimer } from "@/hooks/useTimer";

describe("useTimer", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("initialises with the provided starting value", () => {
    const { result } = renderHook(() => useTimer(42));
    expect(result.current).toBe(42);
  });

  it("increments by 1 after one second", () => {
    const { result } = renderHook(() => useTimer(0));
    act(() => {
      vi.advanceTimersByTime(1000);
    });
    expect(result.current).toBe(1);
  });

  it("increments correctly after multiple seconds", () => {
    const { result } = renderHook(() => useTimer(10));
    act(() => {
      vi.advanceTimersByTime(5000);
    });
    expect(result.current).toBe(15);
  });

  it("does not increment when running is false", () => {
    const { result } = renderHook(() => useTimer(5, false));
    act(() => {
      vi.advanceTimersByTime(3000);
    });
    expect(result.current).toBe(5);
  });

  it("resets to new initialSeconds when it changes", () => {
    let initial = 0;
    const { result, rerender } = renderHook(() => useTimer(initial));

    act(() => {
      vi.advanceTimersByTime(2000);
    });
    expect(result.current).toBe(2);

    // Simulate a backend event delivering a new base value
    initial = 100;
    rerender();

    expect(result.current).toBe(100);
  });
});
