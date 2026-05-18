/**
 * TimelineDetail.test.tsx — smoke tests for the continuous timeline strip.
 */
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { SnapshotRow } from "@/test/analyst-fixtures";
import { TimelineDetail } from "./TimelineDetail";

function snap(ts: string, state: SnapshotRow["state"]): SnapshotRow {
  return {
    ts,
    state,
    deskHeightCm: 75,
    sittingSecs: 0,
    standingSecs: 0,
    breakSecs: 0,
    idleSecs: 0,
    score: 50,
  };
}

const range = { from: "2026-05-16", to: "2026-05-17" };
const snapshots = [
  snap("2026-05-17T08:00:00Z", "Sitting"),
  snap("2026-05-17T09:00:00Z", "Standing"),
];

describe("TimelineDetail", () => {
  it("renders the active weekday and date in header", () => {
    render(
      <TimelineDetail
        range={range}
        selectedDay="2026-05-17"
        snapshots={snapshots}
        onPrev={vi.fn()}
        onNext={vi.fn()}
        nowMs={new Date("2026-05-17T14:30:00Z").getTime()}
      />,
    );
    // 2026-05-17 is a Sunday (Date locale dependent — accept any weekday).
    expect(screen.getByTestId("tl-weekday").textContent?.length ?? 0).toBeGreaterThan(0);
  });

  it("fires onPrev/onNext from nav buttons", () => {
    const onPrev = vi.fn();
    const onNext = vi.fn();
    render(
      <TimelineDetail
        range={range}
        selectedDay="2026-05-17"
        snapshots={snapshots}
        onPrev={onPrev}
        onNext={onNext}
      />,
    );
    fireEvent.click(screen.getByTestId("tl-prev"));
    fireEvent.click(screen.getByTestId("tl-next"));
    expect(onPrev).toHaveBeenCalledTimes(1);
    expect(onNext).toHaveBeenCalledTimes(1);
  });

  it("mounts the scrollable strip container", () => {
    render(
      <TimelineDetail
        range={range}
        selectedDay="2026-05-17"
        snapshots={snapshots}
        onPrev={vi.fn()}
        onNext={vi.fn()}
      />,
    );
    expect(screen.getByTestId("timeline-scroller")).toBeTruthy();
  });
});
