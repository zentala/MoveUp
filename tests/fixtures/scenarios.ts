/**
 * scenarios.ts — Predefined test scenarios using builders.
 *
 * Extracted from builders.ts to keep files under 250 lines.
 */
import {
  sessionStateBuilder,
  todaySummaryBuilder,
} from "./builders";

export const testScenarios = {
  /** User sitting for exactly half the session limit. */
  halfwaySitting: () =>
    sessionStateBuilder()
      .withState("Sitting")
      .withSittingSeconds(1350) // 45 min limit / 2
      .build(),

  /** User sitting at the exact alert threshold (85%). */
  criticalSitting: () =>
    sessionStateBuilder()
      .withState("Sitting")
      .withSittingSeconds(2295) // 2700 * 0.85
      .build(),

  /** User taking a short break (under 5 min). */
  shortBreak: () =>
    sessionStateBuilder()
      .withState("Standing")
      .withBreakSeconds(240) // 4 minutes
      .build(),

  /** User taking a medium break (5-9 min) — gets partial credit. */
  mediumBreak: () =>
    sessionStateBuilder()
      .withState("Standing")
      .withBreakSeconds(420) // 7 minutes
      .build(),

  /** User taking a long break (10+ min) — gets full reset. */
  longBreak: () =>
    sessionStateBuilder()
      .withState("Standing")
      .withBreakSeconds(600) // 10 minutes
      .build(),

  /** User walking around (high desk, no keyboard activity). */
  walking: () =>
    sessionStateBuilder()
      .withState("Walking")
      .withBreakSeconds(120)
      .build(),

  /** Full day with balanced sit/stand ratio. */
  balancedDay: () =>
    todaySummaryBuilder()
      .withSittingTime(14400) // 4 hours
      .withStandingTime(7200) // 2 hours
      .withPositionChanges(12)
      .build(),

  /** Day with too much sitting (poor ergonomics). */
  overSittingDay: () =>
    todaySummaryBuilder()
      .withSittingTime(28800) // 8 hours
      .withStandingTime(3600) // 1 hour
      .withPositionChanges(4)
      .build(),
};
