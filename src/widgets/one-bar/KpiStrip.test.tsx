import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { KpiStrip } from "./KpiStrip";

const mockMetrics = [
  { id: "standing_pct", label: "Standing", result: { value: 15, display: "15%", level: "green" as const, is_personal_best: false } },
  { id: "position_rate", label: "Changes", result: { value: 1.2, display: "1.2/h", level: "green" as const, is_personal_best: true } },
  { id: "hourly_breaks", label: "Breaks", result: { value: 0.71, display: "5/7h", level: "green" as const, is_personal_best: false } },
  { id: "longest_session", label: "Session", result: { value: 2820, display: "47m", level: "yellow" as const, is_personal_best: false } },
];

describe("KpiStrip", () => {
  it("renders 4 metrics", () => {
    render(<KpiStrip metrics={mockMetrics} />);
    expect(screen.getByTestId("kpi-strip")).toBeInTheDocument();
    expect(screen.getByText("15%")).toBeInTheDocument();
    expect(screen.getByText("1.2/h")).toBeInTheDocument();
    expect(screen.getByText("5/7h")).toBeInTheDocument();
    expect(screen.getByText("47m")).toBeInTheDocument();
  });

  it("returns null for empty metrics", () => {
    const { container } = render(<KpiStrip metrics={[]} />);
    expect(container.firstChild).toBeNull();
  });

  it("applies correct color classes", () => {
    render(<KpiStrip metrics={mockMetrics} />);
    const dots = document.querySelectorAll(".kpi-strip__dot");
    expect(dots[0].classList.contains("kpi-strip__dot--green")).toBe(true);
    expect(dots[3].classList.contains("kpi-strip__dot--yellow")).toBe(true);
  });

  it("shows star for personal best", () => {
    render(<KpiStrip metrics={mockMetrics} />);
    expect(screen.getByText("\u2605")).toBeInTheDocument();
  });

  it("shows dash for missing data", () => {
    const noData = [{ id: "test", label: "Test", result: { value: 0, display: "\u2014", level: "green" as const, is_personal_best: false } }];
    render(<KpiStrip metrics={noData} />);
    expect(screen.getByText("\u2014")).toBeInTheDocument();
  });

  it("shows tooltip on hover", () => {
    render(<KpiStrip metrics={mockMetrics} />);
    const items = document.querySelectorAll(".kpi-strip__item");
    expect(items[0].getAttribute("title")).toBe("Standing: 15%");
  });
});
