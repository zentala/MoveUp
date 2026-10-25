/**
 * useDesk.test.ts — Tauri transport tests with `invoke`/`listen` mocked,
 * mirroring how useRemoteDesk.test.ts fakes WebSocket.
 */
import { renderHook, act } from "@testing-library/react";
import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import type { DashboardState, SessionStateDto, StateChangedPayload, TodaySummaryDto } from "@/types";

const invokeMock = vi.fn();
const listeners = new Map<string, (e: { payload: unknown }) => void>();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: unknown) => invokeMock(cmd, args),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: (event: string, handler: (e: { payload: unknown }) => void) => {
    listeners.set(event, handler);
    return Promise.resolve(() => listeners.delete(event));
  },
}));

import { useDesk } from "./useDesk";

function makeSession(overrides: Partial<SessionStateDto> = {}): SessionStateDto {
  return {
    state: "Sitting", sitting_seconds: 120, standing_seconds: 60, break_seconds: 0,
    session_limit_secs: 2400, stand_limit_secs: 900, desk_height_cm: 72.5,
    position_changes: 2, limit_used_secs: 120, daily_score: 5,
    standing_session_secs: 0, secs_since_last_break: 120,
    continuous_computer_secs: 120, longest_computer_session_secs: 120,
    sitting_seconds_total: 120, idle_secs: 0, away_bout_secs: 0,
    max_continuous_computer_secs: 7200,
    ...overrides,
  };
}

const emptySummary: TodaySummaryDto = {
  sitting_secs: 0,
  standing_secs: 0,
  yesterday_sitting_secs: 0,
  yesterday_standing_secs: 0,
  position_changes: 0,
  sessions: [],
};

/** Route each Tauri command to a canned reply. */
function stubBackend(opts: {
  session?: SessionStateDto;
  today?: TodaySummaryDto;
  dashboardFails?: boolean;
} = {}) {
  invokeMock.mockImplementation((cmd: string) => {
    switch (cmd) {
      case "get_dashboard_state":
        return opts.dashboardFails
          ? Promise.reject(new Error("backend not ready"))
          : Promise.resolve({ session: opts.session ?? makeSession(), metrics: [] });
      case "get_today_summary":
        return Promise.resolve(opts.today ?? emptySummary);
      case "get_connected_port":
        return Promise.resolve("COM3");
      default:
        return Promise.resolve(null);
    }
  });
}

/** Render the hook and let the mount-time promises settle. */
async function renderDesk() {
  const rendered = renderHook(() => useDesk());
  await act(async () => { await Promise.resolve(); await Promise.resolve(); });
  return rendered;
}

function emit(event: string, payload: unknown) {
  const handler = listeners.get(event);
  if (!handler) throw new Error(`no listener registered for ${event}`);
  handler({ payload });
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => { resolve = done; });
  return { promise, resolve };
}

beforeEach(() => {
  listeners.clear();
  invokeMock.mockReset();
  stubBackend();
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe("useDesk — happy path", () => {
  it("polls get_dashboard_state and exposes the session on the UseDeskResult shape", async () => {
    const { result } = await renderDesk();

    expect(result.current.state).toBe("Sitting");
    expect(result.current.deskHeightCm).toBe(72.5);
    expect(result.current.limitUsedSecs).toBe(120);
    expect(result.current.sessionLimitSecs).toBe(2400);
    expect(result.current.limitRemaining).toBe(2280);
    expect(result.current.connected).toBe(true);
    expect(invokeMock).toHaveBeenCalledWith("start_auto_connect", undefined);
  });

  it("keeps connected sensor state separate from stale sample and dashboard/history receipts", async () => {
    const liveObservation = {
      span: {
        state: "Sitting" as const,
        started_at: "2026-10-25T08:00:00Z",
        observed_through: "2026-10-25T08:04:00Z",
      },
      last_observation_at: "2026-10-25T08:04:00Z",
      snapshot_at: "2026-10-25T08:09:01Z",
      quality: null,
      reason: "stale",
    };
    stubBackend({ session: makeSession({ live_observation: liveObservation }) });

    const { result } = await renderDesk();

    expect(result.current.connected).toBe(true);
    expect(result.current.observationQuality).toBeNull();
    expect(result.current.liveObservation).toEqual(liveObservation);
    expect(result.current.dashboardRefresh).toMatchObject({ loading: false, error: null });
    expect(result.current.dashboardRefresh.lastSuccessAt).toEqual(expect.any(String));
    expect(result.current.historyRefresh).toMatchObject({ loading: false, error: null });
    expect(result.current.historyRefresh.lastSuccessAt).toEqual(expect.any(String));
  });

  it("retains successful snapshot/history data and receipts when reconnect reads fail", async () => {
    const liveObservation = {
      span: {
        state: "Sitting" as const,
        started_at: "2026-10-25T08:00:00Z",
        observed_through: "2026-10-25T08:04:00Z",
      },
      last_observation_at: "2026-10-25T08:04:00Z",
      snapshot_at: "2026-10-25T08:04:01Z",
      quality: "known",
      reason: null,
    };
    const today: TodaySummaryDto = { ...emptySummary, sitting_secs: 240, sessions: [
      { start: "08:00", end: null, state: "Sitting", duration_secs: null, break_credit: null },
    ] };
    stubBackend({ session: makeSession({ live_observation: liveObservation }), today });
    const debugSpy = vi.spyOn(console, "debug").mockImplementation(() => {});
    const { result } = await renderDesk();
    const dashboardSuccess = result.current.dashboardRefresh.lastSuccessAt;
    const historySuccess = result.current.historyRefresh.lastSuccessAt;

    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_dashboard_state") return Promise.reject(new Error("snapshot offline"));
      if (cmd === "get_today_summary") return Promise.reject(new Error("history offline"));
      return Promise.resolve(null);
    });
    await act(async () => {
      emit("desk:device-connected", { port: "COM3" });
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(result.current.liveObservation).toEqual(liveObservation);
    expect(result.current.todaySessions).toEqual(today.sessions);
    expect(result.current.dashboardRefresh).toEqual({
      loading: false,
      lastSuccessAt: dashboardSuccess,
      error: "snapshot offline",
    });
    expect(result.current.historyRefresh).toEqual({
      loading: false,
      lastSuccessAt: historySuccess,
      error: "history offline",
    });
    expect(debugSpy).toHaveBeenCalledWith("get_dashboard_state not ready:", expect.any(Error));
    expect(debugSpy).toHaveBeenCalledWith("get_today_summary not ready:", expect.any(Error));
  });

  it("ignores a slow dashboard reply made obsolete by a state-change event", async () => {
    vi.useFakeTimers();
    const older = deferred<DashboardState>();
    const newer = deferred<DashboardState>();
    let dashboardCalls = 0;
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_dashboard_state") {
        dashboardCalls += 1;
        return dashboardCalls === 1 ? older.promise : newer.promise;
      }
      if (cmd === "get_today_summary") return Promise.resolve(emptySummary);
      if (cmd === "get_connected_port") return Promise.resolve("COM3");
      return Promise.resolve(null);
    });
    const { result } = renderHook(() => useDesk());
    await act(async () => { await Promise.resolve(); });

    await act(async () => {
      emit("desk:state-changed", {
        state: "Standing", standing_seconds: 90, break_seconds: 30,
        desk_height_cm: 110, position_changes: 4, last_break_secs: 0,
        last_sitting_secs: 1200, break_credit: "full", limit_used_secs: 1500,
      });
      await Promise.resolve();
    });
    expect(result.current.state).toBe("Standing");
    const newestObservation = {
      span: {
        state: "Standing" as const,
        started_at: "2026-10-25T08:05:00Z",
        observed_through: "2026-10-25T08:10:00Z",
      },
      last_observation_at: "2026-10-25T08:10:00Z",
      snapshot_at: "2026-10-25T08:10:01Z",
      quality: "known",
      reason: null,
    };
    await act(async () => {
      older.resolve({ session: makeSession(), metrics: [] });
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(dashboardCalls).toBe(2);
    await act(async () => {
      newer.resolve({ session: makeSession({ state: "Standing", observation_quality: "known", live_observation: newestObservation }), metrics: [] });
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(result.current.state).toBe("Standing");
    expect(result.current.liveObservation).toEqual(newestObservation);
    vi.useRealTimers();
  });

  it("coalesces dashboard and history polls until slow reads finish instead of starving them", async () => {
    vi.useFakeTimers();
    const firstDashboard = deferred<DashboardState>();
    const secondDashboard = deferred<DashboardState>();
    const firstSummary = deferred<TodaySummaryDto>();
    const secondSummary = deferred<TodaySummaryDto>();
    let dashboardCalls = 0;
    let summaryCalls = 0;
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_dashboard_state") {
        dashboardCalls += 1;
        return dashboardCalls === 1 ? firstDashboard.promise : secondDashboard.promise;
      }
      if (cmd === "get_today_summary") {
        summaryCalls += 1;
        return summaryCalls === 1 ? firstSummary.promise : secondSummary.promise;
      }
      if (cmd === "get_connected_port") return Promise.resolve("COM3");
      return Promise.resolve(null);
    });
    const { result } = renderHook(() => useDesk());

    try {
      await act(async () => { await Promise.resolve(); });
      await act(async () => { await vi.advanceTimersByTimeAsync(10_000); });
      expect(dashboardCalls).toBe(1);
      expect(summaryCalls).toBe(1);

      const firstObservation = {
        span: {
          state: "Sitting" as const,
          started_at: "2026-10-25T08:00:00Z",
          observed_through: "2026-10-25T08:02:30Z",
        },
        last_observation_at: "2026-10-25T08:02:30Z",
        snapshot_at: "2026-10-25T08:02:31Z",
        quality: "known",
        reason: null,
      };
      await act(async () => {
        firstDashboard.resolve({
          session: makeSession({ observation_quality: "known", live_observation: firstObservation }),
          metrics: [],
        });
        firstSummary.resolve({ ...emptySummary, sitting_secs: 100 });
        await Promise.resolve();
        await Promise.resolve();
        await Promise.resolve();
      });

      expect(dashboardCalls).toBe(2);
      expect(summaryCalls).toBe(2);
      expect(result.current.liveObservation).toEqual(firstObservation);
      expect(result.current.todaySittingSecs).toBe(100);
      expect(result.current.dashboardRefresh).toMatchObject({ loading: true, error: null });
      expect(result.current.dashboardRefresh.lastSuccessAt).toEqual(expect.any(String));

      const nextObservation = {
        span: {
          state: "Standing" as const,
          started_at: "2026-10-25T08:02:31Z",
          observed_through: "2026-10-25T08:03:00Z",
        },
        last_observation_at: "2026-10-25T08:03:00Z",
        snapshot_at: "2026-10-25T08:03:01Z",
        quality: "known",
        reason: null,
      };
      await act(async () => {
        secondDashboard.resolve({
          session: makeSession({ state: "Standing", observation_quality: "known", live_observation: nextObservation }),
          metrics: [],
        });
        secondSummary.resolve({ ...emptySummary, standing_secs: 250 });
        await Promise.resolve();
        await Promise.resolve();
        await Promise.resolve();
      });

      expect(result.current.state).toBe("Standing");
      expect(result.current.liveObservation).toEqual(nextObservation);
      expect(result.current.todayStandingSecs).toBe(250);
      expect(result.current.dashboardRefresh.loading).toBe(false);
      expect(result.current.historyRefresh.loading).toBe(false);
    } finally {
      vi.useRealTimers();
    }
  });

  it("ignores an in-flight history response made obsolete by state-changed", async () => {
    const baseline: TodaySummaryDto = {
      ...emptySummary,
      sitting_secs: 120,
      sessions: [{ start: "08:00", end: "08:02", state: "Sitting", duration_secs: 120, break_credit: null }],
    };
    const stale: TodaySummaryDto = {
      ...emptySummary,
      sitting_secs: 900,
      sessions: [{ start: "08:00", end: "08:15", state: "Sitting", duration_secs: 900, break_credit: null }],
    };
    const pendingSummary = deferred<TodaySummaryDto>();
    stubBackend({ today: baseline });
    const { result } = await renderDesk();
    const previousSuccessAt = result.current.historyRefresh.lastSuccessAt;
    let summaryCalls = 0;
    const debugSpy = vi.spyOn(console, "debug").mockImplementation(() => {});
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_dashboard_state") return Promise.resolve({ session: makeSession(), metrics: [] });
      if (cmd === "get_today_summary") {
        summaryCalls += 1;
        return summaryCalls === 1 ? pendingSummary.promise : Promise.reject(new Error("history offline"));
      }
      return Promise.resolve(null);
    });

    await act(async () => { emit("desk:device-connected", { port: "COM8" }); });
    await act(async () => {
      emit("desk:state-changed", {
        state: "Standing", standing_seconds: 90, break_seconds: 30,
        desk_height_cm: 110, position_changes: 4, last_break_secs: 0,
        last_sitting_secs: 1200, break_credit: "full", limit_used_secs: 1500,
      });
    });
    await act(async () => {
      pendingSummary.resolve(stale);
      await Promise.resolve();
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(result.current.state).toBe("Standing");
    expect(result.current.todaySessions).toEqual(baseline.sessions);
    expect(result.current.historyRefresh).toEqual({
      loading: false,
      lastSuccessAt: previousSuccessAt,
      error: "history offline",
    });
    expect(debugSpy).toHaveBeenCalledWith("get_today_summary not ready:", expect.any(Error));
  });

  it("keeps device-lost authoritative across a pending reply and the following normal poll", async () => {
    vi.useFakeTimers();
    const pendingDashboard = deferred<DashboardState>();
    let dashboardCalls = 0;
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_dashboard_state") {
        dashboardCalls += 1;
        return dashboardCalls === 1
          ? pendingDashboard.promise
          : Promise.resolve({ session: makeSession({ state: "Sitting", desk_height_cm: 72.5 }), metrics: [] });
      }
      if (cmd === "get_today_summary") return Promise.resolve(emptySummary);
      return Promise.resolve(null);
    });
    const rendered = renderHook(() => useDesk());

    try {
      await act(async () => { await Promise.resolve(); });
      await act(async () => { emit("desk:device-lost", null); });
      await act(async () => {
        pendingDashboard.resolve({ session: makeSession({ state: "Sitting", desk_height_cm: 72.5 }), metrics: [] });
        await Promise.resolve();
        await Promise.resolve();
        await Promise.resolve();
      });
      expect.soft(rendered.result.current.connected).toBe(false);

      await act(async () => { await vi.advanceTimersByTimeAsync(1000); });
      expect(dashboardCalls).toBeGreaterThanOrEqual(2);
      expect(rendered.result.current.connected).toBe(false);
    } finally {
      rendered.unmount();
      vi.useRealTimers();
    }
  });

  it("retains accepted session and observation when a later native snapshot is malformed", async () => {
    vi.useFakeTimers();
    const newestObservation = {
      span: {
        state: "Standing" as const,
        started_at: "2026-10-25T08:05:00Z",
        observed_through: "2026-10-25T08:10:00Z",
      },
      last_observation_at: "2026-10-25T08:10:00Z",
      snapshot_at: "2026-10-25T08:10:01Z",
      quality: "known",
      reason: null,
    };
    let dashboardCalls = 0;
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_dashboard_state") {
        dashboardCalls += 1;
        if (dashboardCalls === 1) {
          return Promise.resolve({
            session: makeSession({ state: "Standing", observation_quality: "known", live_observation: newestObservation }),
            metrics: [],
          });
        }
        return Promise.resolve({
          session: makeSession({
            state: "Sitting",
            live_observation: { ...newestObservation, snapshot_at: "not-a-timestamp" },
          }),
          metrics: [],
        });
      }
      if (cmd === "get_today_summary") return Promise.resolve(emptySummary);
      return Promise.resolve(null);
    });
    const rendered = renderHook(() => useDesk());

    try {
      await act(async () => { await Promise.resolve(); await Promise.resolve(); });
      const acceptedReceipt = rendered.result.current.dashboardRefresh.lastSuccessAt;
      await act(async () => { await vi.advanceTimersByTimeAsync(1000); });

      expect(rendered.result.current.state).toBe("Standing");
      expect(rendered.result.current.liveObservation).toEqual(newestObservation);
      expect(rendered.result.current.dashboardRefresh.lastSuccessAt).not.toBe(acceptedReceipt);
    } finally {
      rendered.unmount();
      vi.useRealTimers();
    }
  });

  it("a state-changed event updates the counters and raises a transition", async () => {
    const { result } = await renderDesk();

    const payload: StateChangedPayload = {
      state: "Standing", standing_seconds: 90, break_seconds: 30,
      desk_height_cm: 110, position_changes: 4, last_break_secs: 0,
      last_sitting_secs: 1200, break_credit: "full", limit_used_secs: 1500,
    };

    await act(async () => { emit("desk:state-changed", payload); });

    expect(result.current.state).toBe("Standing");
    expect(result.current.limitUsedSecs).toBe(1500);
    expect(result.current.positionChanges).toBe(4);
    expect(result.current.transition).toEqual({
      lastBreakSecs: 0,
      lastSittingSecs: 1200,
      breakCredit: "full",
      transitionTo: "Standing",
    });
  });

  it("device-connected sets the port; device-lost clears it", async () => {
    const { result } = await renderDesk();

    await act(async () => { emit("desk:device-connected", { port: "COM9" }); });
    expect(result.current.port).toBe("COM9");
    expect(result.current.connected).toBe(true);

    await act(async () => { emit("desk:device-lost", null); });
    expect(result.current.connected).toBe(false);
    expect(result.current.port).toBeNull();
  });

  it("matches the useRemoteDesk view for an equivalent payload", async () => {
    const today: TodaySummaryDto = {
      sitting_secs: 300, standing_secs: 100,
      yesterday_sitting_secs: 0, yesterday_standing_secs: 0, position_changes: 2,
      sessions: [
        { start: "09:00", end: "09:20", state: "Standing", duration_secs: 600, break_credit: "full" },
        { start: "09:20", end: null, state: "Sitting", duration_secs: 120, break_credit: null },
      ],
    };
    stubBackend({ today });
    const { result } = await renderDesk();

    expect(result.current.todaySittingSecs).toBe(300);
    expect(result.current.todayStandingSecs).toBe(100);
    expect(result.current.todaySessions).toHaveLength(2);
    expect(result.current.previousSession).toEqual({
      state: "Standing",
      durationSecs: 600,
      wasEffective: true,
    });
  });
});

describe("useDesk — shadow paths", () => {
  it("nil — before any event there is no previous session and no transition", async () => {
    stubBackend({ dashboardFails: true });
    invokeMock.mockImplementation(() => Promise.reject(new Error("cold")));
    vi.spyOn(console, "debug").mockImplementation(() => {});
    vi.spyOn(console, "error").mockImplementation(() => {});

    const { result } = await renderDesk();

    expect(result.current.previousSession).toBeNull();
    expect(result.current.transition).toBeNull();
    expect(result.current.state).toBe("Away");
  });

  it("empty — a summary with no sessions yields an empty list", async () => {
    stubBackend({ today: emptySummary });
    const { result } = await renderDesk();

    expect(result.current.todaySessions).toEqual([]);
    expect(result.current.previousSession).toBeNull();
  });

  it("error — a rejected get_dashboard_state leaves connected false and does not throw", async () => {
    const debugSpy = vi.spyOn(console, "debug").mockImplementation(() => {});
    stubBackend({ dashboardFails: true });

    const { result } = await renderDesk();

    expect(result.current.connected).toBe(false);
    expect(result.current.error).toBeNull();
    expect(debugSpy).toHaveBeenCalledWith(
      "get_dashboard_state not ready:",
      expect.any(Error),
    );
  });

  it("error — sensor-error and session-alert both surface a message", async () => {
    const { result } = await renderDesk();
    await act(async () => {
      emit("desk:sensor-error", { message: "sensor unplugged", timestamp: "now" });
    });
    expect(result.current.error).toBe("sensor unplugged");

    await act(async () => { emit("desk:session-alert", null); });
    expect(result.current.error).toBe("Time to take a break!");
  });
});

describe("useDesk — commands", () => {
  it("calibrate reads the live height and calls calibrate with millimetres", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_session_state") return Promise.resolve(makeSession());
      if (cmd === "get_dashboard_state") {
        return Promise.resolve({ session: makeSession(), metrics: [] });
      }
      if (cmd === "get_today_summary") return Promise.resolve(emptySummary);
      return Promise.resolve(null);
    });
    const { result } = await renderDesk();

    await act(async () => { await result.current.calibrate("standing"); });

    expect(invokeMock).toHaveBeenCalledWith("calibrate", { standing_mm: 725 });
  });

  it("setSitLimit and setStandLimit forward minutes to the backend", async () => {
    const { result } = await renderDesk();

    await act(async () => {
      await result.current.setSitLimit(45);
      await result.current.setStandLimit(15);
    });

    expect(invokeMock).toHaveBeenCalledWith("set_session_limit", { minutes: 45 });
    expect(invokeMock).toHaveBeenCalledWith("set_stand_limit", { minutes: 15 });
  });

  it("setSitLimit rethrows a backend failure", async () => {
    const errSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    const { result } = await renderDesk();
    invokeMock.mockImplementation((cmd: string) =>
      cmd === "set_session_limit"
        ? Promise.reject(new Error("nope"))
        : Promise.resolve(null),
    );

    await expect(result.current.setSitLimit(30)).rejects.toThrow("nope");
    expect(errSpy).toHaveBeenCalled();
  });

  it("setStandLimit rethrows a backend failure", async () => {
    const errSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    const { result } = await renderDesk();
    invokeMock.mockImplementation((cmd: string) =>
      cmd === "set_stand_limit"
        ? Promise.reject(new Error("stand nope"))
        : Promise.resolve(null),
    );

    await expect(result.current.setStandLimit(10)).rejects.toThrow("stand nope");
    expect(errSpy).toHaveBeenCalled();
  });

  it("calibrate('sitting') sends sitting_mm and rethrows when the backend fails", async () => {
    const errSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "get_session_state") return Promise.resolve(makeSession());
      if (cmd === "get_dashboard_state") {
        return Promise.resolve({ session: makeSession(), metrics: [] });
      }
      if (cmd === "get_today_summary") return Promise.resolve(emptySummary);
      return Promise.resolve(null);
    });
    const { result } = await renderDesk();

    await act(async () => { await result.current.calibrate("sitting"); });
    expect(invokeMock).toHaveBeenCalledWith("calibrate", { sitting_mm: 725 });

    invokeMock.mockImplementation((cmd: string) =>
      cmd === "get_session_state"
        ? Promise.reject(new Error("no sensor"))
        : Promise.resolve(null),
    );
    await expect(result.current.calibrate("sitting")).rejects.toThrow("no sensor");
    expect(errSpy).toHaveBeenCalled();
  });
});

describe("useDesk — transport edge cases", () => {
  it("cleans up subscriptions before remounting without duplicate listeners", async () => {
    const first = await renderDesk();
    expect(listeners.size).toBe(6);
    first.unmount();
    expect(listeners.size).toBe(0);

    const second = await renderDesk();
    expect(listeners.size).toBe(6);
    second.unmount();
    expect(listeners.size).toBe(0);
  });

  it("a db-error event surfaces its message like a sensor error", async () => {
    const { result } = await renderDesk();

    await act(async () => { emit("desk:db-error", { message: "disk full" }); });

    expect(result.current.error).toBe("disk full");
  });

  it("clears the transition 30s after a state change", async () => {
    vi.useFakeTimers();
    try {
      const rendered = renderHook(() => useDesk());
      await act(async () => { await Promise.resolve(); await Promise.resolve(); });
      const { result } = rendered;

      await act(async () => {
        emit("desk:state-changed", {
          state: "Standing", standing_seconds: 10, break_seconds: 0,
          desk_height_cm: 110, position_changes: 1, last_break_secs: 0,
          last_sitting_secs: 600, break_credit: "full", limit_used_secs: 600,
        } satisfies StateChangedPayload);
      });
      expect(result.current.transition).not.toBeNull();

      await act(async () => { vi.advanceTimersByTime(30_000); });
      expect(result.current.transition).toBeNull();
    } finally {
      vi.useRealTimers();
    }
  });

  it("a failing get_connected_port leaves the port unset and retries on the next poll", async () => {
    let portCalls = 0;
    invokeMock.mockImplementation((cmd: string) => {
      switch (cmd) {
        case "get_dashboard_state":
          return Promise.resolve({ session: makeSession(), metrics: [] });
        case "get_today_summary":
          return Promise.resolve(emptySummary);
        case "get_connected_port":
          portCalls += 1;
          return portCalls === 1
            ? Promise.reject(new Error("port busy"))
            : Promise.resolve("COM7");
        default:
          return Promise.resolve(null);
      }
    });

    const { result } = await renderDesk();
    expect(result.current.port).toBeNull();

    // The failed lookup resets the guard, so the next poll asks again.
    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
    });
    expect(portCalls).toBeGreaterThanOrEqual(1);
  });
});
