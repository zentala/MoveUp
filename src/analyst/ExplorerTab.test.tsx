import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { SessionRow, SnapshotRow } from "@/test/analyst-fixtures";
import type { RangeQueryState } from "./hooks/useRangeQuery";
import { ExplorerTab } from "./ExplorerTab";

const eventQueryMock = vi.hoisted(() => ({ refetch: vi.fn() }));
const coreQueryMock = vi.hoisted(() => ({
  snapshots: {} as RangeQueryState<SnapshotRow>,
  sessions: {} as RangeQueryState<SessionRow>,
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
  coreQueryMock.snapshots = { status: "ready", data: [], refetch: vi.fn(), lastRefreshed: null };
  coreQueryMock.sessions = { status: "ready", data: [], refetch: vi.fn(), lastRefreshed: null };
});

describe("ExplorerTab day assembly", () => {
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

  it("withholds ready composition while required data loads or fails, then recovers after retry", () => {
    coreQueryMock.sessions = { ...coreQueryMock.sessions, status: "ready", data: [conflict[0]] };
    const { rerender } = render(<ExplorerTab range={range} onRangeChange={() => undefined} />);
    expect(screen.getByTestId("timeline-detail")).toBeInTheDocument();
    coreQueryMock.snapshots = { status: "loading", data: null, refetch: vi.fn(), lastRefreshed: 123 };
    rerender(<ExplorerTab range={range} onRangeChange={() => undefined} />);
    expect(screen.queryByTestId("timeline-detail")).toBeNull();
    expect(screen.getByRole("tabpanel")).toHaveAttribute("aria-busy", "true");
    expect(screen.getByTestId("day-ring-2026-05-17")).toHaveAccessibleName(/composition loading/);
    coreQueryMock.snapshots = { ...coreQueryMock.snapshots, status: "error", data: null, error: "snapshot query failed" };
    rerender(<ExplorerTab range={range} onRangeChange={() => undefined} />);
    expect(screen.queryByTestId("timeline-detail")).toBeNull();
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
