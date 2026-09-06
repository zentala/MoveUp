import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import { KpiDonut } from "./KpiDonut";
import { KpiDonutPanel } from "./KpiDonutPanel";
import { aggregateDayKpis } from "../explorer-day-kpis";
import {
  analystSessionsFixture,
  analystSnapshotsFixture,
} from "@/test/analyst-fixtures";

function renderDonut(props: Partial<React.ComponentProps<typeof KpiDonut>> = {}) {
  return render(
    <KpiDonut
      label="Standing %"
      valuePct={0.62}
      centerPrimary="62%"
      centerSecondary="4h 32m"
      animate={false}
      {...props}
    />,
  );
}

describe("KpiDonut", () => {
  it("renders an SVG arc with a filled and an unfilled sector", () => {
    const { container } = renderDonut();
    expect(container.querySelector("svg")).toBeTruthy();
    expect(container.querySelectorAll("path.recharts-sector").length).toBe(2);
  });

  it("stacks the primary and secondary values in the centre", () => {
    const { container } = renderDonut();
    const texts = Array.from(container.querySelectorAll("text")).map((t) => t.textContent);
    expect(texts).toContain("62%");
    expect(texts).toContain("4h 32m");
  });

  it("omits the secondary line when not supplied", () => {
    const { container } = renderDonut({ centerSecondary: undefined });
    const texts = Array.from(container.querySelectorAll("text")).map((t) => t.textContent);
    expect(texts).toContain("62%");
    expect(texts).not.toContain("4h 32m");
  });

  it("shows the label under the arc", () => {
    const { getByText } = renderDonut();
    expect(getByText("Standing %")).toBeTruthy();
  });

  it("reflects the fill fraction on the container", () => {
    const { getByTestId } = renderDonut({ valuePct: 0.25 });
    expect(getByTestId("kpi-donut").getAttribute("data-pct")).toBe("0.2500");
  });

  it.each([
    [3, "1.0000"],
    [-2, "0.0000"],
    [NaN, "0.0000"],
  ])("clamps a fill of %s to %s", (valuePct, expected) => {
    const { getByTestId } = renderDonut({ valuePct });
    expect(getByTestId("kpi-donut").getAttribute("data-pct")).toBe(expected);
  });
});

describe("KpiDonutPanel", () => {
  const day = analystSnapshotsFixture[analystSnapshotsFixture.length - 1].ts.slice(0, 10);
  const kpis = aggregateDayKpis(analystSnapshotsFixture, analystSessionsFixture, day);

  it("renders three donuts for a day with data", () => {
    const { getAllByTestId } = render(<KpiDonutPanel kpis={kpis} animate={false} />);
    expect(getAllByTestId("kpi-donut").length).toBe(3);
  });

  it("labels standing, posture changes and score", () => {
    const { getAllByTestId } = render(<KpiDonutPanel kpis={kpis} animate={false} />);
    const labels = getAllByTestId("kpi-donut").map((n) => n.getAttribute("data-label"));
    expect(labels).toEqual(["Standing %", "Posture changes", "Daily score"]);
  });

  it("recomputes when the selected day changes", () => {
    const otherDay = analystSnapshotsFixture[0].ts.slice(0, 10);
    const otherKpis = aggregateDayKpis(
      analystSnapshotsFixture,
      analystSessionsFixture,
      otherDay,
    );
    const { getAllByTestId, rerender } = render(
      <KpiDonutPanel kpis={kpis} animate={false} />,
    );
    const before = getAllByTestId("kpi-donut")[0].getAttribute("data-pct");
    rerender(<KpiDonutPanel kpis={otherKpis} animate={false} />);
    const after = getAllByTestId("kpi-donut")[0].getAttribute("data-pct");
    expect(otherDay).not.toBe(day);
    expect(after).not.toBe(before);
  });

  it("says so instead of drawing empty arcs when the day has no data", () => {
    const empty = aggregateDayKpis([], [], "1999-01-01");
    const { getByTestId, queryAllByTestId } = render(
      <KpiDonutPanel kpis={empty} animate={false} />,
    );
    expect(getByTestId("kpi-donut-panel-empty").textContent).toContain("No data");
    expect(queryAllByTestId("kpi-donut").length).toBe(0);
  });
});
