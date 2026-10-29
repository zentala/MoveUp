import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { SessionRow, SnapshotRow } from "@/test/analyst-fixtures";
import type { LiveObservation } from "@/types";
import { ExplorerTab } from "./ExplorerTab";
import { localCalendarDay } from "./open-day";

const eventQueryMock = vi.hoisted(() => ({ refetch: vi.fn() }));
const coreQueryMock = vi.hoisted(() => ({
  snapshots: { status: "ready" as string, data: [] as SnapshotRow[] | null, error: null as string | null, lastRefreshed: null as number | string | null, refetch: vi.fn() },
  sessions: { status: "ready" as string, data: [] as SessionRow[] | null, error: null as string | null, lastRefreshed: null as number | string | null, refetch: vi.fn() },
}));

vi.mock("./hooks/useSnapshotsRange", () => ({
  useSnapshotsRange: () => coreQueryMock.snapshots,
}));
vi.mock("./hooks/useSessionsRange", () => ({
  useSessionsRange: () => coreQueryMock.sessions,
}));
vi.mock("./hooks/useEventsRange", () => ({
  useEventsRange: () => ({
    status: "error", data: null, error: "event source failed",
    refetch: eventQueryMock.refetch, lastRefreshed: null,
  }),
}));

vi.mock("./hooks/useLocalDayBoundsRange", () => ({
  useLocalDayBoundsRange: (from: string, to: string) => {
    const rows: Array<{ date_local: string; start_utc: string; end_utc: string; zone_basis: string; historical_source_zone: string }> = [];
    const day = new Date(`${from}T00:00:00`);
    const last = new Date(`${to}T00:00:00`);
    while (day <= last) {
      const date_local = `${day.getFullYear()}-${String(day.getMonth() + 1).padStart(2, "0")}-${String(day.getDate()).padStart(2, "0")}`;
      const start = new Date(day.getTime());
      const end = new Date(day.getTime());
      end.setDate(end.getDate() + 1);
      rows.push({ date_local, start_utc: start.toISOString(), end_utc: end.toISOString(), zone_basis: "current-query-local", historical_source_zone: "unknown" });
      day.setDate(day.getDate() + 1);
    }
    return { status: "ready", data: rows, error: null, refetch: () => undefined, lastRefreshed: null };
  },
}));

const range = { from: "2026-05-15", to: "2026-05-17" };
const snapshots: SnapshotRow[] = [];
const sessions: SessionRow[] = [];

// Synthetic IDs/date/times: an individual Sitting interval contains an Away
// interval, matching the observed conflict shape without copying live rows.
const conflict: SessionRow[] = [
  { id: 1, startedAt: "2026-05-16T08:00:00Z", endedAt: "2026-05-16T08:18:14Z", state: "Sitting",
    durationSecs: 1094, sittingSecs: null, standingSecs: null, positionChanges: null,
    breakCredit: null, historyKind: "individual", dateLocal: "2026-05-16" },
  { id: 2, startedAt: "2026-05-16T08:02:01Z", endedAt: "2026-05-16T08:04:22Z", state: "Away",
    durationSecs: 141, sittingSecs: null, standingSecs: null, positionChanges: null,
    breakCredit: "partial", historyKind: "individual", dateLocal: "2026-05-16" },
];

beforeEach(() => {
  vi.clearAllMocks();
  coreQueryMock.snapshots = { status: "ready", data: [], error: null, refetch: vi.fn(), lastRefreshed: null };
  coreQueryMock.sessions = { status: "ready", data: [], error: null, refetch: vi.fn(), lastRefreshed: null };
});

describe("ExplorerTab day assembly", () => {
  it("opens the actual local Today from a retained custom range and exposes orientation progressively", () => {
    const originalTimeZone = process.env.TZ;
    vi.stubEnv("TZ", "Europe/Warsaw");
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-06-20T10:00:00Z"));
    const onRangeChange = vi.fn();
    const rangeBefore = { from: "2026-06-01", to: "2026-06-10" };
    const rangeToday = { from: "2026-06-11", to: "2026-06-20" };
    try {
      const view = render(<ExplorerTab range={rangeBefore} onRangeChange={onRangeChange} snapshots={[]} sessions={[]} />);
      const orientation = screen.getByText("Coverage and time basis").closest("details");
      const orientationText = screen.getByText(/Calendar bounds use the current query-local timezone/);
      expect(orientation).not.toHaveAttribute("open");
      expect(orientationText).not.toBeVisible();
      fireEvent.click(screen.getByText("Coverage and time basis"));
      expect(orientationText).toBeVisible();
      expect(orientation).toHaveTextContent("Unknown is uncovered elapsed time");
      expect(orientation).toHaveTextContent("historical source timezone is unknown");

      fireEvent.click(screen.getByRole("button", { name: "Today" }));
      expect(onRangeChange).toHaveBeenCalledWith(rangeToday);
      view.rerender(<ExplorerTab range={rangeToday} onRangeChange={onRangeChange} snapshots={[]} sessions={[]} />);
      expect(screen.getByRole("tab", { name: /2026-06-20/ })).toHaveAttribute("aria-selected", "true");
      expect(screen.getByRole("region", { name: "Daily timeline for 2026-06-20" })).toBeInTheDocument();
    } finally {
      vi.useRealTimers();
      if (originalTimeZone === undefined) vi.unstubAllEnvs();
      else process.env.TZ = originalTimeZone;
    }
  });

  it("honors a newer external selected-day request while Today is waiting for its expanded range", () => {
    const originalTimeZone = process.env.TZ;
    vi.stubEnv("TZ", "Europe/Warsaw");
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-06-20T10:00:00Z"));
    const onRangeChange = vi.fn();
    const rangeBefore = { from: "2026-06-01", to: "2026-06-10" };
    const rangeToday = { from: "2026-06-11", to: "2026-06-20" };
    try {
      const view = render(<ExplorerTab range={rangeBefore} onRangeChange={onRangeChange} snapshots={[]} sessions={[]} />);
      fireEvent.click(screen.getByRole("button", { name: "Today" }));
      view.rerender(<ExplorerTab range={rangeToday} onRangeChange={onRangeChange} snapshots={[]} sessions={[]}
        openDayRequest={{ day: "2026-06-15", requestId: "main-window-history" }} />);
      expect(screen.getByRole("tab", { name: /2026-06-15/ })).toHaveAttribute("aria-selected", "true");
      expect(screen.getByRole("tab", { name: /2026-06-20/ })).toHaveAttribute("aria-selected", "false");
    } finally {
      vi.useRealTimers();
      if (originalTimeZone === undefined) vi.unstubAllEnvs();
      else process.env.TZ = originalTimeZone;
    }
  });

  it("reads the current local day immediately instead of waiting for the minute ticker", () => {
    const originalTimeZone = process.env.TZ;
    vi.stubEnv("TZ", "Europe/Warsaw");
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-06-20T21:59:00Z")); // 23:59 in Warsaw.
    const onRangeChange = vi.fn();
    const rangeBefore = { from: "2026-06-01", to: "2026-06-10" };
    const rangeAfter = { from: "2026-06-12", to: "2026-06-21" };
    try {
      const view = render(<ExplorerTab range={rangeBefore} onRangeChange={onRangeChange} snapshots={[]} sessions={[]} />);
      vi.setSystemTime(new Date("2026-06-20T22:01:00Z")); // 00:01; no interval tick elapsed.
      fireEvent.click(screen.getByRole("button", { name: "Today" }));
      expect(onRangeChange).toHaveBeenCalledWith(rangeAfter);
      view.rerender(<ExplorerTab range={rangeAfter} onRangeChange={onRangeChange} snapshots={[]} sessions={[]} />);

      vi.setSystemTime(new Date("2026-06-20T21:59:00Z")); // Clock rollback before the next minute tick.
      fireEvent.click(screen.getByRole("button", { name: "Today" }));
      expect(screen.getByRole("tab", { name: /2026-06-20/ })).toHaveAttribute("aria-selected", "true");
      expect(screen.getByRole("tab", { name: /2026-06-21/ })).toHaveAttribute("aria-selected", "false");
    } finally {
      vi.useRealTimers();
      if (originalTimeZone === undefined) vi.unstubAllEnvs();
      else process.env.TZ = originalTimeZone;
    }
  });

  it("contains real-shaped history conflicts, preserves navigation and Retry, and recovers on a clean range", async () => {
    coreQueryMock.sessions = { ...coreQueryMock.sessions, status: "ready", data: conflict };
    const onRangeChange = vi.fn();
    const { rerender } = render(<ExplorerTab range={range} onRangeChange={onRangeChange} />);
    expect(screen.getByRole("tabpanel")).toHaveTextContent(/History unavailable: Conflicting overlapping intervals/);
    expect(screen.queryByTestId("timeline-detail")).toBeNull();
    expect(screen.queryByText(/Score and break-credit details/)).toBeNull();
    for (const ring of screen.getAllByTestId(/^day-ring-/)) {
      expect(ring.tagName).toBe("SPAN");
      expect(ring).toHaveAccessibleName(/composition unavailable/);
      expect(ring).not.toHaveAccessibleName(/No observations recorded|100.00 percent|0.000 seconds/);
    }
    fireEvent.click(screen.getByRole("tab", { name: /2026-05-16/ }));
    expect(screen.getByRole("tab", { name: /2026-05-16/ })).toHaveAttribute("aria-selected", "true");
    fireEvent.click(screen.getByRole("button", { name: "Previous day" }));
    expect(screen.getByRole("tab", { name: /2026-05-15/ })).toHaveAttribute("aria-selected", "true");
    fireEvent.click(screen.getByRole("button", { name: "Retry loading Analyst data" }));
    expect(coreQueryMock.sessions.refetch).toHaveBeenCalledOnce();
    expect(coreQueryMock.snapshots.refetch).toHaveBeenCalledOnce();
    fireEvent.click(screen.getByTestId("preset-7d"));
    expect(onRangeChange).toHaveBeenCalled();

    coreQueryMock.sessions = { ...coreQueryMock.sessions, status: "ready", data: [conflict[0]] };
    rerender(<ExplorerTab range={{ from: "2026-05-16", to: "2026-05-16" }} onRangeChange={onRangeChange} />);
    await waitFor(() => expect(screen.getByTestId("timeline-detail")).toBeInTheDocument());
    expect(screen.queryAllByText(/History unavailable/)).toHaveLength(0);
    expect(screen.getByTestId("day-ring-2026-05-16").tagName.toLowerCase()).toBe("svg");
    expect(screen.getAllByTestId("timeline-segment")).toHaveLength(1);
  });

  it("reports invalid interval endpoints without fabricating an empty timeline", () => {
    render(<ExplorerTab range={range} onRangeChange={() => undefined} snapshots={[]}
      sessions={[{ ...conflict[0], endedAt: "invalid" }]} />);
    expect(screen.getByRole("tabpanel")).toHaveTextContent(/History unavailable: Invalid interval endpoints/);
    expect(screen.queryByTestId("timeline-detail")).toBeNull();
    expect(screen.getByTestId("day-ring-2026-05-17").tagName).toBe("SPAN");
  });

  it("withholds composition while required data loads and restores cached history after a failed refresh", () => {
    coreQueryMock.sessions = { ...coreQueryMock.sessions, status: "ready", data: [conflict[0]] };
    const { rerender } = render(<ExplorerTab range={range} onRangeChange={() => undefined} />);
    expect(screen.getByTestId("timeline-detail")).toBeInTheDocument();
    coreQueryMock.snapshots = { status: "loading", data: null, error: null, refetch: vi.fn(), lastRefreshed: 123 };
    rerender(<ExplorerTab range={range} onRangeChange={() => undefined} />);
    expect(screen.queryByTestId("timeline-detail")).toBeNull();
    expect(screen.getByRole("tabpanel")).toHaveAttribute("aria-busy", "true");
    expect(screen.getByTestId("day-ring-2026-05-17")).toHaveAccessibleName(/composition loading/);
    coreQueryMock.snapshots = { ...coreQueryMock.snapshots, status: "error", data: null, error: "snapshot query failed" };
    rerender(<ExplorerTab range={range} onRangeChange={() => undefined} />);
    expect(screen.getByTestId("timeline-detail")).toBeInTheDocument();
    expect(screen.queryByText(/Score and break-credit details/)).toBeNull();
    expect(screen.getByTestId("day-ring-2026-05-17")).toHaveAccessibleName(/composition unavailable/);
    fireEvent.click(screen.getByRole("button", { name: "Retry loading Analyst data" }));
    expect(coreQueryMock.snapshots.refetch).toHaveBeenCalledOnce();
    coreQueryMock.snapshots = { ...coreQueryMock.snapshots, status: "ready", data: [] };
    rerender(<ExplorerTab range={range} onRangeChange={() => undefined} />);
    expect(screen.getByTestId("timeline-detail")).toBeInTheDocument();
  });
  it("sends an in-range historical request to useTimelineNav and acknowledges after that day is selected", async () => {
    const acknowledge = vi.fn();
    render(<ExplorerTab range={range} onRangeChange={() => undefined} snapshots={snapshots} sessions={sessions}
      openDayRequest={{ day: "2026-05-16", requestId: "history-16" }} onOpenDayHandled={acknowledge} />);
    await waitFor(() => expect(screen.getByRole("region", { name: "Daily timeline for 2026-05-16" })).toBeInTheDocument());
    expect(screen.getByRole("tab", { name: /2026-05-16/ })).toHaveAttribute("aria-selected", "true");
    expect(acknowledge).toHaveBeenCalledWith("history-16");
  });

  it("renders one donut for each date and keeps score and credit in collapsed details", () => {
    render(<ExplorerTab range={range} onRangeChange={() => undefined} snapshots={snapshots} sessions={sessions} />);
    expect(screen.getByTestId("analyst-header")).toBeInTheDocument();
    expect(screen.queryByTestId("tl-weekday")).toBeNull();
    expect(screen.getByTestId("explorer-tab")).toHaveClass("analyst-local-theme");
    expect(screen.getByTestId("range-from")).toBeInTheDocument();
    expect(screen.getByTestId("timeline-detail")).toBeInTheDocument();
    expect(screen.getByTestId("date-navigator").querySelectorAll('[role="tab"]')).toHaveLength(3);
    expect(screen.getByTestId("date-navigator").querySelectorAll('[data-testid^="day-ring-"]')).toHaveLength(3);
    expect(screen.getByText(/Score and break-credit details/).closest("details")).not.toHaveAttribute("open");
    expect(screen.queryByTestId("kpi-donut-panel")).toBeNull();
    expect(screen.queryByTestId("desk-height-timeline")).toBeNull();
  });

  it("grows the selected-day live interval on a new observation without a minute-tick advance", () => {
    vi.useFakeTimers();
    const startAt = new Date(2026, 9, 25, 10, 0, 0).getTime();
    vi.setSystemTime(startAt + 5_000);
    const today = localCalendarDay(new Date());
    const observation = (throughMs: number): LiveObservation => ({
      span: {
        state: "Standing",
        started_at: new Date(startAt).toISOString(),
        observed_through: new Date(throughMs).toISOString(),
      },
      last_observation_at: new Date(throughMs).toISOString(),
      snapshot_at: new Date(throughMs).toISOString(),
      quality: "known",
      reason: null,
    });
    try {
      const view = render(<ExplorerTab range={{ from: today, to: today }} onRangeChange={() => undefined}
        snapshots={[]} sessions={[]} liveObservation={observation(startAt + 5_000)}
        dashboardRefresh={{ loading: false, lastSuccessAt: new Date(startAt + 5_000).toISOString(), error: null }} />);
      expect(screen.getByTestId("timeline-segment")).toHaveAttribute("data-end-ms", String(startAt + 5_000));

      vi.setSystemTime(startAt + 6_000);
      view.rerender(<ExplorerTab range={{ from: today, to: today }} onRangeChange={() => undefined}
        snapshots={[]} sessions={[]} liveObservation={observation(startAt + 6_000)}
        dashboardRefresh={{ loading: false, lastSuccessAt: new Date(startAt + 6_000).toISOString(), error: null }} />);

      expect(screen.getByTestId("timeline-segment")).toHaveAttribute("data-end-ms", String(startAt + 6_000));
    } finally {
      vi.useRealTimers();
    }
  });

  it("does not let a future producer snapshot move the Analyst's current day", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-25T10:21:05Z"));
    const observation: LiveObservation = {
      span: { state: "Sitting", started_at: "2026-10-25T10:00:00Z", observed_through: "2026-10-25T10:20:00Z" },
      last_observation_at: "2026-10-25T10:20:00Z",
      snapshot_at: "2026-10-26T12:00:00Z",
      quality: null,
      reason: "stale",
    };
    const onRangeChange = vi.fn();
    try {
      render(<ExplorerTab range={{ from: "2026-10-25", to: "2026-10-25" }} onRangeChange={onRangeChange}
        snapshots={[]} sessions={[]} liveObservation={observation}
        dashboardRefresh={{ loading: false, lastSuccessAt: "2026-10-25T10:21:05Z", error: null }} />);
      fireEvent.click(screen.getByTestId("preset-7d"));
      expect(onRangeChange).toHaveBeenCalledWith({ from: "2026-10-19", to: "2026-10-25" });
    } finally {
      vi.useRealTimers();
    }
  });

  it("uses the client clock after a rollback across local midnight on success and the next failure tick", () => {
    const originalTimeZone = process.env.TZ;
    vi.stubEnv("TZ", "Europe/Warsaw");
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-27T00:01:00Z"));
    const onRangeChange = vi.fn();
    const propsAt = (lastSuccessAt: string, error: string | null) => ({
      range: { from: "2026-10-20", to: "2026-10-27" }, onRangeChange, snapshots: [], sessions: [],
      dashboardRefresh: { loading: false, lastSuccessAt, error },
    });
    try {
      const view = render(<ExplorerTab {...propsAt("2026-10-27T00:01:00Z", null)} />);
      vi.setSystemTime(new Date("2026-10-26T22:59:00Z"));
      view.rerender(<ExplorerTab {...propsAt("2026-10-26T22:59:00Z", null)} />);
      fireEvent.click(screen.getByTestId("preset-7d"));
      expect(onRangeChange).toHaveBeenLastCalledWith({ from: "2026-10-20", to: "2026-10-26" });

      vi.setSystemTime(new Date("2026-10-26T22:50:00Z"));
      view.rerender(<ExplorerTab {...propsAt("2026-10-26T22:59:00Z", "dashboard refresh failed")} />);
      act(() => vi.advanceTimersByTime(60_000));
      fireEvent.click(screen.getByTestId("preset-7d"));
      expect(onRangeChange).toHaveBeenLastCalledWith({ from: "2026-10-20", to: "2026-10-26" });
    } finally {
      vi.useRealTimers();
      if (originalTimeZone === undefined) vi.unstubAllEnvs();
      else process.env.TZ = originalTimeZone;
    }
  });

  it("reconciles the automatic selected Analyst day immediately across 02:00 on success and failure, preserving manual history", () => {
    const originalTimeZone = process.env.TZ;
    vi.stubEnv("TZ", "Europe/Warsaw");
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-27T01:01:00Z")); // 02:01 local: automatic day is October 27.
    const onRangeChange = vi.fn();
    const propsAt = (lastSuccessAt: string, error: string | null) => ({
      range: { from: "2026-10-20", to: "2026-10-27" }, onRangeChange, snapshots: [], sessions: [],
      dashboardRefresh: { loading: false, lastSuccessAt, error },
    });
    try {
      const view = render(<ExplorerTab {...propsAt("2026-10-27T01:01:00Z", null)} />);
      expect(screen.getByRole("region", { name: "Daily timeline for 2026-10-27" })).toBeInTheDocument();

      vi.setSystemTime(new Date("2026-10-27T00:59:00Z")); // 01:59 local: automatic day returns to October 26.
      view.rerender(<ExplorerTab {...propsAt("2026-10-27T00:59:00Z", null)} />);
      expect(screen.getByRole("region", { name: "Daily timeline for 2026-10-26" })).toBeInTheDocument();
      expect(screen.getByRole("tab", { name: /2026-10-26/ })).toHaveAttribute("aria-selected", "true");

      vi.setSystemTime(new Date("2026-10-27T01:01:00Z"));
      view.rerender(<ExplorerTab {...propsAt("2026-10-27T01:01:00Z", null)} />);
      vi.setSystemTime(new Date("2026-10-27T00:59:00Z"));
      view.rerender(<ExplorerTab {...propsAt("2026-10-27T01:01:00Z", "dashboard refresh failed")} />);
      expect(screen.getByRole("region", { name: "Daily timeline for 2026-10-26" })).toBeInTheDocument();

      fireEvent.click(screen.getByRole("tab", { name: /2026-10-25/ }));
      vi.setSystemTime(new Date("2026-10-27T01:01:00Z"));
      view.rerender(<ExplorerTab {...propsAt("2026-10-27T01:01:00Z", null)} />);
      expect(screen.getByRole("region", { name: "Daily timeline for 2026-10-25" })).toBeInTheDocument();
      expect(screen.getByRole("tab", { name: /2026-10-25/ })).toHaveAttribute("aria-selected", "true");
    } finally {
      vi.useRealTimers();
      if (originalTimeZone === undefined) vi.unstubAllEnvs();
      else process.env.TZ = originalTimeZone;
    }
  });

  it("keeps routine dashboard request churn out of the polite observation announcement", () => {
    const observation: LiveObservation = {
      span: { state: "Sitting", started_at: "2026-05-17T10:00:00Z", observed_through: "2026-05-17T10:20:00Z" },
      last_observation_at: "2026-05-17T10:20:00Z",
      snapshot_at: "2026-05-17T10:20:00Z",
      quality: "known",
      reason: null,
    };
    const props = { range, onRangeChange: () => undefined, snapshots: [], sessions: [], liveObservation: observation };
    const view = render(<ExplorerTab {...props} dashboardRefresh={{ loading: true, lastSuccessAt: null, error: null }} />);
    const announcement = screen.getByTestId("live-observation-announcement");
    const initial = announcement.textContent;
    view.rerender(<ExplorerTab {...props} dashboardRefresh={{ loading: false, lastSuccessAt: "2026-05-17T12:00:00Z", error: null }} />);
    expect(screen.getByTestId("live-observation-announcement").textContent).toBe(initial);
    expect(screen.getByTestId("view-response-details")).toHaveTextContent("View response received");
    expect(screen.getByTestId("view-response-details")).toHaveAttribute("aria-live", "off");
  });

  it("clips the live interval at the selected Warsaw local-day boundary", () => {
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
      render(<ExplorerTab range={{ from: "2026-10-26", to: "2026-10-26" }} onRangeChange={() => undefined}
        snapshots={[]} sessions={[]} liveObservation={observation}
        dashboardRefresh={{ loading: false, lastSuccessAt: "2026-10-27T00:03:00+01:00", error: null }} />);
      const segment = screen.getByTestId("timeline-segment");
      expect(segment).toHaveAttribute("data-start-ms", String(Date.parse("2026-10-26T22:58:00Z")));
      expect(segment).toHaveAttribute("data-end-ms", String(Date.parse("2026-10-26T23:00:00Z")));
    } finally {
      vi.useRealTimers();
      if (originalTimeZone === undefined) vi.unstubAllEnvs();
      else process.env.TZ = originalTimeZone;
    }
  });

  it("keeps the closed posture prefix through the transition and failed history refresh", () => {
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
    const row: SessionRow = {
      id: 101,
      startedAt: oldObservation.span!.started_at, endedAt: oldObservation.span!.observed_through,
      state: "Sitting", durationSecs: 1200, sittingSecs: 1200, standingSecs: 0, positionChanges: 0,
      breakCredit: null, dateLocal: "2026-10-25", historyKind: "individual",
    };
    try {
      const view = render(<ExplorerTab range={{ from: "2026-10-25", to: "2026-10-25" }} onRangeChange={() => undefined}
        snapshots={[]} sessions={[]} liveObservation={oldObservation} />);
      view.rerender(<ExplorerTab range={{ from: "2026-10-25", to: "2026-10-25" }} onRangeChange={() => undefined}
        snapshots={[]} sessions={[]} liveObservation={nextObservation} />);
      expect(screen.getAllByTestId("timeline-segment")).toHaveLength(2);
      expect(screen.getAllByTestId("timeline-segment")[0]).toHaveAttribute("data-start-ms", String(Date.parse(row.startedAt)));

      view.rerender(<ExplorerTab range={{ from: "2026-10-25", to: "2026-10-25" }} onRangeChange={() => undefined}
        snapshots={[]} sessions={[]} liveObservation={nextObservation} dashboardRefresh={{ loading: false, lastSuccessAt: null, error: "history refresh failed" }} />);
      expect(screen.getAllByTestId("timeline-segment")).toHaveLength(2);
      view.rerender(<ExplorerTab range={{ from: "2026-10-25", to: "2026-10-25" }} onRangeChange={() => undefined}
        snapshots={[]} sessions={[row]} liveObservation={nextObservation} />);
      expect(screen.getAllByTestId("timeline-segment")).toHaveLength(2);
      expect(screen.getByTestId("live-observation-status")).toHaveTextContent("Observed Standing through");
    } finally {
      vi.useRealTimers();
    }
  });

  it("keeps last successful history visible when the live range query fails", () => {
    const row: SessionRow = {
      id: 102,
      startedAt: "2026-05-17T10:00:00Z", endedAt: "2026-05-17T10:20:00Z",
      state: "Sitting", durationSecs: 1200, sittingSecs: 1200, standingSecs: 0, positionChanges: 0,
      breakCredit: null, dateLocal: "2026-05-17", historyKind: "individual",
    };
    coreQueryMock.snapshots.status = "ready";
    coreQueryMock.snapshots.data = [];
    coreQueryMock.snapshots.lastRefreshed = "2026-05-17T12:00:00Z";
    coreQueryMock.sessions.status = "ready";
    coreQueryMock.sessions.data = [row];
    coreQueryMock.sessions.lastRefreshed = "2026-05-17T12:00:00Z";
    const props = { range, onRangeChange: () => undefined };
    const view = render(<ExplorerTab {...props} />);
    expect(screen.getAllByTestId("timeline-segment")[0]).toHaveAttribute("data-start-ms", String(Date.parse(row.startedAt)));

    coreQueryMock.snapshots.status = "error";
    coreQueryMock.snapshots.error = "snapshot refresh failed";
    coreQueryMock.snapshots.data = [] as unknown as SnapshotRow[];
    coreQueryMock.sessions.status = "error";
    coreQueryMock.sessions.error = "history refresh failed";
    coreQueryMock.sessions.data = [] as unknown as SessionRow[];
    view.rerender(<ExplorerTab {...props} />);
    expect(screen.getAllByTestId("timeline-segment")[0]).toHaveAttribute("data-start-ms", String(Date.parse(row.startedAt)));
    expect(screen.getByText(/Failed: snapshot refresh failed.*showing last successful history/)).toBeInTheDocument();

    coreQueryMock.snapshots.status = "ready";
    coreQueryMock.snapshots.error = null;
    coreQueryMock.snapshots.data = [];
    coreQueryMock.sessions.status = "ready";
    coreQueryMock.sessions.error = null;
    coreQueryMock.sessions.data = [];
    view.unmount();
  });

  it("does not acknowledge a request until its date belongs to the active range", () => {
    const acknowledge = vi.fn();
    render(<ExplorerTab range={range} onRangeChange={() => undefined} snapshots={snapshots} sessions={sessions}
      openDayRequest={{ day: "2026-05-20", requestId: "outside" }} onOpenDayHandled={acknowledge} />);
    expect(acknowledge).not.toHaveBeenCalled();
  });

  it("selects a day through the date tab and retains the request-independent nav behavior", async () => {
    render(<ExplorerTab range={range} onRangeChange={() => undefined} snapshots={snapshots} sessions={sessions} />);
    fireEvent.click(screen.getByRole("tab", { name: /2026-05-16/ }));
    await waitFor(() => expect(screen.getByRole("region", { name: "Daily timeline for 2026-05-16" })).toBeInTheDocument());
  });

  it("keeps the existing range preset wired through the relocated header", () => {
    const onRangeChange = vi.fn();
    render(<ExplorerTab range={range} onRangeChange={onRangeChange} snapshots={snapshots} sessions={sessions} />);
    fireEvent.click(screen.getByTestId("preset-7d"));
    expect(onRangeChange).toHaveBeenCalled();
  });

  it("keeps core day composition ready when the supplemental events query fails", () => {
    render(<ExplorerTab range={range} onRangeChange={() => undefined} />);
    expect(screen.getByRole("alert")).toHaveTextContent("Events unavailable: event source failed");
    expect(screen.getByTestId("day-ring-2026-05-17").getAttribute("aria-label")).toContain("Unknown 100.00 percent");
    expect(screen.getByRole("tabpanel")).toHaveAttribute("aria-busy", "false");
    fireEvent.click(screen.getByRole("button", { name: "Retry events" }));
    expect(eventQueryMock.refetch).toHaveBeenCalledTimes(1);
  });
});
