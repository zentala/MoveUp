/**
 * DateNavigator.test.tsx — smoke tests for the navigator card.
 */
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { SnapshotRow } from "@/test/analyst-fixtures";
import { DateNavigator } from "./DateNavigator";

function snap(ts: string, state: SnapshotRow["state"]): SnapshotRow {
  return {
    ts,
    state,
    deskHeightCm: 75,
    sittingSecs: 0,
    standingSecs: 0,
    breakSecs: 0,
    idleSecs: 0,
    score: 50,
  };
}

const baseProps = {
  range: { from: "2026-05-15", to: "2026-05-17" },
  selectedDay: "2026-05-17",
  snapshots: [
    snap("2026-05-15T08:00:00Z", "Sitting"),
    snap("2026-05-16T08:00:00Z", "Standing"),
    snap("2026-05-17T08:00:00Z", "Sitting"),
  ],
  today: "2026-05-17",
};

describe("DateNavigator", () => {
  it("renders one column per day in range", () => {
    render(
      <DateNavigator
        {...baseProps}
        onRangeChange={vi.fn()}
        onSelectDay={vi.fn()}
      />,
    );
    expect(screen.getByTestId("day-2026-05-15")).toBeTruthy();
    expect(screen.getByTestId("day-2026-05-16")).toBeTruthy();
    expect(screen.getByTestId("day-2026-05-17")).toBeTruthy();
  });

  it("marks today with a stamp and active tab via aria-pressed", () => {
    render(
      <DateNavigator
        {...baseProps}
        onRangeChange={vi.fn()}
        onSelectDay={vi.fn()}
      />,
    );
    expect(screen.getByTestId("today-stamp").textContent).toContain("today");
    const active = screen.getByTestId("day-2026-05-17");
    expect(active.getAttribute("aria-pressed")).toBe("true");
  });

  it("fires onSelectDay when a tab is clicked", () => {
    const onSelectDay = vi.fn();
    render(
      <DateNavigator
        {...baseProps}
        onRangeChange={vi.fn()}
        onSelectDay={onSelectDay}
      />,
    );
    fireEvent.click(screen.getByTestId("day-2026-05-16"));
    expect(onSelectDay).toHaveBeenCalledWith("2026-05-16");
  });

  it("snaps range when a preset chip is clicked", () => {
    const onRangeChange = vi.fn();
    render(
      <DateNavigator
        {...baseProps}
        onRangeChange={onRangeChange}
        onSelectDay={vi.fn()}
      />,
    );
    fireEvent.click(screen.getByTestId("preset-7d"));
    expect(onRangeChange).toHaveBeenCalledWith({
      from: "2026-05-11",
      to: "2026-05-17",
    });
  });
});
