import { describe, it, expect, vi } from "vitest";
import { render, screen, act } from "@testing-library/react";
import HeightRail from "./HeightRail";

describe("HeightRail — Pulse Animation", () => {
  it("adds pulse class when state changes from Sitting to Standing", () => {
    const { rerender } = render(
      <HeightRail deskHeightCm={75} state="Sitting">
        Test content
      </HeightRail>
    );

    // Move from Sitting to Standing
    act(() => {
      rerender(
        <HeightRail deskHeightCm={110} state="Standing">
          Test content
        </HeightRail>
      );
    });

    const indicator = screen.getByTitle("110 cm").querySelector(".height-rail__indicator");
    expect(indicator).toHaveClass("height-rail__indicator--pulse");
  });

  it("removes pulse class after 600ms", async () => {
    vi.useFakeTimers();
    try {
      const { rerender } = render(
        <HeightRail deskHeightCm={75} state="Sitting">
          Test content
        </HeightRail>
      );

      // Move from Sitting to Standing
      act(() => {
        rerender(
          <HeightRail deskHeightCm={110} state="Standing">
            Test content
          </HeightRail>
        );
      });

      const indicator = screen.getByTitle("110 cm").querySelector(".height-rail__indicator");
      expect(indicator).toHaveClass("height-rail__indicator--pulse");

      // Advance time by 600ms
      act(() => {
        vi.advanceTimersByTime(600);
      });

      // Pulse class should be removed
      expect(indicator).not.toHaveClass("height-rail__indicator--pulse");
    } finally {
      vi.restoreAllMocks();
    }
  });

  it("does NOT add pulse when state changes from Standing to Walking", () => {
    const { rerender } = render(
      <HeightRail deskHeightCm={110} state="Standing">
        Test content
      </HeightRail>
    );

    // Move from Standing to Walking (both are break states)
    act(() => {
      rerender(
        <HeightRail deskHeightCm={110} state="Walking">
          Test content
        </HeightRail>
      );
    });

    const indicator = screen.getByTitle("110 cm").querySelector(".height-rail__indicator");
    expect(indicator).not.toHaveClass("height-rail__indicator--pulse");
  });

  it.each([
    { from: "Sitting" as const, to: "Walking" as const, height: 110 },
    { from: "Sitting" as const, to: "Away" as const, height: 75 },
  ])("does NOT pulse on $from→$to", ({ from, to, height }) => {
    const { rerender } = render(
      <HeightRail deskHeightCm={75} state={from}>Test</HeightRail>
    );
    act(() => {
      rerender(<HeightRail deskHeightCm={height} state={to}>Test</HeightRail>);
    });
    const indicator = screen.getByTitle(`${height} cm`).querySelector(".height-rail__indicator");
    expect(indicator).not.toHaveClass("height-rail__indicator--pulse");
  });

  it("hides indicator when deskHeightCm is 0 (no reading)", () => {
    render(
      <HeightRail deskHeightCm={0} state="Sitting">
        Test content
      </HeightRail>
    );

    const indicator = screen.queryByTitle(/cm/);
    expect(indicator).not.toBeInTheDocument();
  });

  it("positions indicator based on desk height percentage", () => {
    const { rerender } = render(
      <HeightRail deskHeightCm={60} state="Sitting">
        Test content
      </HeightRail>
    );

    // At minimum (60cm), indicator should be at bottom (0%)
    let indicator = screen.getByTitle("60 cm").querySelector(".height-rail__indicator");
    expect(indicator).toHaveStyle("bottom: 0%");

    // At maximum (130cm), indicator should be at top (100%)
    act(() => {
      rerender(
        <HeightRail deskHeightCm={130} state="Standing">
          Test content
        </HeightRail>
      );
    });

    indicator = screen.getByTitle("130 cm").querySelector(".height-rail__indicator");
    expect(indicator).toHaveStyle("bottom: 100%");

    // At midpoint (95cm), indicator should be at 50%
    act(() => {
      rerender(
        <HeightRail deskHeightCm={95} state="Standing">
          Test content
        </HeightRail>
      );
    });

    indicator = screen.getByTitle("95 cm").querySelector(".height-rail__indicator");
    expect(indicator).toHaveStyle("bottom: 50%");
  });

  it("clamps height values outside min/max range", () => {
    const { rerender } = render(
      <HeightRail deskHeightCm={50} state="Sitting">
        Test content
      </HeightRail>
    );

    // Value below min (60cm) should clamp to min (0%)
    act(() => {
      rerender(
        <HeightRail deskHeightCm={50} state="Sitting">
          Test content
        </HeightRail>
      );
    });

    let indicator = screen.getByTitle("50 cm").querySelector(".height-rail__indicator");
    expect(indicator).toHaveStyle("bottom: 0%");

    // Value above max (130cm) should clamp to max (100%)
    act(() => {
      rerender(
        <HeightRail deskHeightCm={150} state="Standing">
          Test content
        </HeightRail>
      );
    });

    indicator = screen.getByTitle("150 cm").querySelector(".height-rail__indicator");
    expect(indicator).toHaveStyle("bottom: 100%");
  });

  it("only pulses on the first Sitting→Standing transition", async () => {
    vi.useFakeTimers();
    try {
      const { rerender } = render(
        <HeightRail deskHeightCm={75} state="Sitting">
          Test content
        </HeightRail>
      );

      // First transition: Sitting → Standing
      act(() => {
        rerender(
          <HeightRail deskHeightCm={110} state="Standing">
            Test content
          </HeightRail>
        );
      });

      let indicator = screen.getByTitle("110 cm").querySelector(".height-rail__indicator");
      expect(indicator).toHaveClass("height-rail__indicator--pulse");

      // Advance past animation duration
      act(() => {
        vi.advanceTimersByTime(600);
      });

      // Pulse should be removed
      expect(indicator).not.toHaveClass("height-rail__indicator--pulse");

      // Transition back to Sitting
      act(() => {
        rerender(
          <HeightRail deskHeightCm={75} state="Sitting">
            Test content
          </HeightRail>
        );
      });

      indicator = screen.getByTitle("75 cm").querySelector(".height-rail__indicator");
      expect(indicator).not.toHaveClass("height-rail__indicator--pulse");

      // Transition to Standing again should pulse
      act(() => {
        rerender(
          <HeightRail deskHeightCm={110} state="Standing">
            Test content
          </HeightRail>
        );
      });

      indicator = screen.getByTitle("110 cm").querySelector(".height-rail__indicator");
      expect(indicator).toHaveClass("height-rail__indicator--pulse");
    } finally {
      vi.restoreAllMocks();
    }
  });
});
