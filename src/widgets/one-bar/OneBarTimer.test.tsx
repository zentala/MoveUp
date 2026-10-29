/**
 * OneBarTimer.test.tsx — Regression tests for standing timer display.
 *
 * Prevents the bug where popup showed sitting data when standing.
 * Verifies that elapsed/total and color scheme switch correctly per state.
 */
import { describe, it, expect, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { OneBarTimer } from "./OneBarTimer";
import type { WidgetProps } from "@/types";

function props(overrides: Partial<WidgetProps> = {}): WidgetProps {
  return {
    connected: true,
    port: "COM3",
    state: "Sitting",
    deskHeightCm: 72.5,
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

describe("OneBarTimer — standing timer regression", () => {
  it("standing state shows breakSecs / standLimitSecs", () => {
    render(
      <OneBarTimer
        {...props({
          state: "Standing",
          breakSecs: 300,
          standLimitSecs: 900,
          limitUsedSecs: 1500,
          limitSecs: 2700,
        })}
      />,
    );
    const bigNum = screen.getByTestId("one-bar-big-number");
    // 300s = 05:00, 900s = 15:00
    expect(bigNum.textContent).toContain("05:00");
    expect(bigNum.textContent).toContain("15:00");
    // Must NOT show sitting values (25:00 / 45:00)
    expect(bigNum.textContent).not.toContain("25:00");
    expect(bigNum.textContent).not.toContain("45:00");
  });

  it("sitting state shows limitUsedSecs / limitSecs", () => {
    render(
      <OneBarTimer
        {...props({
          state: "Sitting",
          limitUsedSecs: 1500,
          limitSecs: 2700,
          breakSecs: 300,
          standLimitSecs: 900,
        })}
      />,
    );
    const bigNum = screen.getByTestId("one-bar-big-number");
    // 1500s = 25:00, 2700s = 45:00
    expect(bigNum.textContent).toContain("25:00");
    expect(bigNum.textContent).toContain("45:00");
    // Must NOT show standing values (05:00 / 15:00)
    expect(bigNum.textContent).not.toContain("05:00");
  });

  it("standing state passes standing color scheme to ProgressBar", () => {
    render(
      <OneBarTimer
        {...props({
          state: "Standing",
          breakSecs: 480,
          standLimitSecs: 900,
        })}
      />,
    );
    const fill = screen.getByTestId("progress-bar-fill");
    expect(fill.className).toContain("progress-bar__fill--standing");
  });

  it("sitting state passes sitting color scheme to ProgressBar", () => {
    render(
      <OneBarTimer
        {...props({ state: "Sitting" })}
      />,
    );
    const fill = screen.getByTestId("progress-bar-fill");
    expect(fill.className).toContain("progress-bar__fill--sitting");
  });

  it("away state shows breakSecs / standLimitSecs (not sitting data)", () => {
    render(
      <OneBarTimer
        {...props({
          state: "Away",
          breakSecs: 720,
          standLimitSecs: 900,
          limitUsedSecs: 1800,
          limitSecs: 2400,
        })}
      />,
    );
    const bigNum = screen.getByTestId("one-bar-big-number");
    // 720s = 12:00, 900s = 15:00
    expect(bigNum.textContent).toContain("12:00");
    expect(bigNum.textContent).toContain("15:00");
    // Must NOT show sitting values (30:00 / 40:00)
    expect(bigNum.textContent).not.toContain("30:00");
    expect(bigNum.textContent).not.toContain("40:00");
  });

  it("away state passes gray color scheme to ProgressBar", () => {
    render(
      <OneBarTimer
        {...props({ state: "Away", breakSecs: 720, standLimitSecs: 900 })}
      />,
    );
    const fill = screen.getByTestId("progress-bar-fill");
    expect(fill.className).toContain("progress-bar__fill--gray");
  });

  it("walking state shows breakSecs / standLimitSecs", () => {
    render(
      <OneBarTimer
        {...props({
          state: "Walking",
          breakSecs: 600,
          standLimitSecs: 900,
          limitUsedSecs: 2000,
          limitSecs: 2400,
        })}
      />,
    );
    const bigNum = screen.getByTestId("one-bar-big-number");
    // 600s = 10:00, 900s = 15:00
    expect(bigNum.textContent).toContain("10:00");
    expect(bigNum.textContent).toContain("15:00");
  });

  it("displays correct state in tooltip for each state", () => {
    const { rerender } = render(
      <OneBarTimer {...props({ state: "Standing", breakSecs: 0, standLimitSecs: 900 })} />,
    );
    expect(screen.getByTitle(/Standing for/)).toBeInTheDocument();

    rerender(<OneBarTimer {...props({ state: "Sitting" })} />);
    expect(screen.getByTitle(/Sitting for/)).toBeInTheDocument();

    rerender(<OneBarTimer {...props({ state: "Away", breakSecs: 0, standLimitSecs: 900 })} />);
    expect(screen.getByTitle(/Away for/)).toBeInTheDocument();
  });

  it("shows sitting overtime separately, without changing the full balance number", () => {
    const { rerender } = render(
      <OneBarTimer
        {...props({
          state: "Sitting",
          limitRemaining: -300,
          limitUsedSecs: 2700,
          limitSecs: 2400,
        })}
      />,
    );
    const bigNum = screen.getByTestId("one-bar-big-number");
    expect(bigNum.textContent).not.toContain("+");
    expect(screen.getByTestId("one-bar-overtime")).toHaveTextContent("Over limit 5m");

    rerender(
      <OneBarTimer
        {...props({
          state: "Standing",
          limitRemaining: -300,
          breakSecs: 300,
          standLimitSecs: 900,
        })}
      />,
    );
    const bigNum2 = screen.getByTestId("one-bar-big-number");
    expect(bigNum2.textContent).not.toContain("+");
    expect(screen.queryByTestId("one-bar-overtime")).not.toBeInTheDocument();
  });

  it("labels sitting balance and shows the 12 minute overtime separately for 52 over 40", () => {
    render(<OneBarTimer {...props({ limitUsedSecs: 52 * 60, limitSecs: 40 * 60, limitRemaining: -12 * 60, limitRatio: 1.3 })} />);
    expect(screen.getByText("Sitting balance")).toBeInTheDocument();
    expect(screen.getByTestId("one-bar-big-number")).toHaveTextContent("52:00 / 40:00");
    expect(screen.getByTestId("one-bar-overtime")).toHaveTextContent("Over limit 12m");
  });

  it("returns from the secondary credit panel to the sitting status disclosure", () => {
    render(<OneBarTimer {...props({
      limitUsedSecs: 52 * 60,
      limitSecs: 40 * 60,
      limitRemaining: -12 * 60,
      limitRatio: 1.3,
      lastBreakCreditFacts: [{
        break_started_at: "2026-10-29T10:00:00Z",
        observed_break_secs: 300,
        minimum_break_secs: 60,
        multiplier: 2,
        requested_credit_secs: 600,
        prior_applied_credit_secs: 0,
        sitting_before_secs: 480,
        applied_credit_secs: 480,
        sitting_after_secs: 0,
        applied_at: "2026-10-29T10:05:00Z",
        no_credit_reason: null,
      }],
    })} />);
    const disclosure = screen.getByText("Break credit details");

    fireEvent.click(disclosure);
    expect(screen.getByRole("heading", { name: "Captured break credit" })).toBeInTheDocument();
    expect(screen.getByRole("region", { name: "Break credit entries" })).toHaveAttribute("tabindex", "0");

    fireEvent.click(screen.getByRole("button", { name: "Close break credit details" }));
    expect(disclosure).toHaveFocus();
    expect(screen.getByTestId("one-bar-big-number")).toHaveTextContent("52:00 / 40:00");
    expect(screen.getByTestId("one-bar-overtime")).toHaveTextContent("Over limit 12m");
  });
});
