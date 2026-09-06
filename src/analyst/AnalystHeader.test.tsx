/**
 * AnalystHeader.test.tsx — title rendering + nav callbacks.
 */
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { AnalystHeader } from "./AnalystHeader";

describe("AnalystHeader", () => {
  it("renders the weekday and the formatted date of the selected day", () => {
    render(
      <AnalystHeader selectedDay="2026-05-17" onPrev={vi.fn()} onNext={vi.fn()} />,
    );
    expect(screen.getByTestId("analyst-header-weekday").textContent).toBe("Sunday");
    expect(screen.getByTestId("analyst-header-date").textContent).toBe("17 May 2026");
    expect(screen.getByTestId("analyst-header-title").textContent).toContain("·");
  });

  it("fires onPrev when the left arrow is clicked", () => {
    const onPrev = vi.fn();
    render(
      <AnalystHeader selectedDay="2026-05-17" onPrev={onPrev} onNext={vi.fn()} />,
    );
    fireEvent.click(screen.getByTestId("analyst-header-prev"));
    expect(onPrev).toHaveBeenCalledTimes(1);
  });

  it("fires onNext when the right arrow is clicked", () => {
    const onNext = vi.fn();
    render(
      <AnalystHeader selectedDay="2026-05-17" onPrev={vi.fn()} onNext={onNext} />,
    );
    fireEvent.click(screen.getByTestId("analyst-header-next"));
    expect(onNext).toHaveBeenCalledTimes(1);
  });

  it("labels both arrows for screen readers", () => {
    render(
      <AnalystHeader selectedDay="2026-05-17" onPrev={vi.fn()} onNext={vi.fn()} />,
    );
    expect(screen.getByLabelText("Previous day")).toBeTruthy();
    expect(screen.getByLabelText("Next day")).toBeTruthy();
  });

  it("re-renders the title when the selected day changes", () => {
    const { rerender } = render(
      <AnalystHeader selectedDay="2026-05-17" onPrev={vi.fn()} onNext={vi.fn()} />,
    );
    rerender(
      <AnalystHeader selectedDay="2026-05-18" onPrev={vi.fn()} onNext={vi.fn()} />,
    );
    expect(screen.getByTestId("analyst-header-weekday").textContent).toBe("Monday");
    expect(screen.getByTestId("analyst-header-date").textContent).toBe("18 May 2026");
  });
});
