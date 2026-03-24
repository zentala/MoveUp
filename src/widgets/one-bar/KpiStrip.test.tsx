import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { KpiStrip } from "./KpiStrip";

const mockMetrics = [
  { id: "standing_pct", label: "\u2195 Standing", result: { value: 15, display: "15%", level: "green" as const, is_personal_best: false } },
  { id: "position_rate", label: "\u21c4 Changes", result: { value: 1.2, display: "1.2/h", level: "green" as const, is_personal_best: true } },
  { id: "hourly_breaks", label: "\u2615 Breaks", result: { value: 0.71, display: "5/7h", level: "green" as const, is_personal_best: false } },
  { id: "longest_session", label: "\ud83d\udc41 Screen", result: { value: 2820, display: "47m", level: "yellow" as const, is_personal_best: false } },
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

  it("renders badges with data-testid per metric", () => {
    render(<KpiStrip metrics={mockMetrics} />);
    expect(screen.getByTestId("kpi-badge-standing_pct")).toBeInTheDocument();
    expect(screen.getByTestId("kpi-badge-position_rate")).toBeInTheDocument();
    expect(screen.getByTestId("kpi-badge-hourly_breaks")).toBeInTheDocument();
    expect(screen.getByTestId("kpi-badge-longest_session")).toBeInTheDocument();
  });

  it("shows PB for personal best", () => {
    render(<KpiStrip metrics={mockMetrics} />);
    expect(screen.getByText("PB")).toBeInTheDocument();
  });

  it("shows dash for missing data", () => {
    const noData = [{ id: "test", label: "Test", result: { value: 0, display: "\u2014", level: "green" as const, is_personal_best: false } }];
    render(<KpiStrip metrics={noData} />);
    expect(screen.getByText("\u2014")).toBeInTheDocument();
  });

  it("renders labels for each metric", () => {
    render(<KpiStrip metrics={mockMetrics} />);
    expect(screen.getByText("\u2195 Standing")).toBeInTheDocument();
    expect(screen.getByText("\u21c4 Changes")).toBeInTheDocument();
    expect(screen.getByText("\u2615 Breaks")).toBeInTheDocument();
    expect(screen.getByText("\ud83d\udc41 Screen")).toBeInTheDocument();
  });
});
