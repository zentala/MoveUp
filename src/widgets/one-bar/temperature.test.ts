/**
 * temperature.test.ts — tests for temperature escalation logic.
 */
import { describe, it, expect, vi } from "vitest";
import { computeTemperature } from "./temperature";
import type { WidgetProps } from "@/types";

function props(overrides: Partial<WidgetProps> = {}): WidgetProps {
  return {
    connected: true,
    port: "COM3",
    state: "Sitting",
    deskHeightCm: 72.5,
    currentSessionSecs: 600,
    limitSecs: 2400,
    limitRemaining: 1800,
    limitRatio: 0.25,
    breakSecs: 0,
    breakResetThreshold: 600,
    breakResetProgress: 0,
    previousSession: null,
    todaySessions: [],
    todayChanges: 3,
    todayStandingSecs: 0,
    todaySittingSecs: 600,
    todayScore: 0,
    error: null,
    onOpenSettings: vi.fn(),
    ...overrides,
  };
}

describe("computeTemperature", () => {
  it("returns 'calm' when sitting below 50%", () => {
    expect(computeTemperature(props({ limitRatio: 0 }))).toBe("calm");
    expect(computeTemperature(props({ limitRatio: 0.25 }))).toBe("calm");
    expect(computeTemperature(props({ limitRatio: 0.49 }))).toBe("calm");
  });

  it("returns 'warm' when sitting 50-79%", () => {
    expect(computeTemperature(props({ limitRatio: 0.5 }))).toBe("warm");
    expect(computeTemperature(props({ limitRatio: 0.65 }))).toBe("warm");
    expect(computeTemperature(props({ limitRatio: 0.79 }))).toBe("warm");
  });

  it("returns 'hot' when sitting 80-99%", () => {
    expect(computeTemperature(props({ limitRatio: 0.8 }))).toBe("hot");
    expect(computeTemperature(props({ limitRatio: 0.95 }))).toBe("hot");
    expect(computeTemperature(props({ limitRatio: 0.99 }))).toBe("hot");
  });

  it("returns 'burning' when sitting at or over 100%", () => {
    expect(computeTemperature(props({ limitRatio: 1.0 }))).toBe("burning");
    expect(computeTemperature(props({ limitRatio: 1.5 }))).toBe("burning");
  });

  it("returns 'standing' when standing without full reset", () => {
    expect(
      computeTemperature(
        props({ state: "Standing", breakResetProgress: 0.5 }),
      ),
    ).toBe("standing");
  });

  it("returns 'standing' for Walking without full reset", () => {
    expect(
      computeTemperature(
        props({ state: "Walking", breakResetProgress: 0.3 }),
      ),
    ).toBe("standing");
  });

  it("returns 'reset' when standing with full reset", () => {
    expect(
      computeTemperature(
        props({ state: "Standing", breakResetProgress: 1.0 }),
      ),
    ).toBe("reset");
    expect(
      computeTemperature(
        props({ state: "Walking", breakResetProgress: 1.5 }),
      ),
    ).toBe("reset");
  });

  it("returns 'away' when state is Away", () => {
    expect(computeTemperature(props({ state: "Away" }))).toBe("away");
  });

  it("away takes priority over limitRatio", () => {
    expect(
      computeTemperature(props({ state: "Away", limitRatio: 1.5 })),
    ).toBe("away");
  });

  it("standing takes priority over limitRatio", () => {
    expect(
      computeTemperature(
        props({
          state: "Standing",
          limitRatio: 1.0,
          breakResetProgress: 0.2,
        }),
      ),
    ).toBe("standing");
  });
});
