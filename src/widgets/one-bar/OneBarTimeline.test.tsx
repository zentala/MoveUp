/**
 * OneBarTimeline.test.tsx — tests for the session timeline component.
 *
 * Verifies: hour markers always present, session blocks render,
 * tooltips, empty state, live block.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { OneBarTimeline } from "./OneBarTimeline";
import "./one-bar.css";
import type { WidgetProps, SessionEntry } from "@/types";
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
    const sessions = makeSessions(1, 21);
    render(<OneBarTimeline {...baseProps({ todaySessions: sessions })} />);
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
  });
});
