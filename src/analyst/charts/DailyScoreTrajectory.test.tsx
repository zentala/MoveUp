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

  it("does not assign a local-hour path when source date and offset are missing", () => {
    const unknownSourceDate = {
      ...analystSnapshotsFixture[0],
      localDate: null,
      timezoneOffsetSecs: null,
    };
    const { container } = render(<DailyScoreTrajectory data={[unknownSourceDate]} />);
    expect(container.querySelector("svg")).toBeTruthy();
    expect(container.querySelectorAll("path")).toHaveLength(0);
  });
});
