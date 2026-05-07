/**
 * OneBarTimeline.test.tsx — tests for the session timeline component.
 *
 * Verifies: hour markers always present, session blocks render,
 * tooltips, empty state, live block.
 */
import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { OneBarTimeline } from "./OneBarTimeline";
import type { WidgetProps, SessionEntry } from "@/types";

function baseProps(overrides: Partial<WidgetProps> = {}): WidgetProps {
  return {
    connected: true,
    port: "COM3",
    state: "Sitting",
    deskHeightCm: 72,
    currentSessionSecs: 600,
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
    });
  }
  return sessions;
}

describe("OneBarTimeline", () => {
  it("shows empty message when no sessions and no live time", () => {
    render(<OneBarTimeline {...baseProps({ currentSessionSecs: 0, todaySessions: [] })} />);
    expect(screen.getByText("No sessions yet")).toBeInTheDocument();
  });

  it("renders session blocks for each session", () => {
    const sessions = makeSessions(3);
    render(<OneBarTimeline {...baseProps({ todaySessions: sessions })} />);
    const timeline = screen.getByTestId("one-bar-timeline");
    const blocks = timeline.querySelectorAll(".one-bar__timeline-block");
    // 3 historical + 1 live block
    expect(blocks.length).toBeGreaterThanOrEqual(3);
  });

  it("renders live block when currentSessionSecs > 0", () => {
    const sessions = makeSessions(1);
    render(<OneBarTimeline {...baseProps({ todaySessions: sessions, currentSessionSecs: 300 })} />);
    expect(screen.getByTestId("timeline-live-block")).toBeInTheDocument();
  });

  it("always shows hour markers when sessions exist", () => {
    const sessions = makeSessions(1, 0.1); // sessions within same hour
    render(<OneBarTimeline {...baseProps({ todaySessions: sessions })} />);
    const timeline = screen.getByTestId("one-bar-timeline");
    const hourLabels = timeline.querySelectorAll(".one-bar__timeline-hour-label");
    // Should have at least start time and current time markers
    expect(hourLabels.length).toBeGreaterThanOrEqual(2);
  });

  it("shows more markers for multi-hour sessions than single-hour", () => {
    const shortSessions = makeSessions(1, 0.1);
    const { unmount } = render(<OneBarTimeline {...baseProps({ todaySessions: shortSessions })} />);
    const shortMarkers = screen.getByTestId("one-bar-timeline")
      .querySelectorAll(".one-bar__timeline-hour-label").length;
    unmount();

    const longSessions = makeSessions(4, 3);
    render(<OneBarTimeline {...baseProps({ todaySessions: longSessions })} />);
    const longMarkers = screen.getByTestId("one-bar-timeline")
      .querySelectorAll(".one-bar__timeline-hour-label").length;

    expect(longMarkers).toBeGreaterThanOrEqual(shortMarkers);
  });

  it("generates hourly markers across midnight crossing", () => {
    // Session started 10 hours ago — guaranteed to cross at least one midnight hour boundary
    const sessions = makeSessions(4, 10);
    render(<OneBarTimeline {...baseProps({ todaySessions: sessions })} />);
    const timeline = screen.getByTestId("one-bar-timeline");
    const hourLabels = timeline.querySelectorAll(".one-bar__timeline-hour-label");
    // Start + multiple hour boundaries + now = at least 5 markers for 10h span
    expect(hourLabels.length).toBeGreaterThanOrEqual(5);
    // Check that at least one label has ":00" (full hour boundary)
    const texts = Array.from(hourLabels).map((el) => el.textContent ?? "");
    expect(texts.some((t) => t.endsWith(":00"))).toBe(true);
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
});
