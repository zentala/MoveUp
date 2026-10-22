import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { KpiTrend } from "./KpiTrend";
import { analystDailyKpisFixture } from "@/test/analyst-fixtures";
import type { DailyKpi } from "@/test/analyst-fixtures";

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

  it("renders Unknown for null, preserves a measured zero, and leaves a dated sparkline gap", () => {
    const data: DailyKpi[] = [
      { ...analystDailyKpisFixture[0], dateLocal: "2026-05-15", longestSessionSecs: 600 },
      { ...analystDailyKpisFixture[0], dateLocal: "2026-05-16", longestSessionSecs: null },
      { ...analystDailyKpisFixture[0], dateLocal: "2026-05-17", longestSessionSecs: 0 },
    ];
    const { rerender } = render(<KpiTrend data={data} />);
    const label = screen.getByText("Longest session");
    const card = label.parentElement;
    expect(card).toHaveTextContent("0s");
    const sparkline = card?.querySelector<SVGSVGElement>("svg");
    expect(sparkline?.getAttribute("aria-label")).toContain("gaps mark unknown");
    expect(sparkline?.querySelector("path")?.getAttribute("d")?.match(/M/g)).toHaveLength(2);
    expect(Array.from(sparkline?.querySelectorAll("circle") ?? []).map((point) => point.getAttribute("data-index")))
      .toEqual(["0", "2"]);

    const unknown: DailyKpi[] = [
      { ...data[0], longestSessionSecs: null },
      { ...data[1], longestSessionSecs: null },
    ];
    rerender(<KpiTrend data={unknown} />);
    expect(label.parentElement?.parentElement).toHaveTextContent("Unknown");
  });
});
