import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import { DailyScoreTrajectory } from "./DailyScoreTrajectory";
import { analystSnapshotsFixture } from "@/test/analyst-fixtures";

describe("DailyScoreTrajectory", () => {
  it("renders an SVG with at least one line path", () => {
    const { container } = render(<DailyScoreTrajectory data={analystSnapshotsFixture} />);
    expect(container.querySelector("svg")).toBeTruthy();
    expect(container.querySelectorAll("path").length).toBeGreaterThan(0);
  });
});
