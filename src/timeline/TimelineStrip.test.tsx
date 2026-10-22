import { act, fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { TimelineStrip } from "./TimelineStrip";
import { observationMarkers, type TimelineInterval } from "./intervals";

const start = Date.parse("2026-10-09T00:00:00+02:00");
const window = { startMs: start, endMs: start + 86_400_000 };
const intervals: TimelineInterval[] = [
  { startMs: start + 32_400_000, endMs: start + 33_000_000, state: "Sitting", provenance: "session" },
  { startMs: start + 33_000_000, endMs: start + 34_116_580, state: "Away", provenance: "session" },
  { startMs: start + 34_116_580, endMs: start + 34_116_590, state: "Standing", provenance: "snapshot" },
];

describe("shared timeline renderer", () => {
  it("uses identical endpoints, proportions and posture colors in compact and detail modes", () => {
    const { rerender } = render(<TimelineStrip intervals={intervals} window={window} nowMs={window.endMs} mode="compact" />);
    const compact = screen.getAllByTestId("timeline-segment").map((node) => [
      node.dataset.state, node.dataset.startMs, node.dataset.endMs, (node as HTMLElement).style.width,
      (node as HTMLElement).style.background,
    ]);
    rerender(<TimelineStrip intervals={intervals} window={window} nowMs={window.endMs} mode="detail" />);
    expect(screen.getAllByTestId("timeline-segment").map((node) => [
      node.dataset.state, node.dataset.startMs, node.dataset.endMs, (node as HTMLElement).style.width,
      (node as HTMLElement).style.background,
    ])).toEqual(compact);
    expect(parseFloat((screen.getAllByTestId("timeline-segment")[2] as HTMLElement).style.width))
      .toBeCloseTo(10 / 86_400_000 * 100, 12);
  });

  it("shows exact source, endpoints and fractional duration after click or keyboard focus", () => {
    render(<TimelineStrip intervals={intervals} window={window} nowMs={window.endMs} />);
    const away = screen.getAllByTestId("timeline-segment")[1];
    fireEvent.click(away);
    expect(screen.getByTestId("timeline-interval-detail")).toHaveTextContent("18m 36.58s");
    expect(screen.getByTestId("timeline-interval-detail")).toHaveTextContent("source: session");
    fireEvent.keyDown(away, { key: "Escape" });
    expect(screen.queryByTestId("timeline-interval-detail")).not.toBeInTheDocument();
    fireEvent.focus(away);
    expect(screen.getByRole("tooltip")).toHaveTextContent("Away");
  });

  it("makes intervals keyboard reachable and exposes Unknown and Future in the details list", () => {
    render(<TimelineStrip intervals={intervals} window={window} nowMs={start + 43_200_000} />);
    const segments = screen.getAllByTestId("timeline-segment");
    act(() => segments[0].focus());
    fireEvent.keyDown(segments[0], { key: "End" });
    expect(document.activeElement).toBe(segments[2]);
    fireEvent.keyDown(segments[2], { key: "Home" });
    expect(document.activeElement).toBe(segments[0]);
    expect(screen.getByRole("combobox", { name: "Interval details" })).toHaveTextContent("Unknown");
    expect(screen.getByRole("combobox", { name: "Interval details" })).toHaveTextContent("Future");
    const unknown = screen.getAllByTestId("timeline-unknown");
    expect(unknown).toHaveLength(2);
    expect(unknown[0]).toHaveAttribute("aria-label", expect.stringContaining("9h 0s"));
    expect(unknown[1]).toHaveAttribute("aria-label", expect.stringContaining("2h 31m 23.41s"));
    expect(screen.getByTestId("timeline-future")).toBeInTheDocument();
  });

  it("renders conflicting source evidence as a data error", () => {
    render(<TimelineStrip intervals={[intervals[0], { ...intervals[0], state: "Standing" }]}
      window={window} nowMs={window.endMs} />);
    expect(screen.getByRole("alert")).toHaveTextContent("Conflicting overlapping");
  });

  it("renders a 2px by 6px known observation tick without painting elapsed duration", () => {
    const at = start + 60_000;
    const markers = observationMarkers([{
      ts: new Date(at).toISOString(),
      state: "standing",
      observationQuality: "known",
      connected: true,
      localDate: "2026-10-09",
      timezoneOffsetSecs: 7200,
    }]);
    render(<TimelineStrip intervals={[]} markers={markers} window={window} nowMs={window.endMs} />);
    const marker = screen.getByTestId("timeline-observation-marker");
    expect(marker).toHaveAttribute("data-at-ms", String(at));
    expect(marker.getAttribute("title")).toContain("Standing");
    expect(marker.getAttribute("title")).toContain("duration unknown");
    expect((marker as HTMLElement).style.width).toBe("2px");
    expect((marker as HTMLElement).style.height).toBe("6px");
    expect((marker as HTMLElement).style.top).toBe("0px");
    expect(screen.queryByTestId("timeline-segment")).not.toBeInTheDocument();
    fireEvent.change(screen.getByRole("combobox", { name: "Interval details" }), {
      target: { value: `observation:${at}` },
    });
    expect(screen.getByTestId("timeline-interval-detail")).toHaveTextContent("Observation instant");
    expect(screen.getByTestId("timeline-interval-detail")).toHaveTextContent("duration unknown");
    expect(screen.getByTestId("timeline-interval-detail")).toHaveTextContent("source local date: 2026-10-09");
  });

  it("uses secondary ink for a point without known quality/state", () => {
    const at = start + 60_000;
    const markers = observationMarkers([{
      ts: new Date(at).toISOString(), state: "away", observationQuality: "unknown", connected: true,
    }]);
    render(<TimelineStrip intervals={[]} markers={markers} window={window} nowMs={window.endMs} />);
    const marker = screen.getByTestId("timeline-observation-marker") as HTMLElement;
    expect(marker.style.background).toBe("var(--ink-secondary, #c6c6c6)");
    expect(marker.getAttribute("title")).toContain("Unknown");
    expect(marker.getAttribute("title")).toContain("duration unknown");
  });
});
