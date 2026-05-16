import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import { DeskHeightTimeline } from "./DeskHeightTimeline";
import { analystSnapshotsFixture } from "@/test/analyst-fixtures";

describe("DeskHeightTimeline", () => {
  it("renders an SVG with a path when given snapshots", () => {
    const { container } = render(<DeskHeightTimeline data={analystSnapshotsFixture} />);
    const svg = container.querySelector("svg");
    expect(svg).toBeTruthy();
    expect(container.querySelectorAll("path").length).toBeGreaterThan(0);
  });

  it("renders an empty state when no data is provided", () => {
    const { container } = render(<DeskHeightTimeline data={[]} />);
    const svg = container.querySelector("svg");
    expect(svg).toBeTruthy();
    expect(container.textContent).toContain("No data");
  });
});
