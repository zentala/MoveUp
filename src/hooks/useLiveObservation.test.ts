import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { DashboardState, SessionStateDto } from "@/types";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

import { useLiveObservation } from "./useLiveObservation";

function makeSession(overrides: Partial<SessionStateDto> = {}): SessionStateDto {
  return {
    state: "Sitting", sitting_seconds: 120, standing_seconds: 0, break_seconds: 0,
    session_limit_secs: 2400, stand_limit_secs: 900, desk_height_cm: 72,
    position_changes: 0, limit_used_secs: 120, daily_score: 0,
    standing_session_secs: 0, secs_since_last_break: 120,
    continuous_computer_secs: 120, longest_computer_session_secs: 120,
    sitting_seconds_total: 120, idle_secs: 0, away_bout_secs: 0,
    max_continuous_computer_secs: 7200,
    ...overrides,
  };
}

function dashboard(overrides: Partial<SessionStateDto> = {}): DashboardState {
  return { session: makeSession(overrides), metrics: [] };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => { resolve = done; });
  return { promise, resolve };
}

beforeEach(() => {
  delete window.__MOVEUP_UI_DIAGNOSTICS__;
  invokeMock.mockReset();
  vi.useRealTimers();
});

afterEach(() => { vi.useRealTimers(); });

describe("useLiveObservation", () => {
  it("publishes only accepted dashboard completions and retains success while a later read fails", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(1_000);
    const pending = deferred<DashboardState>();
    invokeMock.mockReturnValueOnce(pending.promise).mockRejectedValueOnce(new Error("PRIVATE query"));
    const { startUiDiagnostics } = await import("../uiDiagnostics");
    const stop = startUiDiagnostics();
    const { unmount } = renderHook(() => useLiveObservation());
    expect(window.__MOVEUP_UI_DIAGNOSTICS__?.queries.dashboard).toEqual({
      outcome: "not_observed", last_result_ms: null, last_success_ms: null,
    });
    vi.setSystemTime(2_000);
    await act(async () => { pending.resolve(dashboard()); });
    expect(window.__MOVEUP_UI_DIAGNOSTICS__?.queries.dashboard).toEqual({
      outcome: "succeeded", last_result_ms: 2_000, last_success_ms: 2_000,
    });
    await act(async () => { await vi.advanceTimersByTimeAsync(1_000); });
    expect(window.__MOVEUP_UI_DIAGNOSTICS__?.queries).toEqual({
      dashboard: { outcome: "failed", last_result_ms: 3_000, last_success_ms: 2_000 },
      today_summary: { outcome: "not_observed", last_result_ms: null, last_success_ms: null },
    });
    expect(invokeMock.mock.calls.map(([command]) => command)).toEqual(["get_dashboard_state", "get_dashboard_state"]);
    expect(JSON.stringify(window.__MOVEUP_UI_DIAGNOSTICS__)).not.toContain("PRIVATE");
    unmount();
    stop();
  });

  it.each(["success", "failure"])("ignores a %s received after unmount", async (outcome) => {
    let resolve!: (value: DashboardState) => void;
    let reject!: (error: Error) => void;
    invokeMock.mockReturnValue(new Promise<DashboardState>((done, fail) => { resolve = done; reject = fail; }));
    const { startUiDiagnostics } = await import("../uiDiagnostics");
    const stop = startUiDiagnostics();
    const { unmount } = renderHook(() => useLiveObservation());
    const before = structuredClone(window.__MOVEUP_UI_DIAGNOSTICS__);
    unmount();
    await act(async () => {
      if (outcome === "success") resolve(dashboard());
      else reject(new Error("PRIVATE late result"));
    });
    expect(window.__MOVEUP_UI_DIAGNOSTICS__).toEqual(before);
    stop();
  });
  it("reads only the existing dashboard snapshot and reports its producer and receipt facts", async () => {
    invokeMock.mockResolvedValue(dashboard({
      observation_quality: null,
      live_observation: {
        span: {
          state: "Sitting",
          started_at: "2026-10-25T08:00:00Z",
          observed_through: "2026-10-25T08:04:00Z",
        },
        last_observation_at: "2026-10-25T08:04:00Z",
        snapshot_at: "2026-10-25T08:09:01Z",
        quality: null,
        reason: "stale",
      },
    }));
    const { result, unmount } = renderHook(() => useLiveObservation());

    await act(async () => { await Promise.resolve(); await Promise.resolve(); });

    expect(invokeMock).toHaveBeenCalledTimes(1);
    expect(invokeMock).toHaveBeenCalledWith("get_dashboard_state");
    expect(invokeMock).not.toHaveBeenCalledWith("start_auto_connect", undefined);
    expect(result.current.observationQuality).toBeNull();
    expect(result.current.liveObservation?.reason).toBe("stale");
    expect(result.current.refresh.lastSuccessAt).toEqual(expect.any(String));
    unmount();
  });

  it("retains the last source snapshot and receipt while exposing a failed refresh", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-25T08:09:02Z"));
    invokeMock
      .mockResolvedValueOnce(dashboard({
        observation_quality: "known",
        live_observation: {
          span: {
            state: "Sitting",
            started_at: "2026-10-25T08:00:00Z",
            observed_through: "2026-10-25T08:09:00Z",
          },
          last_observation_at: "2026-10-25T08:09:00Z",
          snapshot_at: "2026-10-25T08:09:01Z",
          quality: "known",
          reason: null,
        },
      }))
      .mockRejectedValueOnce(new Error("snapshot offline"));
    const { result, unmount } = renderHook(() => useLiveObservation());

    await act(async () => { await Promise.resolve(); await Promise.resolve(); });
    const lastSuccessAt = result.current.refresh.lastSuccessAt;
    await act(async () => { await vi.advanceTimersByTimeAsync(1_000); });

    expect(result.current.liveObservation?.span?.observed_through).toBe("2026-10-25T08:09:00Z");
    expect(result.current.observationQuality).toBe("known");
    expect(result.current.refresh).toEqual({
      loading: false,
      lastSuccessAt,
      error: "snapshot offline",
    });
    unmount();
    vi.useRealTimers();
  });

  it("accepts a slow snapshot response and coalesces poll ticks while it is in flight", async () => {
    vi.useFakeTimers();
    const first = deferred<DashboardState>();
    const second = deferred<DashboardState>();
    let calls = 0;
    invokeMock.mockImplementation(() => {
      calls += 1;
      return calls === 1 ? first.promise : second.promise;
    });
    const { result, unmount } = renderHook(() => useLiveObservation());

    await act(async () => { await Promise.resolve(); });
    await act(async () => { await vi.advanceTimersByTimeAsync(2_500); });
    expect(calls).toBe(1);

    await act(async () => {
      first.resolve(dashboard({
        observation_quality: "known",
        live_observation: {
          span: {
            state: "Sitting",
            started_at: "2026-10-25T08:00:00Z",
            observed_through: "2026-10-25T08:02:30Z",
          },
          last_observation_at: "2026-10-25T08:02:30Z",
          snapshot_at: "2026-10-25T08:02:31Z",
          quality: "known",
          reason: null,
        },
      }));
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(calls).toBe(2);
    expect(result.current.liveObservation?.span?.observed_through).toBe("2026-10-25T08:02:30Z");

    await act(async () => {
      second.resolve(dashboard({
        observation_quality: "known",
        live_observation: {
          span: {
            state: "Standing",
            started_at: "2026-10-25T08:02:31Z",
            observed_through: "2026-10-25T08:03:00Z",
          },
          last_observation_at: "2026-10-25T08:03:00Z",
          snapshot_at: "2026-10-25T08:03:01Z",
          quality: "known",
          reason: null,
        },
      }));
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(result.current.liveObservation?.span?.state).toBe("Standing");
    unmount();
    vi.useRealTimers();
  });
});
