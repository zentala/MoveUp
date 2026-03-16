/**
 * SessionProgress.test.tsx — unit tests for the SessionProgress component.
 *
 * Verifies correct width percentages and CSS color classes at key thresholds.
 */
import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import SessionProgress from "@/components/SessionProgress";

describe("SessionProgress", () => {
  it("renders at 0% when sittingSeconds is 0", () => {
    render(<SessionProgress sittingSeconds={0} limitSeconds={2400} />);
    const bar = screen.getByTestId("progress-bar");
    expect(bar).toHaveStyle({ width: "0%" });
    expect(bar).toHaveClass("progress-fill--ok");
  });

  it("renders at 50% when halfway through the session", () => {
    render(<SessionProgress sittingSeconds={1200} limitSeconds={2400} />);
    const bar = screen.getByTestId("progress-bar");
    expect(bar).toHaveStyle({ width: "50%" });
    expect(bar).toHaveClass("progress-fill--ok");
  });

  it("renders at 100% when limit is exactly reached", () => {
    render(<SessionProgress sittingSeconds={2400} limitSeconds={2400} />);
    const bar = screen.getByTestId("progress-bar");
    expect(bar).toHaveStyle({ width: "100%" });
    expect(bar).toHaveClass("progress-fill--alert");
  });

  it("clamps to 100% when sittingSeconds exceeds limitSeconds", () => {
    render(<SessionProgress sittingSeconds={3000} limitSeconds={2400} />);
    const bar = screen.getByTestId("progress-bar");
    expect(bar).toHaveStyle({ width: "100%" });
  });

  it("applies warn class at 70% of the session (above 60% threshold)", () => {
    // 70% → 0.7, which is >= 0.6 (warn) but < 0.85 (alert)
    render(<SessionProgress sittingSeconds={1680} limitSeconds={2400} />);
    const bar = screen.getByTestId("progress-bar");
    expect(bar).toHaveClass("progress-fill--warn");
  });

  it("applies alert class at 90% of the session (above 85% threshold)", () => {
    // 90% → 0.9, which is >= 0.85 (alert)
    render(<SessionProgress sittingSeconds={2160} limitSeconds={2400} />);
    const bar = screen.getByTestId("progress-bar");
    expect(bar).toHaveClass("progress-fill--alert");
  });

  it("shows 'take a break' when limit is exceeded", () => {
    render(<SessionProgress sittingSeconds={2500} limitSeconds={2400} />);
    expect(screen.getByText(/take a break/i)).toBeInTheDocument();
  });

  it("shows remaining time when under the limit", () => {
    // 1 minute remaining = 60 seconds
    render(<SessionProgress sittingSeconds={2340} limitSeconds={2400} />);
    expect(screen.getByText(/left/i)).toBeInTheDocument();
    expect(screen.getByText(/01:00 left/i)).toBeInTheDocument();
  });

  it("renders at 0% when limitSeconds is 0 (avoids divide-by-zero)", () => {
    render(<SessionProgress sittingSeconds={0} limitSeconds={0} />);
    const bar = screen.getByTestId("progress-bar");
    expect(bar).toHaveStyle({ width: "0%" });
  });
});
