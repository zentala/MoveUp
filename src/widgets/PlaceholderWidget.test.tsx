/**
 * PlaceholderWidget.test.tsx — verifies the dev widget renders without crash.
 */
import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { PlaceholderWidget } from "./PlaceholderWidget";
import type { WidgetProps, DeskState } from "@/types";

function mockWidgetProps(overrides: Partial<WidgetProps> = {}): WidgetProps {
  return {
    connected: true,
    port: "COM3",
    state: "Sitting",
    deskHeightCm: 72.5,
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
    todayChanges: 3,
    todayStandingSecs: 1200,
    todaySittingSecs: 3600,
    todayScore: 12,
    metrics: [],
    error: null,
    onOpenSettings: vi.fn(),
    ...overrides,
  };
}

describe("PlaceholderWidget", () => {
  it("renders without crashing", () => {
    render(<PlaceholderWidget {...mockWidgetProps()} />);
    expect(screen.getByTestId("placeholder-widget")).toBeInTheDocument();
  });

  it("displays connection info", () => {
    render(<PlaceholderWidget {...mockWidgetProps()} />);
    expect(screen.getByText(/connected: true/)).toBeInTheDocument();
    expect(screen.getByText(/COM3/)).toBeInTheDocument();
  });

  it("displays state info", () => {
    render(<PlaceholderWidget {...mockWidgetProps({ state: "Standing" })} />);
    expect(screen.getByText(/Standing/)).toBeInTheDocument();
  });

  it("shows waiting when state is null", () => {
    render(<PlaceholderWidget {...mockWidgetProps({ state: null as unknown as DeskState })} />);
    expect(screen.getByText(/waiting/)).toBeInTheDocument();
  });

  it("shows previous session when available", () => {
    const props = mockWidgetProps({
      previousSession: { state: "Standing", durationSecs: 300, wasEffective: true },
    });
    render(<PlaceholderWidget {...props} />);
    expect(screen.getByText(/Standing 300s/)).toBeInTheDocument();
  });

  it("shows error banner when error exists", () => {
    render(<PlaceholderWidget {...mockWidgetProps({ error: "Sensor lost" })} />);
    expect(screen.getByText("Sensor lost")).toBeInTheDocument();
  });

  it("calls onOpenSettings when settings button clicked", () => {
    const onOpen = vi.fn();
    render(<PlaceholderWidget {...mockWidgetProps({ onOpenSettings: onOpen })} />);
    screen.getByText("settings").click();
    expect(onOpen).toHaveBeenCalledOnce();
  });
});
