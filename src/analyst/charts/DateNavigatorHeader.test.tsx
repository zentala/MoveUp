import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { DateNavigatorHeader } from "./DateNavigatorHeader";

describe("DateNavigatorHeader", () => {
  it("names each date boundary and marks a preset only when both endpoints match", () => {
    const onRangeChange = vi.fn();
    render(<DateNavigatorHeader range={{ from: "2026-05-01", to: "2026-05-07" }}
      onRangeChange={onRangeChange} onToday={vi.fn()} today="2026-05-17" />);

    expect(screen.getByLabelText("From")).toHaveValue("2026-05-01");
    expect(screen.getByLabelText("To")).toHaveValue("2026-05-07");
    expect(screen.getByRole("button", { name: "7d" })).toHaveAttribute("aria-pressed", "false");
    expect(screen.getByRole("button", { name: "14d" })).toHaveAttribute("aria-pressed", "false");
  });

  it("moves a selected recent-range preset to the current endpoint and exposes Today", () => {
    const onRangeChange = vi.fn();
    const onToday = vi.fn();
    render(<DateNavigatorHeader range={{ from: "2026-05-11", to: "2026-05-17" }}
      onRangeChange={onRangeChange} onToday={onToday} today="2026-05-17" />);

    expect(screen.getByRole("button", { name: "7d" })).toHaveAttribute("aria-pressed", "true");
    fireEvent.click(screen.getByRole("button", { name: "Today" }));
    expect(onToday).toHaveBeenCalledOnce();
    fireEvent.click(screen.getByRole("button", { name: "14d" }));
    expect(onRangeChange).toHaveBeenCalledWith({ from: "2026-05-04", to: "2026-05-17" });
  });
});
