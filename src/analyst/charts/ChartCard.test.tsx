import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { ChartCard } from "./ChartCard";

describe("ChartCard help", () => {
  it("opens for focus and hover and closes with Escape", () => {
    render(<ChartCard title="Sessions" help="Recorded session explanation"><div>content</div></ChartCard>);
    const button = screen.getByRole("button", { name: "Help: Sessions" });
    fireEvent.focus(button);
    expect(screen.getByRole("tooltip")).toHaveTextContent("Recorded session explanation");
    fireEvent.keyDown(button, { key: "Escape" });
    expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();
    fireEvent.mouseEnter(button.parentElement!);
    expect(screen.getByRole("tooltip")).toBeInTheDocument();
  });
});
