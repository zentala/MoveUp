import { describe, expect, it } from "vitest";
import { render } from "@testing-library/react";
import type { SnapshotRow } from "@/test/analyst-fixtures";
import { DailyScoreTrajectory } from "./DailyScoreTrajectory";

function sample(
  ts: string,
  score: number | null,
  overrides: Partial<SnapshotRow> = {},
): SnapshotRow {
  return {
    ts,
    state: null,
    deskHeightCm: 0,
    sittingSecs: 0,
    standingSecs: 0,
    breakSecs: 0,
    idleSecs: 0,
    score,
    localDate: "2026-10-22",
    timezoneOffsetSecs: 0,
    ...overrides,
  };
}

describe("DailyScoreTrajectory", () => {
  it("renders every signed recorded score, including a measured zero, without connecting samples", () => {
    const rows = [
      sample("2026-10-22T09:00:00Z", -90),
      sample("2026-10-22T09:01:00Z", 0),
      sample("2026-10-22T12:00:00Z", 150),
    ];
    const { container, getByText } = render(<DailyScoreTrajectory data={rows} />);

    expect(container.querySelectorAll("circle.score-sample")).toHaveLength(3);
    expect(container.querySelector("path.score-line")).toBeNull();
    expect(container.textContent).toContain("3 recorded samples");
    expect(container.textContent).toContain("0 unknown");
    expect(container.querySelector("svg")?.getAttribute("aria-label")).toContain("signed");
    expect(getByText("−90")).toBeTruthy();
    expect(getByText("+150")).toBeTruthy();
    expect(container.querySelector('circle[aria-label*="score −90"]')).toBeTruthy();
    expect(container.querySelector('circle[aria-label*="score 0"]')).toBeTruthy();
  });

  it("keeps null and source-metadata gaps unknown across missing hours and source-local midnight", () => {
    const rows = [
      sample("2026-10-22T22:30:00Z", -2, {
        localDate: "2026-10-23",
        timezoneOffsetSecs: 7200,
      }),
      sample("2026-10-22T22:31:00Z", null, {
        localDate: "2026-10-23",
        timezoneOffsetSecs: 7200,
      }),
      sample("2026-10-23T01:00:00Z", 5, {
        localDate: "2026-10-23",
        timezoneOffsetSecs: 7200,
      }),
      sample("2026-10-23T02:00:00Z", 7, { timezoneOffsetSecs: null }),
      sample("bad timestamp", 8),
      sample("2026-10-23T01:00:00Z", 9, {
        localDate: "2026-10-22",
        timezoneOffsetSecs: 7200,
      }),
    ];
    const { container } = render(<DailyScoreTrajectory data={rows} />);

    expect(container.querySelectorAll("circle.score-sample")).toHaveLength(2);
    expect(container.textContent).toContain("2 recorded samples");
    expect(container.textContent).toContain("4 unknown");
    expect(container.textContent).toContain("2026-10-23 00:30:00 UTC+02:00");
    expect(container.querySelector("path.score-line")).toBeNull();
    expect(container.textContent).not.toContain("2026-10-22 22:30:00");
    expect(container.querySelector('circle[data-score="9"]')).toBeNull();
  });

  it("preserves score samples when posture coverage is unknown and distinguishes an empty range", () => {
    const row = sample("2026-10-22T09:00:00Z", 4, {
      observationQuality: null,
    });
    const rendered = render(<DailyScoreTrajectory data={[row]} />);
    expect(rendered.container.querySelectorAll("circle.score-sample")).toHaveLength(1);

    const empty = render(<DailyScoreTrajectory data={[]} />);
    expect(empty.container.textContent).toContain("No score samples recorded");
  });
});
