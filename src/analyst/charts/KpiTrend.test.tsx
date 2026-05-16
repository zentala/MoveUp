import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { KpiTrend } from "./KpiTrend";
import { analystDailyKpisFixture } from "@/test/analyst-fixtures";

describe("KpiTrend", () => {
  it("renders one sparkline SVG per KPI", () => {
    const { container } = render(<KpiTrend data={analystDailyKpisFixture} />);
    const grid = screen.getByTestId("kpi-trend-grid");
    expect(grid).toBeTruthy();
    const svgs = container.querySelectorAll("svg");
    // Each of 4 KPIs renders a sparkline svg.
    expect(svgs.length).toBeGreaterThanOrEqual(4);
  });

  it("shows the four KPI labels", () => {
    const { container } = render(<KpiTrend data={analystDailyKpisFixture} />);
    expect(container.textContent).toContain("Standing %");
    expect(container.textContent).toContain("Changes");
    expect(container.textContent).toContain("Longest session");
    expect(container.textContent).toContain("Score");
  });
});
