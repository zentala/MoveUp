import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { StateGantt } from "./StateGantt";
import { analystSnapshotsFixture } from "@/test/analyst-fixtures";

describe("StateGantt", () => {
  it("renders an SVG with rects for state segments", () => {
    const { container } = render(<StateGantt data={analystSnapshotsFixture} />);
    expect(container.querySelector("svg")).toBeTruthy();
    expect(container.querySelectorAll("rect").length).toBeGreaterThan(0);
  });

  it("renders a legend with all four states", () => {
    render(<StateGantt data={analystSnapshotsFixture} />);
    const legend = screen.getByTestId("legend");
    expect(legend.textContent).toContain("Sitting");
    expect(legend.textContent).toContain("Standing");
    expect(legend.textContent).toContain("Walking");
    expect(legend.textContent).toContain("Away");
  });
});
