/**
 * useExponentialPoll.test.ts — pure-function tests for `nextDelay`.
 *
 * The hook itself is integration-tested via StepsWidget; the schedule
 * math is the part where bugs would silently change polling cadence,
 * so it gets its own unit coverage.
 */
import { describe, it, expect } from "vitest";
import { nextDelay } from "./useExponentialPoll";

const LADDER = [60_000, 120_000, 300_000, 600_000, 1_800_000];
const SUCCESS = 5 * 60 * 1000;

describe("nextDelay", () => {
  it("returns successIntervalMs when no failures recorded", () => {
    expect(nextDelay(0, SUCCESS, LADDER)).toBe(SUCCESS);
  });

  it("returns the first ladder step on first failure", () => {
    expect(nextDelay(1, SUCCESS, LADDER)).toBe(LADDER[0]);
  });

  it("climbs the ladder on consecutive failures", () => {
    expect(nextDelay(2, SUCCESS, LADDER)).toBe(LADDER[1]);
    expect(nextDelay(3, SUCCESS, LADDER)).toBe(LADDER[2]);
    expect(nextDelay(4, SUCCESS, LADDER)).toBe(LADDER[3]);
    expect(nextDelay(5, SUCCESS, LADDER)).toBe(LADDER[4]);
  });

  it("caps at the last ladder step beyond ladder length", () => {
    expect(nextDelay(99, SUCCESS, LADDER)).toBe(LADDER[LADDER.length - 1]);
  });

  it("works with an empty ladder when only success interval is meaningful", () => {
    // Defensive: nobody should call with empty ladder + failures > 0,
    // but the function shouldn't crash.
    expect(nextDelay(0, SUCCESS, [])).toBe(SUCCESS);
  });
});
