import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { dayComposition } from "@/timeline/intervals";
import { DailyCompositionRing } from "./DailyCompositionRing";

const startMs = Date.parse("2026-03-29T00:00:00Z");
const makeWindow = (hours: number) => ({ startMs, endMs: startMs + hours * 3_600_000 });

describe("DailyCompositionRing", () => {
  it.each([23, 24, 25])("uses a %i-hour full-calendar-day denominator", (hours) => {
    const endMs = startMs + hours * 3_600_000;
    const composition = dayComposition([], makeWindow(hours), endMs);
    render(<DailyCompositionRing date="2026-03-29" composition={composition} />);
    const ring = screen.getByRole("img");
    expect(ring.getAttribute("aria-label")).toContain(`denominator ${(hours * 3600).toFixed(3)} seconds`);
    expect(ring.getAttribute("aria-label")).toContain("Unknown 100.00 percent");
    expect(ring.querySelector('[data-state="Away"]')).toBeNull();
  });

  it("hatches only the mathematically proportional Unknown segment", () => {
    const composition = dayComposition([
      { startMs, endMs: startMs + 3_600_000, state: "Standing", provenance: "session" },
    ], makeWindow(24), startMs + 24 * 3_600_000);
    render(<DailyCompositionRing date="2026-03-29" composition={composition} />);
    const ring = screen.getByRole("img");
    const unknown = ring.querySelector<SVGCircleElement>('[data-state="Unknown"]');
    expect(unknown?.getAttribute("stroke")).toContain("unknown-hatch-");
    const dash = unknown?.getAttribute("stroke-dasharray")?.split(" ").map(Number);
    expect(dash?.[0]).toBeCloseTo(2 * Math.PI * 20 * 23 / 24);
    expect(dash?.[0]).not.toBe(2);
    expect(ring.querySelector("pattern line")).not.toBeNull();
    expect(ring.querySelector('[data-state="Future"]')).toBeNull();
  });

  it("keeps time after now as a distinct unfilled Future segment", () => {
    const nowMs = startMs + 12 * 3_600_000;
    const composition = dayComposition([], makeWindow(24), nowMs);
    render(<DailyCompositionRing date="2026-03-29" composition={composition} />);
    const ring = screen.getByRole("img");
    const future = ring.querySelector<SVGCircleElement>('[data-state="Future"]');
    expect(future?.getAttribute("fill")).toBe("none");
    expect(future?.getAttribute("stroke-width")).toBe("2");
    expect(Number(future?.getAttribute("stroke-dasharray")?.split(" ")[0])).toBeCloseTo(2 * Math.PI * 20 / 2);
    expect(ring.getAttribute("aria-label")).toContain("Future 50.00 percent");
  });

  it("shows unavailable coverage without drawing a fabricated zero ring", () => {
    render(<DailyCompositionRing date="2026-03-29" composition={null} />);
    const ring = screen.getByRole("img");
    expect(ring.getAttribute("aria-label")).toContain("coverage unknown");
    expect(ring.querySelector("circle")).toBeNull();
  });
});
