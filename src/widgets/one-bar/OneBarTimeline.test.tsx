/**
 * OneBarTimeline.test.tsx — tests for the session timeline component.
 *
 * Verifies: hour markers always present, session blocks render,
 * tooltips, empty state, live block.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { act, fireEvent, render, screen } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { OneBarTimeline } from "./OneBarTimeline";
import "./one-bar.css";
import type { WidgetProps, SessionEntry } from "@/types";
import type { LiveObservation } from "@/generated/LiveObservation";
import { computeHourMarkers } from "@/utils/timeline";
import { createOpenDayRequest, localCalendarDay, OPEN_ANALYST_DAY_KEY, parseOpenDayRequest } from "@/analyst/open-day";

function baseProps(overrides: Partial<WidgetProps> = {}): WidgetProps {
  return {
    connected: true,
    port: "COM3",
    state: "Sitting",
    deskHeightCm: 72,
    limitUsedSecs: 600,
    limitSecs: 2400,
    standLimitSecs: 900,
    limitRemaining: 1800,
    limitRatio: 0.25,
    breakSecs: 0,
    breakResetThreshold: 600,
    breakResetProgress: 0,
    previousSession: null,
    todaySessions: [],
    todayChanges: 0,
    todayStandingSecs: 0,
    todaySittingSecs: 600,
    todayScore: 0,
    idleSecs: 0,
    continuousComputerSecs: 0,
    metrics: [],
    error: null,
    onOpenSettings: vi.fn(),
    ...overrides,
  };
}

function makeSessions(count: number, hoursAgo = 2): SessionEntry[] {
  const now = Date.now();
  const sessions: SessionEntry[] = [];
  const startMs = now - hoursAgo * 3600_000;
  const gap = (hoursAgo * 3600_000) / (count + 1);

  for (let i = 0; i < count; i++) {
    const start = startMs + i * gap;
    sessions.push({
      start: new Date(start).toISOString(),
      end: new Date(start + gap * 0.8).toISOString(),
      state: i % 2 === 0 ? "Sitting" : "Standing",
      duration_secs: Math.floor((gap * 0.8) / 1000),
      break_credit: null,
      history_kind: "individual",
    });
  }
  return sessions;
}

describe("OneBarTimeline", () => {
  beforeEach(() => {
    localStorage.removeItem(OPEN_ANALYST_DAY_KEY);
    vi.mocked(invoke).mockReset().mockResolvedValue(undefined as never);
    vi.stubGlobal("ResizeObserver", class {
      constructor(private readonly callback: ResizeObserverCallback) {}
      observe(target: Element) {
        this.callback([{ target, contentRect: { width: 420 } as DOMRectReadOnly } as ResizeObserverEntry], this as unknown as ResizeObserver);
      }
      disconnect() {}
      unobserve() {}
    });
  });

  afterEach(() => vi.unstubAllGlobals());

  it("shows empty message when no sessions and no live time", () => {
    render(<OneBarTimeline {...baseProps({ limitUsedSecs: 0, todaySessions: [] })} />);
    expect(screen.getByText("No recorded intervals yet")).toBeInTheDocument();
  });

  it("sends today's local calendar day when the empty timeline is activated", () => {
    render(<OneBarTimeline {...baseProps({ limitUsedSecs: 0, todaySessions: [] })} />);
    fireEvent.keyDown(screen.getByTestId("one-bar-timeline"), { key: "Enter" });
    expect(parseOpenDayRequest(localStorage.getItem(OPEN_ANALYST_DAY_KEY))?.day)
      .toBe(localCalendarDay(new Date()));
    expect(invoke).toHaveBeenCalledTimes(1);
    expect(invoke).toHaveBeenCalledWith("open_analyst_window");
  });

  it("uses the local day at activation time when midnight passes before the display clock tick", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date(2026, 9, 22, 23, 59, 30));
    const view = render(<OneBarTimeline {...baseProps({ limitUsedSecs: 0, todaySessions: [] })} />);
    vi.setSystemTime(new Date(2026, 9, 23, 0, 0, 5));
    fireEvent.click(screen.getByTestId("one-bar-timeline"));
    expect(parseOpenDayRequest(localStorage.getItem(OPEN_ANALYST_DAY_KEY))?.day)
      .toBe("2026-10-23");
    view.unmount();
    vi.useRealTimers();
  });

  it("opens Analyst from nonempty timeline background without stealing interval selection", () => {
    const sessions = makeSessions(2);
    render(<OneBarTimeline {...baseProps({ todaySessions: sessions })} />);
    const timeline = screen.getByTestId("one-bar-timeline");
    const interval = screen.getAllByTestId("timeline-segment")[0];
    fireEvent.click(interval);
    expect(invoke).not.toHaveBeenCalled();
    expect(interval).toHaveAttribute("aria-pressed", "true");
    fireEvent.keyDown(interval, { key: "Enter" });
    expect(invoke).not.toHaveBeenCalled();

    fireEvent.click(timeline);
    expect(invoke).toHaveBeenCalledTimes(1);
    expect(parseOpenDayRequest(localStorage.getItem(OPEN_ANALYST_DAY_KEY))?.day)
      .toBe(localCalendarDay(new Date()));
  });

  it("uses unique requests for repeated same-day activation", () => {
    render(<OneBarTimeline {...baseProps({ limitUsedSecs: 0, todaySessions: [] })} />);
    const timeline = screen.getByTestId("one-bar-timeline");
    fireEvent.click(timeline);
    const first = localStorage.getItem(OPEN_ANALYST_DAY_KEY);
    fireEvent.click(timeline);
    const second = localStorage.getItem(OPEN_ANALYST_DAY_KEY);
    expect(JSON.parse(second!).requestId).not.toBe(JSON.parse(first!).requestId);
    expect(invoke).toHaveBeenCalledTimes(2);
  });

  it("still opens the window when local storage is denied", () => {
    const setItem = vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("storage denied");
    });
    const warn = vi.spyOn(console, "warn").mockImplementation(() => undefined);
    render(<OneBarTimeline {...baseProps({ limitUsedSecs: 0, todaySessions: [] })} />);
    const timeline = screen.getByTestId("one-bar-timeline");
    fireEvent.click(timeline);
    expect(invoke).toHaveBeenCalledTimes(1);
    expect(warn).toHaveBeenCalled();
    setItem.mockRestore();
    warn.mockRestore();
  });

  it("cleans up only its failed request and allows a retry", async () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => undefined);
    vi.mocked(invoke).mockRejectedValueOnce(new Error("window unavailable"));
    render(<OneBarTimeline {...baseProps({ limitUsedSecs: 0, todaySessions: [] })} />);
    const timeline = screen.getByTestId("one-bar-timeline");
    fireEvent.click(timeline);
    const newer = createOpenDayRequest("2026-10-23", Date.now(), "newer-request");
    localStorage.setItem(OPEN_ANALYST_DAY_KEY, JSON.stringify(newer));

    await vi.waitFor(() => expect(warn).toHaveBeenCalledWith("open_analyst_window failed:", expect.any(Error)));
    expect(localStorage.getItem(OPEN_ANALYST_DAY_KEY)).toBe(JSON.stringify(newer));
    fireEvent.click(timeline);
    expect(invoke).toHaveBeenCalledTimes(2);
    warn.mockRestore();
  });

  it("renders session blocks for each session", () => {
    const sessions = makeSessions(3);
    render(<OneBarTimeline {...baseProps({ todaySessions: sessions })} />);
    const timeline = screen.getByTestId("one-bar-timeline");
    const blocks = timeline.querySelectorAll(".one-bar__timeline-block");
    expect(blocks).toHaveLength(3);
    expect(screen.queryByTestId("timeline-live-block")).not.toBeInTheDocument();
  });

  it("keeps legacy Sitting unknown while rendering qualified Sitting in compact chronology", () => {
    const now = Date.now();
    const sessions: SessionEntry[] = [
      {
        start: new Date(now - 20 * 60_000).toISOString(),
        end: new Date(now - 10 * 60_000).toISOString(),
        state: "Sitting",
        duration_secs: 600,
        break_credit: null,
      },
      {
        start: new Date(now - 10 * 60_000).toISOString(),
        end: new Date(now - 5 * 60_000).toISOString(),
        state: "Sitting",
        duration_secs: 300,
        break_credit: null,
        history_kind: "individual",
      },
    ];
    render(<OneBarTimeline {...baseProps({ todaySessions: sessions })} />);
    const segments = screen.getAllByTestId("timeline-segment");
    expect(segments).toHaveLength(1);
    expect(segments[0]).toHaveAttribute("data-state", "Sitting");
    expect(segments[0]).toHaveAttribute("data-start-ms", String(Date.parse(sessions[1].start)));
  });

  it("does not extend a current posture from credited load without a fresh observed endpoint", () => {
    const sessions = makeSessions(1);
    render(<OneBarTimeline {...baseProps({ todaySessions: sessions, limitUsedSecs: 300 })} />);
    expect(screen.queryByTestId("timeline-live-block")).not.toBeInTheDocument();
    expect(screen.getAllByTestId("timeline-unknown")).toHaveLength(1);
  });

  it("renders the retained live span before a completed row and stops at observed-through", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-25T11:03:00+02:00"));
    const liveObservation: LiveObservation = {
      span: {
        state: "Sitting",
        started_at: "2026-10-25T10:00:00+02:00",
        observed_through: "2026-10-25T10:20:00+02:00",
      },
      last_observation_at: "2026-10-25T10:20:00+02:00",
      snapshot_at: "2026-10-25T11:00:00+02:00",
      quality: null,
      reason: "stale",
    };
    const props = { ...baseProps({ todaySessions: [] }), liveObservation };
    try {
      render(<OneBarTimeline {...props} />);
      expect(screen.getByTestId("timeline-live-block")).toHaveAttribute(
        "data-end-ms",
        String(Date.parse(liveObservation.span!.observed_through)),
      );
      expect(screen.getAllByTestId("timeline-unknown")).toHaveLength(1);
    } finally {
      vi.useRealTimers();
    }
  });

  it("grows the live prefix on a new reading without waiting for the minute display tick", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-25T10:00:05Z"));
    const makeObservation = (through: string): LiveObservation => ({
      span: {
        state: "Sitting",
        started_at: "2026-10-25T10:00:00Z",
        observed_through: through,
      },
      last_observation_at: through,
      snapshot_at: through,
      quality: "known",
      reason: null,
    });
    const firstProps = { ...baseProps({ todaySessions: [], dashboardRefresh: { loading: false, lastSuccessAt: "2026-10-25T10:00:05Z", error: null } }), liveObservation: makeObservation("2026-10-25T10:00:05Z") };
    try {
      const view = render(<OneBarTimeline {...firstProps} />);
      expect(screen.getByTestId("timeline-live-block")).toHaveAttribute("data-end-ms", String(Date.parse("2026-10-25T10:00:05Z")));

      vi.setSystemTime(new Date("2026-10-25T10:00:06Z"));
      view.rerender(<OneBarTimeline {...{ ...firstProps,
        dashboardRefresh: { loading: false, lastSuccessAt: "2026-10-25T10:00:06Z", error: null },
        liveObservation: makeObservation("2026-10-25T10:00:06Z") }} />);

      expect(screen.getByTestId("timeline-live-block")).toHaveAttribute("data-end-ms", String(Date.parse("2026-10-25T10:00:06Z")));
    } finally {
      vi.useRealTimers();
    }
  });

  it("does not let a future producer snapshot extend the popup's current-time window", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-25T10:21:05Z"));
    const observation: LiveObservation = {
      span: { state: "Sitting", started_at: "2026-10-25T10:00:00Z", observed_through: "2026-10-25T10:20:00Z" },
      last_observation_at: "2026-10-25T10:20:00Z",
      snapshot_at: "2026-10-26T12:00:00Z",
      quality: null,
      reason: "stale",
    };
    try {
      render(<OneBarTimeline {...baseProps({ todaySessions: [], liveObservation: observation,
        dashboardRefresh: { loading: false, lastSuccessAt: "2026-10-25T10:21:05Z", error: null } })} />);
      expect(screen.getByTestId("timeline-strip")).toHaveAttribute("data-end-ms", String(Date.parse("2026-10-25T10:21:05Z")));
    } finally {
      vi.useRealTimers();
    }
  });

  it("follows the client clock backward across local midnight on success and on the next failure tick", () => {
    const originalTimeZone = process.env.TZ;
    vi.stubEnv("TZ", "Europe/Warsaw");
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-27T00:01:00Z"));
    const row: SessionEntry = {
      start: "2026-10-26T22:30:00Z", end: "2026-10-26T22:40:00Z", state: "Sitting",
      duration_secs: 600, break_credit: null, history_kind: "individual",
    };
    const propsAt = (lastSuccessAt: string, error: string | null) => baseProps({ todaySessions: [row],
      dashboardRefresh: { loading: false, lastSuccessAt, error } });
    try {
      const view = render(<OneBarTimeline {...propsAt("2026-10-27T00:01:00Z", null)} />);
      vi.setSystemTime(new Date("2026-10-26T22:59:00Z"));
      view.rerender(<OneBarTimeline {...propsAt("2026-10-26T22:59:00Z", null)} />);
      expect(screen.getByTestId("timeline-strip")).toHaveAttribute("data-end-ms", String(Date.parse("2026-10-26T22:59:00Z")));

      vi.setSystemTime(new Date("2026-10-26T22:50:00Z"));
      view.rerender(<OneBarTimeline {...propsAt("2026-10-26T22:59:00Z", "dashboard refresh failed")} />);
      act(() => vi.advanceTimersByTime(60_000));
      expect(screen.getByTestId("timeline-strip")).toHaveAttribute("data-end-ms", String(Date.parse("2026-10-26T22:51:00Z")));
    } finally {
      vi.useRealTimers();
      if (originalTimeZone === undefined) vi.unstubAllEnvs();
      else process.env.TZ = originalTimeZone;
    }
  });

  it("uses the rendered window origin for hour markers when an older retained prefix remains", () => {
    const originalTimeZone = process.env.TZ;
    vi.stubEnv("TZ", "UTC");
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-25T14:05:00Z"));
    const prior: LiveObservation = {
      span: { state: "Sitting", started_at: "2026-10-25T10:00:00Z", observed_through: "2026-10-25T10:20:00Z" },
      last_observation_at: "2026-10-25T10:20:00Z", snapshot_at: "2026-10-25T10:20:00Z", quality: null, reason: "stale",
    };
    const current: LiveObservation = {
      span: { state: "Standing", started_at: "2026-10-25T13:00:00Z", observed_through: "2026-10-25T14:00:00Z" },
      last_observation_at: "2026-10-25T14:00:00Z", snapshot_at: "2026-10-25T14:00:00Z", quality: "known", reason: null,
    };
    const laterRow: SessionEntry = {
      start: "2026-10-25T12:00:00Z", end: "2026-10-25T12:10:00Z", state: "Sitting",
      duration_secs: 600, break_credit: null, history_kind: "individual",
    };
    try {
      const view = render(<OneBarTimeline {...baseProps({ todaySessions: [], liveObservation: prior })} />);
      view.rerender(<OneBarTimeline {...baseProps({ todaySessions: [laterRow], liveObservation: current })} />);
      const firstHourLabel = screen.getByTestId("one-bar-timeline").querySelector(".one-bar__timeline-hour-label");
      expect(firstHourLabel).toHaveTextContent("10:00");
      expect(screen.getByTestId("timeline-strip")).toHaveAttribute("data-start-ms", String(Date.parse(prior.span!.started_at)));
    } finally {
      vi.useRealTimers();
      if (originalTimeZone === undefined) vi.unstubAllEnvs();
      else process.env.TZ = originalTimeZone;
    }
  });

  it("ends marker instants and proportions at the same window end as the strip", () => {
    const originalTimeZone = process.env.TZ;
    vi.stubEnv("TZ", "UTC");
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-25T14:05:00Z"));
    const refresh = { loading: false, lastSuccessAt: "2026-10-25T14:05:00Z", error: null };
    const row: SessionEntry = {
      start: "2026-10-25T10:00:00Z", end: "2026-10-25T10:10:00Z", state: "Sitting",
      duration_secs: 600, break_credit: null, history_kind: "individual",
    };
    const laterRow: SessionEntry = {
      start: "2026-10-25T11:00:00Z", end: "2026-10-25T11:10:00Z", state: "Standing",
      duration_secs: 600, break_credit: null, history_kind: "individual",
    };
    try {
      const view = render(<OneBarTimeline {...baseProps({ todaySessions: [row], dashboardRefresh: refresh })} />);
      vi.setSystemTime(new Date("2026-10-25T15:00:00Z"));
      view.rerender(<OneBarTimeline {...baseProps({ todaySessions: [row, laterRow], dashboardRefresh: refresh })} />);
      const markers = [...screen.getByTestId("one-bar-timeline").querySelectorAll<HTMLElement>(".one-bar__timeline-hour")];
      const endLabel = markers[markers.length - 1]?.querySelector(".one-bar__timeline-hour-label--end");
      expect(endLabel).toHaveTextContent("14:05");
      expect(endLabel).not.toHaveTextContent("15:00");
      const noon = markers[2];
      const expectedPct = ((Date.parse("2026-10-25T12:00:00Z") - Date.parse(row.start)) /
        (Date.parse("2026-10-25T14:05:00Z") - Date.parse(row.start))) * 100;
      expect(Number.parseFloat(noon!.style.left)).toBeCloseTo(expectedPct, 2);
      expect(screen.getByTestId("timeline-strip")).toHaveAttribute("data-end-ms", String(Date.parse("2026-10-25T14:05:00Z")));
    } finally {
      vi.useRealTimers();
      if (originalTimeZone === undefined) vi.unstubAllEnvs();
      else process.env.TZ = originalTimeZone;
    }
  });

  it("clips a continuing live span at today's Warsaw local midnight", () => {
    const originalTimeZone = process.env.TZ;
    vi.stubEnv("TZ", "Europe/Warsaw");
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-27T00:03:00Z"));
    const observation: LiveObservation = {
      span: { state: "Standing", started_at: "2026-10-26T22:58:00Z", observed_through: "2026-10-26T23:02:00Z" },
      last_observation_at: "2026-10-26T23:02:00Z",
      snapshot_at: "2026-10-26T23:02:00Z",
      quality: "known",
      reason: null,
    };
    try {
      render(<OneBarTimeline {...baseProps({ todaySessions: [], liveObservation: observation,
        dashboardRefresh: { loading: false, lastSuccessAt: "2026-10-27T00:03:00Z", error: null } })} />);
      expect(screen.getByTestId("timeline-strip")).toHaveAttribute("data-start-ms", String(Date.parse("2026-10-26T23:00:00Z")));
      expect(screen.getByTestId("timeline-live-block")).toHaveAttribute("data-start-ms", String(Date.parse("2026-10-26T23:00:00Z")));
    } finally {
      vi.useRealTimers();
      if (originalTimeZone === undefined) vi.unstubAllEnvs();
      else process.env.TZ = originalTimeZone;
    }
  });

  it("replaces the live block with its recorded row in either arrival order", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-25T11:03:00+02:00"));
    const liveObservation: LiveObservation = {
      span: {
        state: "Sitting",
        started_at: "2026-10-25T10:00:00+02:00",
        observed_through: "2026-10-25T10:20:00+02:00",
      },
      last_observation_at: "2026-10-25T10:20:00+02:00",
      snapshot_at: "2026-10-25T11:00:00+02:00",
      quality: null,
      reason: "stale",
    };
    const row: SessionEntry = {
      start: "2026-10-25T10:00:00+02:00",
      end: "2026-10-25T10:20:00+02:00",
      state: "Sitting",
      duration_secs: 1200,
      break_credit: null,
      history_kind: "individual",
    };
    try {
      const liveFirst = render(<OneBarTimeline {...{ ...baseProps({ todaySessions: [] }), liveObservation }} />);
      expect(screen.getAllByTestId("timeline-live-block")).toHaveLength(1);
      liveFirst.rerender(<OneBarTimeline {...{ ...baseProps({ todaySessions: [row] }), liveObservation }} />);
      expect(screen.queryByTestId("timeline-live-block")).not.toBeInTheDocument();
      expect(screen.getAllByTestId("timeline-segment")).toHaveLength(1);
      expect(screen.getByTestId("timeline-segment")).toHaveAttribute("data-end-ms", String(Date.parse(row.end!)));
      liveFirst.unmount();

      const rowFirst = render(<OneBarTimeline {...baseProps({ todaySessions: [row] })} />);
      expect(screen.getAllByTestId("timeline-segment")).toHaveLength(1);
      rowFirst.rerender(<OneBarTimeline {...{ ...baseProps({ todaySessions: [row] }), liveObservation }} />);
      expect(screen.getAllByTestId("timeline-segment")).toHaveLength(1);
      expect(screen.queryByTestId("timeline-live-block")).not.toBeInTheDocument();
    } finally {
      vi.useRealTimers();
    }
  });

  it("retains the closed posture prefix while the new live posture arrives before its row", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-25T10:21:05Z"));
    const oldObservation: LiveObservation = {
      span: { state: "Sitting", started_at: "2026-10-25T10:00:00Z", observed_through: "2026-10-25T10:20:00Z" },
      last_observation_at: "2026-10-25T10:20:00Z", snapshot_at: "2026-10-25T10:20:00Z", quality: "known", reason: null,
    };
    const nextObservation: LiveObservation = {
      span: { state: "Standing", started_at: "2026-10-25T10:20:00Z", observed_through: "2026-10-25T10:21:00Z" },
      last_observation_at: "2026-10-25T10:21:00Z", snapshot_at: "2026-10-25T10:21:00Z", quality: "known", reason: null,
    };
    const closedRow: SessionEntry = {
      start: oldObservation.span!.started_at, end: oldObservation.span!.observed_through,
      state: "Sitting", duration_secs: 1200, break_credit: null, history_kind: "individual",
    };
    try {
      const view = render(<OneBarTimeline {...{ ...baseProps({ todaySessions: [] }), liveObservation: oldObservation }} />);
      view.rerender(<OneBarTimeline {...{ ...baseProps({ todaySessions: [] }), liveObservation: nextObservation }} />);
      expect(screen.getAllByTestId("timeline-segment")).toHaveLength(1);
      expect(screen.getByTestId("timeline-segment")).toHaveAttribute("data-start-ms", String(Date.parse(oldObservation.span!.started_at)));
      expect(screen.getByTestId("timeline-live-block")).toHaveAttribute("data-end-ms", String(Date.parse(nextObservation.span!.observed_through)));

      // A failed history refresh leaves the prior prefix visible until its row arrives.
      view.rerender(<OneBarTimeline {...{ ...baseProps({ todaySessions: [], dashboardRefresh: { loading: false, lastSuccessAt: null, error: "history refresh failed" } }), liveObservation: nextObservation }} />);
      expect(screen.getAllByTestId("timeline-segment")).toHaveLength(1);
      expect(screen.getByTestId("timeline-live-block")).toBeInTheDocument();
      view.rerender(<OneBarTimeline {...{ ...baseProps({ todaySessions: [closedRow] }), liveObservation: nextObservation }} />);
      expect(screen.getAllByTestId("timeline-live-block")).toHaveLength(1);
      expect(screen.getByTestId("timeline-segment")).toHaveAttribute("data-start-ms", String(Date.parse(closedRow.start)));
    } finally {
      vi.useRealTimers();
    }
  });

  it("always shows hour markers when sessions exist", () => {
    const sessions = makeSessions(1, 0.1); // sessions within same hour
    render(<OneBarTimeline {...baseProps({ todaySessions: sessions })} />);
    const timeline = screen.getByTestId("one-bar-timeline");
    const markers = timeline.querySelectorAll(".one-bar__timeline-hour");
    const hourLabels = timeline.querySelectorAll(".one-bar__timeline-hour-label");
    // Keep all markers while showing the readable endpoint labels when width is unknown.
    expect(markers.length).toBeGreaterThanOrEqual(2);
    expect(hourLabels.length).toBeGreaterThanOrEqual(2);
    expect(hourLabels.length).toBeLessThanOrEqual(markers.length);
  });

  it("shows more markers for multi-hour sessions than single-hour", () => {
    const shortSessions = makeSessions(1, 0.1);
    const { unmount } = render(<OneBarTimeline {...baseProps({ todaySessions: shortSessions })} />);
    const shortMarkers = screen.getByTestId("one-bar-timeline")
      .querySelectorAll(".one-bar__timeline-hour").length;
    unmount();

    const longSessions = makeSessions(4, 3);
    render(<OneBarTimeline {...baseProps({ todaySessions: longSessions })} />);
    const longMarkers = screen.getByTestId("one-bar-timeline")
      .querySelectorAll(".one-bar__timeline-hour").length;

    expect(longMarkers).toBeGreaterThanOrEqual(shortMarkers);
  });

  it("generates hourly markers across midnight crossing", () => {
    // Session started 10 hours ago — guaranteed to cross at least one midnight hour boundary
    const sessions = makeSessions(4, 10);
    render(<OneBarTimeline {...baseProps({ todaySessions: sessions })} />);
    const timeline = screen.getByTestId("one-bar-timeline");
    const markers = timeline.querySelectorAll(".one-bar__timeline-hour");
    const hourLabels = timeline.querySelectorAll(".one-bar__timeline-hour-label");
    // Start + multiple hour boundaries + now = at least 5 markers for 10h span
    expect(markers.length).toBeGreaterThanOrEqual(5);
    expect(hourLabels.length).toBeGreaterThanOrEqual(2);
    // The measured-width layout keeps full-hour labels when they fit.
    const texts = Array.from(hourLabels).map((element) => element.textContent ?? "");
    expect(texts.some((text) => text.endsWith(":00"))).toBe(true);
  });

  it("keeps both distinct repeated local-hour instants and labels their runtime offsets", () => {
    const originalTimeZone = process.env.TZ;
    vi.stubEnv("TZ", "Europe/Warsaw");
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-25T01:30:00Z"));
    try {
      const markers = computeHourMarkers({ startMs: Date.parse("2026-10-24T22:00:00Z"), endMs: Date.now() });
      const repeated = markers.filter((marker) => marker.label.startsWith("2:00"));
      expect(repeated).toHaveLength(2);
      expect(repeated.map((marker) => marker.atMs)).toEqual([
        Date.parse("2026-10-25T00:00:00Z"),
        Date.parse("2026-10-25T01:00:00Z"),
      ]);
      expect(repeated.map((marker) => marker.label)).toEqual(["2:00 UTC+02:00", "2:00 UTC+01:00"]);
      expect(repeated[1].leftPct).toBeGreaterThan(repeated[0].leftPct);
    } finally {
      vi.useRealTimers();
      if (originalTimeZone === undefined) vi.unstubAllEnvs();
      else process.env.TZ = originalTimeZone;
    }
  });

  it("keeps Lord Howe half-hour transition markers truthful and after a second-fold start", () => {
    const originalTimeZone = process.env.TZ;
    vi.stubEnv("TZ", "Australia/Lord_Howe");
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-04-04T16:45:00Z"));
    try {
      const startBeforeFold = Date.parse("2026-04-04T14:10:00Z");
      const markers = computeHourMarkers({ startMs: startBeforeFold, endMs: Date.now() });
      expect(markers.some((marker) => marker.atMs === Date.parse("2026-04-04T15:30:00Z") && marker.label === "2:00")).toBe(true);
      expect(markers.some((marker) => marker.label === "1:00" && marker.atMs === Date.parse("2026-04-04T15:00:00Z"))).toBe(false);

      const secondFoldStart = Date.parse("2026-04-04T15:15:00Z");
      const fromSecondFold = computeHourMarkers({ startMs: secondFoldStart, endMs: Date.now() });
      expect(fromSecondFold.slice(1, -1).every((marker) => marker.atMs > secondFoldStart)).toBe(true);
      expect(fromSecondFold.some((marker) => marker.atMs === Date.parse("2026-04-04T15:30:00Z") && marker.label === "2:00")).toBe(true);
      expect(fromSecondFold.some((marker) => marker.atMs === Date.parse("2026-04-04T16:30:00Z") && marker.label === "3:00")).toBe(true);
      for (const marker of fromSecondFold.slice(1, -1)) {
        const local = new Date(marker.atMs);
        const actual = local.getHours() + ":" + String(local.getMinutes()).padStart(2, "0");
        expect(marker.label.startsWith(actual)).toBe(true);
      }
    } finally {
      vi.useRealTimers();
      if (originalTimeZone === undefined) vi.unstubAllEnvs();
      else process.env.TZ = originalTimeZone;
    }
  });

  it("uses recorded endpoints rather than the nullable summary duration", () => {
    const sessions = makeSessions(2, 1);
    sessions[0] = { ...sessions[0], duration_secs: null };
    render(<OneBarTimeline {...baseProps({ todaySessions: sessions })} />);
    const timeline = screen.getByTestId("one-bar-timeline");
    const completed = Array.from(timeline.querySelectorAll(".one-bar__timeline-block"));
    expect(completed).toHaveLength(2);
    expect(completed[0]).toHaveAttribute("data-start-ms", String(Date.parse(sessions[0].start)));
    expect(completed[0]).toHaveAttribute("data-end-ms", String(Date.parse(sessions[0].end!)));
  });

  it("hour labels contain time format with colon", () => {
    const sessions = makeSessions(2, 0.5);
    render(<OneBarTimeline {...baseProps({ todaySessions: sessions })} />);
    const timeline = screen.getByTestId("one-bar-timeline");
    const labels = timeline.querySelectorAll(".one-bar__timeline-hour-label");
    for (const label of labels) {
      expect(label.textContent).toMatch(/\d+:\d{2}/);
    }
  });

  it("keeps the compact axis and readable labels inside its 24px strip", () => {
    const sessions = makeSessions(2, 3);
    render(<OneBarTimeline {...baseProps({ todaySessions: sessions })} />);
    const timeline = screen.getByTestId("one-bar-timeline");
    const strip = screen.getByTestId("timeline-strip");
    const labels = [...timeline.querySelectorAll<HTMLElement>(".one-bar__timeline-hour-label")];

    expect(timeline).toHaveStyle({ height: "24px", minHeight: "24px", maxHeight: "24px" });
    expect(strip).toHaveStyle({ height: "24px", minHeight: "24px" });
    expect(labels.length).toBeGreaterThan(1);
    expect(labels.every((label) => Number.parseFloat(getComputedStyle(label).fontSize) >= 10)).toBe(true);
    expect(labels[0]).toHaveClass("one-bar__timeline-hour-label--start");
    expect(labels[labels.length - 1]).toHaveClass("one-bar__timeline-hour-label--end");
  });

  it("keeps all tick markers and spaces the readable label subset within 420px", () => {
    const fakeTimersWereActive = vi.isFakeTimers();
    const previousClock = vi.getMockedSystemTime();
    const previousNow = Date.now();
    let unmount: (() => void) | undefined;
    if (!fakeTimersWereActive) vi.useFakeTimers();
    try {
      // Mid-June is outside the Warsaw spring/fall clock-change window.
      vi.setSystemTime(new Date("2026-06-15T12:00:00+02:00"));
      const sessions = makeSessions(1, 21);
      const view = render(<OneBarTimeline {...baseProps({ todaySessions: sessions })} />);
      unmount = view.unmount;
      const timeline = screen.getByTestId("one-bar-timeline");
      const markers = [...timeline.querySelectorAll<HTMLElement>(".one-bar__timeline-hour")];
      const labels = [...timeline.querySelectorAll<HTMLElement>(".one-bar__timeline-hour-label")];

      expect(markers.length).toBeGreaterThan(20);
      expect(labels.length).toBeGreaterThan(2);
      expect(labels.length).toBeLessThan(markers.length);
      expect(labels[0]).toHaveClass("one-bar__timeline-hour-label--start");
      expect(labels[labels.length - 1]).toHaveClass("one-bar__timeline-hour-label--end");
      expect(labels.every((label) => Number.parseFloat(getComputedStyle(label).fontSize) >= 10)).toBe(true);

      const boxes = labels.map((label) => {
        const marker = label.parentElement!;
        const center = Number.parseFloat(marker.style.left) / 100 * 420;
        const width = (label.textContent?.length ?? 0) * 6 + 4;
        const left = label.classList.contains("one-bar__timeline-hour-label--start")
          ? center
          : label.classList.contains("one-bar__timeline-hour-label--end") ? center - width : center - width / 2;
        return { left, right: left + width };
      });
      for (let index = 1; index < boxes.length; index += 1) {
        expect(boxes[index].left - boxes[index - 1].right).toBeGreaterThanOrEqual(8);
      }
    } finally {
      unmount?.();
      if (fakeTimersWereActive) vi.setSystemTime(previousClock ?? new Date(previousNow));
      else vi.useRealTimers();
    }
  });
});
