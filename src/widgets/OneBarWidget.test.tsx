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
    elapsed: 600,
    total: 2400,
    colorScheme: "sitting" as const,
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

  it("shows desk height in header", () => {
    render(<OneBarWidget {...props({ deskHeightCm: 72.4 })} />);
    expect(screen.getByText("72 cm")).toBeInTheDocument();
  });

  it("shows limitRemaining as big number, not sittingSeconds", () => {
    render(
      <OneBarWidget
        {...props({
          limitRemaining: 1800,
          currentSessionSecs: 600,
          limitSecs: 2400,
        })}
      />,
    );
    const bigNum = screen.getByTestId("one-bar-big-number");
    // 1800s = 30:00
    expect(bigNum.textContent).toContain("30:00");
  });

  it("renders all sub-components", () => {
    render(<OneBarWidget {...props()} />);
    expect(screen.getByTestId("one-bar-timeline")).toBeInTheDocument();
    expect(screen.getByTestId("one-bar-timer")).toBeInTheDocument();
    expect(screen.getByTestId("one-bar-coach")).toBeInTheDocument();
  });

  it("calls onOpenSettings when settings button clicked", () => {
    const onOpen = vi.fn();
    render(<OneBarWidget {...props({ onOpenSettings: onOpen })} />);
    screen.getByTitle("Settings").click();
    expect(onOpen).toHaveBeenCalledOnce();
  });

  it("shows standing duration when state is Standing", () => {
    render(
      <OneBarWidget
        {...props({
          state: "Standing",
          breakSecs: 180,
          currentSessionSecs: 600,
        })}
      />,
    );
    // Should show break duration (3m), not sitting duration (10m)
    expect(screen.getByText(/for 3m/)).toBeInTheDocument();
  });

  it("shows sitting duration when state is Sitting", () => {
    render(
      <OneBarWidget
        {...props({
          state: "Sitting",
          breakSecs: 0,
          currentSessionSecs: 240,
        })}
      />,
    );
    expect(screen.getByText(/for 4m/)).toBeInTheDocument();
  });

  it("standing timer must NOT show currentSessionSecs (the serde contract bug)", () => {
    // Regression: serde(rename_all=snake_case) made state="standing" (lowercase),
    // so isStandingOrAway was always false, and sessionDuration always showed
    // currentSessionSecs (=0) instead of breakSecs.
    render(
      <OneBarWidget
        {...props({
          state: "Standing",
          breakSecs: 420,    // 7 minutes of standing
          currentSessionSecs: 0,  // reset on transition
        })}
      />,
    );
    // Must show 7m (breakSecs), not 0s (currentSessionSecs)
    expect(screen.getByText(/for 7m/)).toBeInTheDocument();
    expect(screen.queryByText(/for 0s/)).not.toBeInTheDocument();
  });

  it("walking timer shows breakSecs not currentSessionSecs", () => {
    render(
      <OneBarWidget
        {...props({
          state: "Walking",
          breakSecs: 300,
          currentSessionSecs: 0,
        })}
      />,
    );
    expect(screen.getByText(/for 5m/)).toBeInTheDocument();
  });

  it("away timer shows breakSecs not currentSessionSecs", () => {
    render(
      <OneBarWidget
        {...props({
          state: "Away",
          breakSecs: 120,
          currentSessionSecs: 0,
        })}
      />,
    );
    expect(screen.getByText(/for 2m/)).toBeInTheDocument();
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
