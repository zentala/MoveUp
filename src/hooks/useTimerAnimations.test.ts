/**
 * useTimerAnimations.test.ts — unit tests for the useTimerAnimations hook.
 */
import { renderHook, act } from "@testing-library/react";
import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { useTimerAnimations } from "@/hooks/useTimerAnimations";

describe("useTimerAnimations", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("initial state: both false", () => {
    const { result } = renderHook(() => useTimerAnimations("Sitting", 0.5));
    expect(result.current.fade).toBe(false);
    expect(result.current.shimmer).toBe(false);
  });

  it("fade triggers on state change", () => {
    const state = "Sitting" as const;
    const { result, rerender } = renderHook(
      ({ s }) => useTimerAnimations(s, 0.5),
      { initialProps: { s: state as "Sitting" | "Standing" | "Walking" | "Away" } },
    );

    rerender({ s: "Standing" });
    expect(result.current.fade).toBe(true);
  });

  it("fade clears after timeout", () => {
    const { result, rerender } = renderHook(
      ({ s }) => useTimerAnimations(s, 0.5),
      { initialProps: { s: "Sitting" as "Sitting" | "Standing" | "Walking" | "Away" } },
    );

    rerender({ s: "Standing" });
    expect(result.current.fade).toBe(true);

    act(() => {
      vi.advanceTimersByTime(300);
    });
    expect(result.current.fade).toBe(false);
  });

  it("shimmer triggers when progress crosses 1.0 downward", () => {
    const { result, rerender } = renderHook(
      ({ p }) => useTimerAnimations("Sitting", p),
      { initialProps: { p: 1.0 } },
    );

    rerender({ p: 0.8 });
    expect(result.current.shimmer).toBe(true);
  });

  it("shimmer does not trigger on normal progress decrease", () => {
    const { result, rerender } = renderHook(
      ({ p }) => useTimerAnimations("Sitting", p),
      { initialProps: { p: 0.7 } },
    );

    rerender({ p: 0.5 });
    expect(result.current.shimmer).toBe(false);
  });

  it("no shimmer when progress goes up", () => {
    const { result, rerender } = renderHook(
      ({ p }) => useTimerAnimations("Sitting", p),
      { initialProps: { p: 0.5 } },
    );

    rerender({ p: 1.0 });
    expect(result.current.shimmer).toBe(false);
  });
});
