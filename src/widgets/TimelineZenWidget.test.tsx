/**
 * TimelineZenWidget.test.tsx — tests for the Timeline Zen widget.
 *
 * Verifies rendering in all states, timeline height, and absence of
 * coach text / previous session cards.
 */
import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { TimelineZenWidget } from "./TimelineZenWidget";
import type { WidgetProps, SessionEntry } from "@/types";

function mockWidgetProps(overrides: Partial<WidgetProps> = {}): WidgetProps {
  return {
    connected: true,
    port: "COM3",
    state: "Sitting",
    deskHeightCm: 72.5,
    currentSessionSecs: 600,
    limitSecs: 2400,
    limitRemaining: 1800,
    limitRatio: 0.25,
    breakSecs: 0,
    breakResetThreshold: 600,
    breakResetProgress: 0,
    previousSession: null,
    todaySessions: [],
    todayChanges: 3,
    todayStandingSecs: 1200,
    todaySittingSecs: 3600,
    todayScore: 12,
    error: null,
    onOpenSettings: vi.fn(),
    ...overrides,
  };
}

function makeSessions(): SessionEntry[] {
  const now = new Date();
  const ago = (mins: number) =>
    new Date(now.getTime() - mins * 60 * 1000).toISOString();
  return [
    { start: ago(60), end: ago(30), state: "Sitting", duration_secs: 1800 },
    { start: ago(30), end: ago(20), state: "Standing", duration_secs: 600 },
    { start: ago(20), end: null, state: "Sitting", duration_secs: 1200 },
  ];
}

describe("TimelineZenWidget", () => {
  it("renders without crashing", () => {
    render(<TimelineZenWidget {...mockWidgetProps()} />);
    expect(screen.getByTestId("timeline-zen-widget")).toBeInTheDocument();
  });

  it("renders timeline and status sections", () => {
    render(<TimelineZenWidget {...mockWidgetProps()} />);
    expect(screen.getByTestId("zen-timeline")).toBeInTheDocument();
    expect(screen.getByTestId("zen-status")).toBeInTheDocument();
  });

  it("renders in Sitting state", () => {
    render(<TimelineZenWidget {...mockWidgetProps({ state: "Sitting" })} />);
    expect(screen.getByText("●")).toBeInTheDocument();
  });

  it("renders in Standing state", () => {
    render(<TimelineZenWidget {...mockWidgetProps({ state: "Standing" })} />);
    expect(screen.getByTestId("zen-status")).toBeInTheDocument();
  });

  it("renders in Away state", () => {
    render(<TimelineZenWidget {...mockWidgetProps({ state: "Away" })} />);
    expect(screen.getByTestId("zen-status")).toBeInTheDocument();
  });

  it("renders with null state", () => {
    render(<TimelineZenWidget {...mockWidgetProps({ state: null })} />);
    expect(screen.getByTestId("timeline-zen-widget")).toBeInTheDocument();
  });

  it("does not display coach text or previous session", () => {
    const props = mockWidgetProps({
      previousSession: { state: "Standing", durationSecs: 300, wasEffective: true },
    });
    render(<TimelineZenWidget {...props} />);
    const widget = screen.getByTestId("timeline-zen-widget");
    expect(widget.textContent).not.toContain("Standing 300s");
    expect(widget.textContent).not.toContain("effective");
  });

  it("renders session blocks when sessions exist", () => {
    const props = mockWidgetProps({ todaySessions: makeSessions() });
    render(<TimelineZenWidget {...props} />);
    const timeline = screen.getByTestId("zen-timeline");
    const blocks = timeline.querySelectorAll(".zen-timeline__block");
    expect(blocks.length).toBe(3);
  });

  it("shows limitRemaining not sittingSeconds in timer", () => {
    const props = mockWidgetProps({ limitRemaining: 1800, limitSecs: 2400 });
    render(<TimelineZenWidget {...props} />);
    expect(screen.getByText("30:00")).toBeInTheDocument();
    expect(screen.getByText("40:00")).toBeInTheDocument();
  });

  it("shows desk height", () => {
    render(<TimelineZenWidget {...mockWidgetProps({ deskHeightCm: 72.4 })} />);
    expect(screen.getByText("72.4 cm")).toBeInTheDocument();
  });

  it("timeline has 48px height", () => {
    render(<TimelineZenWidget {...mockWidgetProps()} />);
    const timeline = screen.getByTestId("zen-timeline");
    expect(timeline.style.height).toBe("48px");
  });
});
