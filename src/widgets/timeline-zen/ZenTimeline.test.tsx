/**
 * ZenTimeline.test.tsx — tests for the ZenTimeline component.
 *
 * Verifies timeline rendering, block count, height, and state colors.
 */
import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { ZenTimeline } from "./ZenTimeline";
import type { SessionEntry } from "@/types";

function makeSessions(count: number): SessionEntry[] {
  const now = new Date();
  const sessions: SessionEntry[] = [];
  const states: Array<"Sitting" | "Standing"> = ["Sitting", "Standing"];

  for (let i = count; i > 0; i--) {
    const start = new Date(now.getTime() - i * 20 * 60 * 1000);
    const end = new Date(start.getTime() + 15 * 60 * 1000);
    sessions.push({
      start: start.toISOString(),
      end: end.toISOString(),
      state: states[i % 2],
      duration_secs: 900,
    });
  }
  return sessions;
}

describe("ZenTimeline", () => {
  it("renders without crashing with no sessions", () => {
    render(<ZenTimeline sessions={[]} />);
    expect(screen.getByTestId("zen-timeline")).toBeInTheDocument();
  });

  it("renders correct number of session blocks", () => {
    render(<ZenTimeline sessions={makeSessions(4)} />);
    const blocks = screen.getByTestId("zen-timeline")
      .querySelectorAll(".zen-timeline__block");
    expect(blocks.length).toBe(4);
  });

  it("has zen-timeline class for CSS height", () => {
    render(<ZenTimeline sessions={[]} />);
    expect(screen.getByTestId("zen-timeline").className).toContain("zen-timeline");
  });

  it("applies correct color classes to blocks", () => {
    const sessions: SessionEntry[] = [
      {
        start: new Date(Date.now() - 3600000).toISOString(),
        end: new Date(Date.now() - 1800000).toISOString(),
        state: "Sitting",
        duration_secs: 1800,
      },
      {
        start: new Date(Date.now() - 1800000).toISOString(),
        end: null,
        state: "Standing",
        duration_secs: 1800,
      },
    ];
    render(<ZenTimeline sessions={sessions} />);
    const blocks = screen.getByTestId("zen-timeline")
      .querySelectorAll(".zen-timeline__block");
    expect(blocks[0].className).toContain("--sitting");
    expect(blocks[1].className).toContain("--standing");
  });

  it("renders a now indicator", () => {
    render(<ZenTimeline sessions={[]} />);
    const now = screen.getByTestId("zen-timeline")
      .querySelector(".zen-timeline__now");
    expect(now).toBeTruthy();
  });

  it("blocks have title tooltip with state and duration", () => {
    const sessions: SessionEntry[] = [
      {
        start: new Date(Date.now() - 3600000).toISOString(),
        end: new Date(Date.now() - 1800000).toISOString(),
        state: "Sitting",
        duration_secs: 1800,
      },
    ];
    render(<ZenTimeline sessions={sessions} />);
    const block = screen.getByTestId("zen-timeline")
      .querySelector(".zen-timeline__block") as HTMLElement;
    expect(block.title).toContain("Sitting");
    expect(block.title).toContain("30m");
  });
});
