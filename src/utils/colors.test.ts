/**
 * colors.test.ts — unit tests for the canonical color palette.
 *
 * Verifies that TypeScript color values match Rust colors.rs.
 */
import { describe, it, expect } from "vitest";
import {
  THRESHOLD_YELLOW,
  THRESHOLD_RED,
  NEUTRAL,
  SITTING_YELLOW,
  SITTING_RED,
  AWAY_GRAY,
  sittingColorForRatio,
} from "@/utils/colors";

describe("color constants match Rust colors.rs", () => {
  it("thresholds match", () => {
    expect(THRESHOLD_YELLOW).toBe(0.60);
    expect(THRESHOLD_RED).toBe(0.85);
  });

  it("neutral color matches", () => {
    expect(NEUTRAL).toBe("#5a5548");
  });

  it("sitting warning colors match", () => {
    expect(SITTING_YELLOW).toBe("#ffc107");
    expect(SITTING_RED).toBe("#f44336");
  });

  it("away color is gray", () => {
    expect(AWAY_GRAY).toBe("#808080");
  });
});

describe("sittingColorForRatio", () => {
  it("returns neutral below 60%", () => {
    expect(sittingColorForRatio(0)).toBe("#5a5548");
    expect(sittingColorForRatio(0.3)).toBe("#5a5548");
    expect(sittingColorForRatio(0.59)).toBe("#5a5548");
  });

  it("returns yellow at 60-84%", () => {
    expect(sittingColorForRatio(0.60)).toBe("#ffc107");
    expect(sittingColorForRatio(0.70)).toBe("#ffc107");
    expect(sittingColorForRatio(0.84)).toBe("#ffc107");
  });

  it("returns red at 85%+", () => {
    expect(sittingColorForRatio(0.85)).toBe("#f44336");
    expect(sittingColorForRatio(0.95)).toBe("#f44336");
    expect(sittingColorForRatio(1.0)).toBe("#f44336");
  });
});
