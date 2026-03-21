/**
 * ZenStatus.test.tsx — tests for the ZenStatus component.
 *
 * Verifies timer display, dot color classes, and bar fill behavior.
 */
import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { ZenStatus } from "./ZenStatus";

describe("ZenStatus", () => {
  it("renders without crashing", () => {
    render(
      <ZenStatus
        state="Sitting"
        limitRemaining={1800}
        limitSecs={2400}
        limitRatio={0.25}
        deskHeightCm={72.5}
      />,
    );
    expect(screen.getByTestId("zen-status")).toBeInTheDocument();
  });

  it("shows limitRemaining as timer", () => {
    render(
      <ZenStatus
        state="Sitting"
        limitRemaining={1234}
        limitSecs={2400}
        limitRatio={0.5}
        deskHeightCm={70}
      />,
    );
    expect(screen.getByText("20:34")).toBeInTheDocument();
  });

  it("shows negative remaining with minus sign", () => {
    render(
      <ZenStatus
        state="Sitting"
        limitRemaining={-120}
        limitSecs={2400}
        limitRatio={1.05}
        deskHeightCm={70}
      />,
    );
    expect(screen.getByText("-02:00")).toBeInTheDocument();
  });

  it("applies correct dot class for each state", () => {
    const { rerender } = render(
      <ZenStatus
        state="Sitting"
        limitRemaining={0}
        limitSecs={0}
        limitRatio={0}
        deskHeightCm={0}
      />,
    );
    expect(screen.getByText("●").className).toContain("zen-dot--sitting");

    rerender(
      <ZenStatus
        state="Standing"
        limitRemaining={0}
        limitSecs={0}
        limitRatio={0}
        deskHeightCm={0}
      />,
    );
    expect(screen.getByText("●").className).toContain("zen-dot--standing");

    rerender(
      <ZenStatus
        state="Away"
        limitRemaining={0}
        limitSecs={0}
        limitRatio={0}
        deskHeightCm={0}
      />,
    );
    expect(screen.getByText("●").className).toContain("zen-dot--away");
  });

  it("shows desk height with one decimal", () => {
    render(
      <ZenStatus
        state="Sitting"
        limitRemaining={0}
        limitSecs={0}
        limitRatio={0}
        deskHeightCm={72.456}
      />,
    );
    expect(screen.getByText("72.5 cm")).toBeInTheDocument();
  });

  it("bar fill width reflects ratio capped at 100%", () => {
    render(
      <ZenStatus
        state="Sitting"
        limitRemaining={0}
        limitSecs={2400}
        limitRatio={1.5}
        deskHeightCm={70}
      />,
    );
    const bar = screen.getByTestId("zen-bar");
    const fill = bar.querySelector(".zen-bar__fill") as HTMLElement;
    expect(fill.style.width).toBe("100%");
  });

  it("uses away class when state is null", () => {
    render(
      <ZenStatus
        state={null}
        limitRemaining={0}
        limitSecs={0}
        limitRatio={0}
        deskHeightCm={0}
      />,
    );
    expect(screen.getByText("●").className).toContain("zen-dot--away");
  });
});
