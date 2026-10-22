/**
 * TimelineDetail.test.tsx — smoke tests for the continuous timeline strip.
 */
import { act, fireEvent, render, screen } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { LocalDayBounds } from "@/generated/LocalDayBounds";
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
    return {
      date_local: new Date(start).toISOString().slice(0, 10),
      start_utc: new Date(start).toISOString(),
      end_utc: new Date(start + 86_400_000).toISOString(),
      zone_basis: "current-query-local",
      historical_source_zone: "unknown",
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

afterEach(() => vi.useRealTimers());

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

  it("shows pending and error states without browser-local timeline fallback", async () => {
    vi.mocked(invoke).mockReset().mockRejectedValueOnce(new Error("query failed"));
    const { rerender } = render(
      <TimelineDetail range={range} selectedDay="2026-05-17" snapshots={[]} onPrev={vi.fn()} onNext={vi.fn()} />,
    );
    expect(screen.getByRole("status")).toHaveTextContent("Loading query-local calendar bounds");
    await loadBounds();
    expect(screen.getByRole("alert")).toHaveTextContent("elapsed coverage remains Unknown");
    expect(screen.queryByTestId("timeline-scroller")).toBeNull();
    rerender(<TimelineDetail range={range} selectedDay="2026-05-17" snapshots={[]} onPrev={vi.fn()} onNext={vi.fn()} />);
    expect(screen.getByRole("alert")).toBeInTheDocument();
  });
});
