/**
 * OneBarWidget.test.tsx — verifies the One Bar widget renders correctly
 * across all states and temperature levels.
 */
import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { OneBarWidget } from "./OneBarWidget";
import type { WidgetProps } from "@/types";

function props(overrides: Partial<WidgetProps> = {}): WidgetProps {
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

describe("OneBarWidget", () => {
  it("renders without crashing", () => {
    render(<OneBarWidget {...props()} />);
    expect(screen.getByTestId("one-bar-widget")).toBeInTheDocument();
  });

  it("applies calm temperature class when sitting < 50%", () => {
    render(<OneBarWidget {...props({ limitRatio: 0.3 })} />);
    const el = screen.getByTestId("one-bar-widget");
    expect(el.className).toContain("one-bar--calm");
  });

  it("applies warm temperature class when sitting 50-80%", () => {
    render(<OneBarWidget {...props({ limitRatio: 0.6 })} />);
    const el = screen.getByTestId("one-bar-widget");
    expect(el.className).toContain("one-bar--warm");
  });

  it("applies hot temperature class when sitting 80-100%", () => {
    render(<OneBarWidget {...props({ limitRatio: 0.9 })} />);
    const el = screen.getByTestId("one-bar-widget");
    expect(el.className).toContain("one-bar--hot");
  });

  it("applies burning temperature class when over limit", () => {
    render(<OneBarWidget {...props({ limitRatio: 1.2 })} />);
    const el = screen.getByTestId("one-bar-widget");
    expect(el.className).toContain("one-bar--burning");
  });

  it("applies standing temperature class", () => {
    render(
      <OneBarWidget
        {...props({ state: "Standing", breakResetProgress: 0.5 })}
      />,
    );
    const el = screen.getByTestId("one-bar-widget");
    expect(el.className).toContain("one-bar--standing");
  });

  it("applies away temperature class", () => {
    render(<OneBarWidget {...props({ state: "Away" })} />);
    const el = screen.getByTestId("one-bar-widget");
    expect(el.className).toContain("one-bar--away");
  });

  it("applies reset temperature class", () => {
    render(
      <OneBarWidget
        {...props({ state: "Standing", breakResetProgress: 1.0 })}
      />,
    );
    const el = screen.getByTestId("one-bar-widget");
    expect(el.className).toContain("one-bar--reset");
  });

  it("shows desk height in header via StateIndicator", () => {
    render(<OneBarWidget {...props({ deskHeightCm: 72.4 })} />);
    expect(screen.getByText("72 cm")).toBeInTheDocument();
  });

  it("shows state label in header via StateIndicator", () => {
    const { rerender } = render(<OneBarWidget {...props({ state: "Sitting" })} />);
    expect(screen.getByText("Sitting")).toBeInTheDocument();

    rerender(<OneBarWidget {...props({ state: "Standing" })} />);
    expect(screen.getByText("Standing")).toBeInTheDocument();

    rerender(<OneBarWidget {...props({ state: "Away" })} />);
    expect(screen.getByText("Away")).toBeInTheDocument();
  });

  it("shows activity status in header via StateIndicator", () => {
    render(<OneBarWidget {...props({ idleSecs: 0 })} />);
    expect(screen.getByText("Active")).toBeInTheDocument();
  });

  it("shows elapsed/total as big number", () => {
    render(
      <OneBarWidget
        {...props({ currentSessionSecs: 600, limitSecs: 2400 })}
      />,
    );
    const bigNum = screen.getByTestId("one-bar-big-number");
    // 600s = 10:00, 2400s = 40:00
    expect(bigNum.textContent).toContain("10:00");
    expect(bigNum.textContent).toContain("40:00");
  });

  it("renders all sub-components", () => {
    render(<OneBarWidget {...props()} />);
    expect(screen.getByTestId("one-bar-timeline")).toBeInTheDocument();
    expect(screen.getByTestId("one-bar-timer")).toBeInTheDocument();
    expect(screen.getByTestId("progress-bar-container")).toBeInTheDocument();
  });

  it("renders KpiStrip with metrics", () => {
    render(
      <OneBarWidget
        {...props({
          metrics: [
            {
              id: "standing_pct",
              label: "\u2195 Standing",
              result: { value: 15, display: "15%", level: "green", is_personal_best: false },
            },
            {
              id: "position_rate",
              label: "\u21c4 Changes",
              result: { value: 1.2, display: "1.2/h", level: "green", is_personal_best: true },
            },
          ],
        })}
      />,
    );
    expect(screen.getByTestId("kpi-strip")).toBeInTheDocument();
    expect(screen.getByText("15%")).toBeInTheDocument();
    expect(screen.getByText("1.2/h")).toBeInTheDocument();
  });

  it("calls onOpenSettings when settings button clicked", () => {
    const onOpen = vi.fn();
    render(<OneBarWidget {...props({ onOpenSettings: onOpen })} />);
    screen.getByTitle("Settings").click();
    expect(onOpen).toHaveBeenCalledOnce();
  });

  it("big number shows break elapsed/standing target for standing", () => {
    render(
      <OneBarWidget
        {...props({
          state: "Standing",
          breakSecs: 180,
          standLimitSecs: 600,
        })}
      />,
    );
    const bigNum = screen.getByTestId("one-bar-big-number");
    expect(bigNum.textContent).toContain("03:00");
    expect(bigNum.textContent).toContain("10:00");
  });

  it("away state has gray color scheme on progress bar", () => {
    render(
      <OneBarWidget
        {...props({
          state: "Away",
          currentSessionSecs: 120,
          limitSecs: 600,
        })}
      />,
    );
    const fill = screen.getByTestId("progress-bar-fill");
    expect(fill.className).toContain("progress-bar__fill--gray");
  });

  it("shows previous session info when available", () => {
    render(
      <OneBarWidget
        {...props({
          previousSession: {
            state: "Standing",
            durationSecs: 960,
            wasEffective: true,
          },
        })}
      />,
    );
    expect(screen.getByText(/previously/)).toBeInTheDocument();
    expect(screen.getByText(/stood/)).toBeInTheDocument();
  });
});
