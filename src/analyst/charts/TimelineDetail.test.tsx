/**
 * TimelineDetail.test.tsx — smoke tests for the continuous timeline strip.
 */
import { act, fireEvent, render, screen } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { LocalDayBounds } from "@/generated/LocalDayBounds";
import type { LocalHourTick } from "@/generated/LocalHourTick";
import type { SnapshotRow } from "@/test/analyst-fixtures";
import { TimelineDetail } from "./TimelineDetail";

function snap(ts: string, state: SnapshotRow["state"]): SnapshotRow {
  return {
    ts,
    state,
    deskHeightCm: 75,
    sittingSecs: 0,
    standingSecs: 0,
    breakSecs: 0,
    idleSecs: 0,
    score: 50,
  };
}

const range = { from: "2026-05-16", to: "2026-05-17" };
const snapshots = [
  snap("2026-05-17T08:00:00Z", "Sitting"),
  snap("2026-05-17T09:00:00Z", "Standing"),
];

function bounds(from: string, to: string): LocalDayBounds[] {
  const first = Date.parse(`${from}T00:00:00Z`);
  const last = Date.parse(`${to}T00:00:00Z`);
  return Array.from({ length: Math.floor((last - first) / 86_400_000) + 1 }, (_, index) => {
    const start = first + index * 86_400_000;
    const hour_ticks: LocalHourTick[] = Array.from({ length: 24 }, (_, hour) => ({
      at_utc: new Date(start + hour * 3_600_000).toISOString(),
      wall_time: `${String(hour).padStart(2, "0")}:00`,
      utc_offset_secs: 0,
    }));
    return {
      date_local: new Date(start).toISOString().slice(0, 10),
      start_utc: new Date(start).toISOString(),
      end_utc: new Date(start + 86_400_000).toISOString(),
      zone_basis: "current-query-local",
      historical_source_zone: "unknown",
      hour_ticks,
    };
  });
}

beforeEach(() => {
  vi.useFakeTimers();
  vi.mocked(invoke).mockReset().mockImplementation((_command, args) => {
    const rangeArgs = args as { from: string; to: string };
    return Promise.resolve(bounds(rangeArgs.from, rangeArgs.to));
  });
});

class ResizeObserverMock {
  static latest: ResizeObserverMock | null = null;
  private target: Element | null = null;
  constructor(private readonly callback: ResizeObserverCallback) { ResizeObserverMock.latest = this; }
  observe(target: Element): void { this.target = target; }
  disconnect(): void {}
  unobserve(): void {}
  trigger(width: number): void {
    if (!this.target) return;
    this.callback([{ target: this.target, contentRect: { width } as DOMRectReadOnly } as ResizeObserverEntry], this as unknown as ResizeObserver);
  }
}

afterEach(() => { vi.useRealTimers(); vi.unstubAllGlobals(); ResizeObserverMock.latest = null; });

async function loadBounds() {
  await act(async () => { await vi.advanceTimersByTimeAsync(250); });
}

describe("TimelineDetail", () => {
  it("renders the active weekday and date in header after query bounds load", async () => {
    render(
      <TimelineDetail
        range={range}
        selectedDay="2026-05-17"
        snapshots={snapshots}
        onPrev={vi.fn()}
        onNext={vi.fn()}
        nowMs={new Date("2026-05-17T14:30:00Z").getTime()}
      />,
    );
    await loadBounds();
    // 2026-05-17 is a Sunday (Date locale dependent — accept any weekday).
    expect(screen.getByTestId("tl-weekday").textContent?.length ?? 0).toBeGreaterThan(0);
  });

  it("fires onPrev/onNext from nav buttons", async () => {
    const onPrev = vi.fn();
    const onNext = vi.fn();
    render(
      <TimelineDetail
        range={range}
        selectedDay="2026-05-17"
        snapshots={snapshots}
        onPrev={onPrev}
        onNext={onNext}
      />,
    );
    await loadBounds();
    fireEvent.click(screen.getByTestId("tl-prev"));
    fireEvent.click(screen.getByTestId("tl-next"));
    expect(onPrev).toHaveBeenCalledTimes(1);
    expect(onNext).toHaveBeenCalledTimes(1);
  });

  it("mounts the scrollable strip container", async () => {
    render(
      <TimelineDetail
        range={range}
        selectedDay="2026-05-17"
        snapshots={snapshots}
        onPrev={vi.fn()}
        onNext={vi.fn()}
      />,
    );
    await loadBounds();
    expect(screen.getByTestId("timeline-scroller")).toBeTruthy();
  });

  it("renders only authoritative supplied hour ticks and keeps bounds when an old map is absent", async () => {
    const start = "2026-05-16T00:00:00Z";
    const end = "2026-05-17T00:00:00Z";
    vi.mocked(invoke).mockReset().mockResolvedValueOnce([{
      date_local: "2026-05-16", start_utc: start, end_utc: end,
      zone_basis: "current-query-local", historical_source_zone: "unknown",
      hour_ticks: [
        { at_utc: "2026-05-16T00:00:00Z", wall_time: "00:00", utc_offset_secs: 0 },
        { at_utc: "2026-05-16T01:00:00Z", wall_time: "01:00", utc_offset_secs: 0 },
      ],
    }]);
    const firstView = render(<TimelineDetail range={{ from: "2026-05-16", to: "2026-05-16" }} selectedDay="2026-05-16"
      snapshots={[]} onPrev={vi.fn()} onNext={vi.fn()} nowMs={Date.parse("2026-05-16T12:00:00Z")} />);
    await loadBounds();
    expect(screen.getByTestId("timeline-scroller")).toHaveAttribute("data-window-start-ms", String(Date.parse(start)));
    expect(screen.getByTestId("hour-ticks").querySelectorAll("line")).toHaveLength(2);
    firstView.unmount();

    vi.mocked(invoke).mockReset().mockResolvedValueOnce([{
      date_local: "2026-05-16", start_utc: start, end_utc: end,
      zone_basis: "current-query-local", historical_source_zone: "unknown",
    }]);
    const { unmount } = render(<TimelineDetail range={{ from: "2026-05-16", to: "2026-05-16" }} selectedDay="2026-05-16"
      snapshots={[]} onPrev={vi.fn()} onNext={vi.fn()} nowMs={Date.parse("2026-05-16T12:00:00Z")} />);
    await loadBounds();
    expect(screen.getByTestId("timeline-scroller")).toHaveAttribute("data-window-start-ms", String(Date.parse(start)));
    expect(screen.getByTestId("hour-axis-unavailable")).toBeInTheDocument();
    unmount();
  });

  it("labels both occurrences of a repeated local hour with their supplied offsets", async () => {
    vi.stubGlobal("ResizeObserver", ResizeObserverMock);
    vi.mocked(invoke).mockReset().mockResolvedValueOnce([{
      date_local: "2026-11-01",
      start_utc: "2026-11-01T04:00:00Z",
      end_utc: "2026-11-02T05:00:00Z",
      zone_basis: "current-query-local",
      historical_source_zone: "unknown",
      hour_ticks: [
        { at_utc: "2026-11-01T04:00:00Z", wall_time: "00:00", utc_offset_secs: -14_400 },
        { at_utc: "2026-11-01T05:00:00Z", wall_time: "01:00", utc_offset_secs: -14_400 },
        { at_utc: "2026-11-01T06:00:00Z", wall_time: "01:00", utc_offset_secs: -18_000 },
      ],
    }]);
    render(<TimelineDetail range={{ from: "2026-11-01", to: "2026-11-01" }} selectedDay="2026-11-01"
      snapshots={[]} onPrev={vi.fn()} onNext={vi.fn()} nowMs={Date.parse("2026-11-01T12:00:00Z")} />);
    await loadBounds();
    const scroller = screen.getByTestId("timeline-scroller");
    Object.defineProperty(scroller, "clientWidth", { configurable: true, value: 760 });
    act(() => ResizeObserverMock.latest?.trigger(760));
    const labels = [...screen.getByTestId("hour-ticks").querySelectorAll("text")];
    expect(labels.map((node) => node.getAttribute("aria-label"))).toEqual(["00:00", "01:00 −04:00", "01:00 −05:00"]);
    expect(labels[0]).toHaveAttribute("text-anchor", "start");
    expect(labels[1]).toHaveAttribute("data-at-utc", "2026-11-01T05:00:00Z");
    expect(labels[1]).toHaveAttribute("data-utc-offset-secs", "-14400");
    expect(labels[2]).toHaveAttribute("data-at-utc", "2026-11-01T06:00:00Z");
    expect(labels[2]).toHaveAttribute("data-utc-offset-secs", "-18000");
    expect(labels[1].getAttribute("data-label-row")).not.toBe(labels[2].getAttribute("data-label-row"));
    const boxes = labels.map((node) => {
      const x = Number(node.getAttribute("x"));
      const width = (node.getAttribute("aria-label")?.length ?? 0) * 6;
      const anchor = node.getAttribute("text-anchor");
      const left = anchor === "start" ? x : anchor === "end" ? x - width : x - width / 2;
      return { row: node.getAttribute("data-label-row"), left, right: left + width };
    });
    expect(boxes.every((box) => box.left >= 0 && box.right <= 761)).toBe(true);
    for (const row of new Set(boxes.map((box) => box.row))) {
      const rowBoxes = boxes.filter((box) => box.row === row).sort((a, b) => a.left - b.left);
      expect(rowBoxes.every((box, index) => index === 0 || rowBoxes[index - 1].right <= box.left + 1)).toBe(true);
    }
    expect([...screen.getByTestId("timeline-legend").querySelectorAll("li")].map((item) => item.textContent)).toEqual([
      "Sitting", "Standing", "Walking", "Away", "Unknown", "Future", "Observation points",
    ]);
  });

  it("clips rendered future intervals at the injected current instant without mutating source endpoints", async () => {
    const now = Date.parse("2026-05-17T12:00:00Z");
    const crossing = { startMs: now - 10 * 60_000, endMs: now + 10 * 60_000, state: "Standing" as const, provenance: "session" as const };
    const future = { startMs: now + 60 * 60_000, endMs: now + 2 * 60 * 60_000, state: "Walking" as const, provenance: "session" as const };
    render(<TimelineDetail range={{ from: "2026-05-17", to: "2026-05-17" }} selectedDay="2026-05-17"
      snapshots={[]} intervals={[crossing, future]} onPrev={vi.fn()} onNext={vi.fn()} nowMs={now} />);
    await loadBounds();
    const segments = screen.getAllByTestId("timeline-segment");
    expect(segments).toHaveLength(1);
    expect(segments[0]).toHaveAttribute("data-start-ms", String(crossing.startMs));
    expect(segments[0]).toHaveAttribute("data-end-ms", String(now));
    expect(screen.getAllByTestId("timeline-future")).toHaveLength(1);
    expect(future.startMs).toBe(now + 60 * 60_000);
    expect(crossing.endMs).toBe(now + 10 * 60_000);
  });

  it("keeps snapshot readings as 2px point ticks and paints exact intervals only", async () => {
    const at = new Date("2026-05-17T10:00:00Z").getTime();
    const sourceSnapshots = [{
      ...snap(new Date(at).toISOString(), "Standing"),
      observationQuality: "known",
      connected: true,
      localDate: "2026-05-17",
      timezoneOffsetSecs: 0,
    }];
    const sessionStart = new Date("2026-05-17T09:00:00Z").getTime();
    const sessionEnd = new Date("2026-05-17T09:10:00Z").getTime();
    render(
      <TimelineDetail
        range={{ from: "2026-05-17", to: "2026-05-17" }}
        selectedDay="2026-05-17"
        snapshots={sourceSnapshots}
        intervals={[{ startMs: sessionStart, endMs: sessionEnd, state: "Sitting", provenance: "session" }]}
        onPrev={vi.fn()}
        onNext={vi.fn()}
        nowMs={new Date("2026-05-17T12:00:00Z").getTime()}
      />,
    );
    await loadBounds();
    expect(screen.getAllByTestId("timeline-segment")).toHaveLength(1);
    expect(screen.getByTestId("timeline-segment")).toHaveAttribute("data-start-ms", String(sessionStart));
    const marker = screen.getByTestId("timeline-observation-marker");
    expect(marker).toHaveAttribute("width", "2");
    expect(marker).toHaveAttribute("height", "6");
    expect(screen.getByText(/Observation ticks mark instants; duration unknown/)).toBeInTheDocument();
    expect(screen.getByRole("combobox", { name: "Timeline details" })).toHaveTextContent("duration unknown");
    fireEvent.change(screen.getByRole("combobox", { name: "Timeline details" }), {
      target: { value: `observation-${at}` },
    });
    expect(screen.getByTestId("timeline-observation-detail")).toHaveTextContent("duration unknown");
  });

  it("keeps an in-window future snapshot out of colored history and timeline details", async () => {
    const nowMs = Date.parse("2026-05-17T10:00:00Z");
    const sourceSnapshots = [
      {
        ...snap("2026-05-17T09:59:00Z", "Standing"),
        observationQuality: "known",
        connected: true,
        localDate: "2026-05-17",
        timezoneOffsetSecs: 0,
      },
      {
        ...snap("2026-05-17T10:01:00Z", "Walking"),
        observationQuality: "known",
        connected: true,
        localDate: "2026-05-17",
        timezoneOffsetSecs: 0,
      },
    ];
    render(
      <TimelineDetail
        range={{ from: "2026-05-17", to: "2026-05-17" }}
        selectedDay="2026-05-17"
        snapshots={sourceSnapshots}
        onPrev={vi.fn()}
        onNext={vi.fn()}
        nowMs={nowMs}
      />,
    );
    await loadBounds();
    const markers = screen.getAllByTestId("timeline-observation-marker");
    expect(markers).toHaveLength(1);
    expect(markers[0]).toHaveAttribute("data-at-ms", String(Date.parse("2026-05-17T09:59:00Z")));
    expect(markers[0]).toHaveAttribute("width", "2");
    expect(markers[0]).toHaveAttribute("height", "6");
    expect(screen.getByText(/Observation ticks mark instants; duration unknown/)).toBeInTheDocument();
    const details = screen.getByRole("combobox", { name: "Timeline details" });
    expect(details).toHaveTextContent("Observation instant 2026-05-17T09:59:00.000Z");
    expect(details).not.toHaveTextContent("2026-05-17T10:01:00.000Z");
    fireEvent.change(details, { target: { value: `observation-${Date.parse("2026-05-17T09:59:00Z")}` } });
    expect(screen.getByTestId("timeline-observation-detail")).toHaveTextContent("duration unknown");
  });

  it("leaves an empty date Unknown and clips intervals to explicit UTC bounds", async () => {
    const start = Date.parse("2026-05-16T23:50:00Z");
    const end = Date.parse("2026-05-17T00:10:00Z");
    render(
      <TimelineDetail
        range={{ from: "2026-05-17", to: "2026-05-17" }}
        selectedDay="2026-05-17"
        snapshots={[]}
        intervals={[{ startMs: start, endMs: end, state: "Standing", provenance: "session" }]}
        onPrev={vi.fn()}
        onNext={vi.fn()}
        nowMs={Date.parse("2026-05-18T00:00:00Z")}
      />,
    );
    await loadBounds();
    expect(screen.getByTestId("timeline-segment")).toHaveAttribute(
      "data-start-ms", String(Date.parse("2026-05-17T00:00:00Z")),
    );
    expect(screen.getByTestId("timeline-segment")).toHaveAttribute(
      "data-end-ms", String(Date.parse("2026-05-17T00:10:00Z")),
    );
    expect(screen.getAllByTestId("timeline-unknown").length).toBeGreaterThan(0);
    expect(screen.getByText(/current query-local timezone; historical capture timezone unknown/)).toBeInTheDocument();
  });

  it.each([
    { date: "2026-03-29", start: "2026-03-28T23:00:00.000Z", end: "2026-03-29T22:00:00.000Z", hours: 23 },
    { date: "2026-05-17", start: "2026-05-17T00:00:00.000Z", end: "2026-05-18T00:00:00.000Z", hours: 24 },
    { date: "2026-10-25", start: "2026-10-24T22:00:00.000Z", end: "2026-10-25T23:00:00.000Z", hours: 25 },
  ])("fits the exact $hours-hour query-local day to measured width and updates on resize", async ({ date, start, end, hours }) => {
    vi.stubGlobal("ResizeObserver", ResizeObserverMock);
    const startMs = Date.parse(start);
    const endMs = Date.parse(end);
    vi.mocked(invoke).mockReset().mockImplementation(() => Promise.resolve([{
      date_local: date,
      start_utc: start,
      end_utc: end,
      zone_basis: "current-query-local",
      historical_source_zone: "unknown",
    }]));
    render(<TimelineDetail range={{ from: date, to: date }} selectedDay={date} snapshots={[]}
      intervals={[{ startMs: startMs - 60_000, endMs: endMs + 60_000, state: "Standing", provenance: "session" }]}
      onPrev={vi.fn()} onNext={vi.fn()} nowMs={endMs + 60_000} />);
    await loadBounds();

    const scroller = screen.getByTestId("timeline-scroller");
    Object.defineProperty(scroller, "clientWidth", { configurable: true, value: 1222 });
    act(() => ResizeObserverMock.latest?.trigger(1222));
    const svg = screen.getByRole("img", { name: `Daily timeline ${date} to ${date}` });
    const segment = screen.getByTestId("timeline-segment");
    expect(Number(svg.getAttribute("width"))).toBe(1222);
    expect(svg).toHaveStyle({ display: "block" });
    expect(scroller).toHaveAttribute("data-window-start-ms", String(startMs));
    expect(scroller).toHaveAttribute("data-window-end-ms", String(endMs));
    expect(Number(segment.getAttribute("x"))).toBe(0);
    expect(Number(segment.getAttribute("width"))).toBeCloseTo(1222, 5);
    expect(segment).toHaveAttribute("data-start-ms", String(startMs));
    expect(segment).toHaveAttribute("data-end-ms", String(endMs));
    expect(screen.queryByTestId("timeline-unknown")).toBeNull();

    Object.defineProperty(scroller, "clientWidth", { configurable: true, value: 760 });
    act(() => ResizeObserverMock.latest?.trigger(760));
    expect(Number(svg.getAttribute("width"))).toBe(760);
    expect(Number(segment.getAttribute("width"))).toBeCloseTo(760, 5);
    expect(hours).toBe((endMs - startMs) / 3_600_000);
  });

  it("reserves the rendered strip geometry while real query-local bounds are loading", async () => {
    vi.stubGlobal("ResizeObserver", ResizeObserverMock);
    let resolveBounds: ((value: LocalDayBounds[]) => void) | undefined;
    vi.mocked(invoke).mockReset().mockImplementationOnce(() => new Promise((resolve) => {
      resolveBounds = resolve as (value: LocalDayBounds[]) => void;
    }));
    const start = Date.parse("2026-05-17T00:00:00Z");
    const end = Date.parse("2026-05-18T00:00:00Z");
    render(<TimelineDetail range={{ from: "2026-05-17", to: "2026-05-17" }}
      selectedDay="2026-05-17" snapshots={[]} intervals={[]} onPrev={vi.fn()} onNext={vi.fn()}
      nowMs={Date.parse("2026-05-17T12:00:00Z")} showHeader={false} />);

    expect(screen.getByRole("status")).toHaveTextContent("Loading query-local calendar bounds");
    const loadingSlot = screen.getByTestId("timeline-scroller");
    expect(loadingSlot).toHaveStyle({ height: "98px" });
    expect(loadingSlot.querySelector("svg")).toBeNull();
    expect(screen.queryByTestId("timeline-unknown")).toBeNull();
    expect(screen.queryByTestId("timeline-segment")).toBeNull();

    await act(async () => {
      await vi.advanceTimersByTimeAsync(250);
      resolveBounds?.([{
        date_local: "2026-05-17",
        start_utc: new Date(start).toISOString(),
        end_utc: new Date(end).toISOString(),
        zone_basis: "current-query-local",
        historical_source_zone: "unknown",
      }]);
    });

    const readySlot = screen.getByTestId("timeline-scroller");
    expect(readySlot).toBe(loadingSlot);
    expect(readySlot).toHaveStyle({ height: "98px" });
    expect(readySlot.style.height).not.toBe("84px");
    expect(readySlot.querySelector('[role="status"]')).toBeNull();
    expect(screen.getByTestId("hour-axis-unavailable")).toBeInTheDocument();
    expect(screen.getAllByTestId("timeline-unknown").length).toBeGreaterThan(0);
  });

  it("shows pending and error states without browser-local timeline fallback", async () => {
    vi.mocked(invoke).mockReset().mockRejectedValueOnce(new Error("query failed"));
    const { rerender } = render(
      <TimelineDetail range={range} selectedDay="2026-05-17" snapshots={[]} onPrev={vi.fn()} onNext={vi.fn()} />,
    );
    expect(screen.getByRole("status")).toHaveTextContent("Loading query-local calendar bounds");
    await loadBounds();
    expect(screen.getByRole("alert")).toHaveTextContent("elapsed coverage remains Unknown");
    expect(screen.getByTestId("timeline-scroller").querySelector("svg")).toBeNull();
    rerender(<TimelineDetail range={range} selectedDay="2026-05-17" snapshots={[]} onPrev={vi.fn()} onNext={vi.fn()} />);
    expect(screen.getByRole("alert")).toBeInTheDocument();
  });
});
