/**
 * ProgressBar.test.tsx — unit tests for the unified ProgressBar component.
 */
import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { ProgressBar } from "@/components/ProgressBar";

describe("ProgressBar", () => {
  it("renders overlay variant with correct classes", () => {
    render(<ProgressBar elapsed={100} total={200} variant="overlay" colorScheme="sitting" />);
    const container = screen.getByTestId("progress-bar-container");
    expect(container).toHaveClass("progress-bar", "progress-bar--overlay");
  });

  it("renders inline variant with correct classes", () => {
    render(<ProgressBar elapsed={100} total={200} variant="inline" colorScheme="sitting" />);
    const container = screen.getByTestId("progress-bar-container");
    expect(container).toHaveClass("progress-bar", "progress-bar--inline");
  });

  it("shows 0% width when total is 0 (div-by-zero guard)", () => {
    render(<ProgressBar elapsed={100} total={0} variant="inline" colorScheme="gray" />);
    const fill = screen.getByTestId("progress-bar-fill");
    expect(fill).toHaveStyle({ width: "0%" });
  });

  it("calculates correct width: elapsed=600, total=2400 → 25%", () => {
    render(<ProgressBar elapsed={600} total={2400} variant="inline" colorScheme="sitting" />);
    const fill = screen.getByTestId("progress-bar-fill");
    expect(fill).toHaveStyle({ width: "25%" });
  });

  it("applies sitting color scheme class", () => {
    render(<ProgressBar elapsed={100} total={200} variant="inline" colorScheme="sitting" />);
    const fill = screen.getByTestId("progress-bar-fill");
    expect(fill).toHaveClass("progress-bar__fill--sitting");
  });

  it("applies standing color scheme class", () => {
    render(<ProgressBar elapsed={100} total={200} variant="inline" colorScheme="standing" />);
    const fill = screen.getByTestId("progress-bar-fill");
    expect(fill).toHaveClass("progress-bar__fill--standing");
  });

  it("applies gray color scheme class", () => {
    render(<ProgressBar elapsed={100} total={200} variant="inline" colorScheme="gray" />);
    const fill = screen.getByTestId("progress-bar-fill");
    expect(fill).toHaveClass("progress-bar__fill--gray");
  });

  it("adds shimmer class when shimmer prop is true", () => {
    render(<ProgressBar elapsed={100} total={200} variant="inline" colorScheme="sitting" shimmer />);
    const container = screen.getByTestId("progress-bar-container");
    expect(container).toHaveClass("progress-bar--shimmer");
  });

  it("overlay variant has pointerEvents none", () => {
    render(<ProgressBar elapsed={100} total={200} variant="overlay" colorScheme="sitting" />);
    const container = screen.getByTestId("progress-bar-container");
    expect(container).toHaveStyle({ pointerEvents: "none" });
  });

  it("caps ratio at 1.0 when elapsed exceeds total", () => {
    render(<ProgressBar elapsed={3000} total={2400} variant="inline" colorScheme="sitting" />);
    const fill = screen.getByTestId("progress-bar-fill");
    expect(fill).toHaveStyle({ width: "100%" });
  });
});
