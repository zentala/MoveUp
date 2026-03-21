/**
 * OneBarCoach.test.tsx — tests for coach message selection logic.
 */
import { describe, it, expect, vi } from "vitest";
import { selectCoachMessage } from "./OneBarCoach";
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

describe("selectCoachMessage", () => {
  describe("sitting states", () => {
    it("returns 'On track.' when ratio < 0.5", () => {
      expect(selectCoachMessage(props({ limitRatio: 0.3 }))).toBe(
        "On track.",
      );
    });

    it("mentions changes count when ratio 0.5-0.8", () => {
      const msg = selectCoachMessage(
        props({ limitRatio: 0.6, todayChanges: 5 }),
      );
      expect(msg).toContain("Half limit");
      expect(msg).toContain("5 changes");
    });

    it("tells to stand when ratio 0.8-1.0", () => {
      const msg = selectCoachMessage(
        props({ limitRatio: 0.9, limitRemaining: 240 }),
      );
      expect(msg).toContain("Stand up within");
      expect(msg).toContain("min");
    });

    it("warns about overtime when ratio >= 1.0", () => {
      const msg = selectCoachMessage(
        props({ limitRatio: 1.2, limitRemaining: -300 }),
      );
      expect(msg).toContain("Limit exceeded");
      expect(msg).toContain("Losing points");
    });
  });

  describe("standing states", () => {
    it("shows time to reset when not yet reset", () => {
      const msg = selectCoachMessage(
        props({
          state: "Standing",
          breakResetProgress: 0.5,
          breakResetThreshold: 600,
        }),
      );
      expect(msg).toContain("min left until reset");
    });

    it("announces reset when breakResetProgress >= 1.0", () => {
      const msg = selectCoachMessage(
        props({
          state: "Standing",
          breakResetProgress: 1.0,
          limitSecs: 2400,
        }),
      );
      expect(msg).toContain("Reset!");
      expect(msg).toContain("40 min");
    });

    it("works for Walking state too", () => {
      const msg = selectCoachMessage(
        props({
          state: "Walking",
          breakResetProgress: 1.0,
          limitSecs: 2400,
        }),
      );
      expect(msg).toContain("Reset!");
    });
  });

  describe("away state", () => {
    it("acknowledges break when not yet reset", () => {
      const msg = selectCoachMessage(
        props({
          state: "Away",
          breakResetProgress: 0.3,
          breakResetThreshold: 600,
        }),
      );
      expect(msg).toContain("Break counts");
      expect(msg).toContain("min to reset");
    });

    it("announces reset when breakResetProgress >= 1.0", () => {
      const msg = selectCoachMessage(
        props({
          state: "Away",
          breakResetProgress: 1.0,
          limitSecs: 2400,
        }),
      );
      expect(msg).toContain("Reset!");
      expect(msg).toContain("Come back");
    });
  });
});
