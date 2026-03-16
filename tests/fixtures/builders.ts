/**
 * builders.ts — Test data builders for creating consistent test fixtures.
 *
 * Provides builder pattern for test objects to avoid repetitive setup
 * and improve readability of integration/E2E tests.
 */

/**
 * Builder for SessionState test fixtures.
 * Usage:
 *   const state = sessionStateBuilder()
 *     .withSittingSeconds(1800)
 *     .withState("standing")
 *     .build();
 */
export class SessionStateBuilder {
  private data = {
    state: "sitting" as const,
    sitting_seconds: 0,
    standing_seconds: 0,
    break_seconds: 0,
    session_limit_secs: 2700,
    stand_limit_secs: 0,
    desk_height_cm: 75.0,
    position_changes: 0,
  };

  withState(state: "sitting" | "standing" | "walking" | "away") {
    this.data.state = state;
    return this;
  }

  withSittingSeconds(seconds: number) {
    this.data.sitting_seconds = seconds;
    return this;
  }

  withStandingSeconds(seconds: number) {
    this.data.standing_seconds = seconds;
    return this;
  }

  withBreakSeconds(seconds: number) {
    this.data.break_seconds = seconds;
    return this;
  }

  withSessionLimit(minutes: number) {
    this.data.session_limit_secs = minutes * 60;
    return this;
  }

  withStandLimit(minutes: number) {
    this.data.stand_limit_secs = minutes * 60;
    return this;
  }

  withDeskHeight(cm: number) {
    this.data.desk_height_cm = cm;
    return this;
  }

  withPositionChanges(count: number) {
    this.data.position_changes = count;
    return this;
  }

  build() {
    return { ...this.data };
  }
}

/**
 * Factory for SessionStateBuilder.
 */
export function sessionStateBuilder() {
  return new SessionStateBuilder();
}

/**
 * Builder for AppConfig test fixtures.
 */
export class SettingsBuilder {
  private data = {
    sitting_mm: 750,
    standing_mm: 1050,
    desk_thickness_mm: 30,
    sit_limit_mins: 40,
    stand_limit_mins: 0,
    notify_inactivity: true,
    notify_daily_posture_balance: true,
    notify_praise_halfway: true,
  };

  withSittingHeight(mm: number) {
    this.data.sitting_mm = mm;
    return this;
  }

  withStandingHeight(mm: number) {
    this.data.standing_mm = mm;
    return this;
  }

  withDeskThickness(mm: number) {
    this.data.desk_thickness_mm = mm;
    return this;
  }

  withSitLimit(minutes: number) {
    this.data.sit_limit_mins = minutes;
    return this;
  }

  withStandLimit(minutes: number) {
    this.data.stand_limit_mins = minutes;
    return this;
  }

  withNotifications(options: Partial<Record<string, boolean>>) {
    if (options.inactivity !== undefined) this.data.notify_inactivity = options.inactivity;
    if (options.posture !== undefined) this.data.notify_daily_posture_balance = options.posture;
    if (options.praise !== undefined) this.data.notify_praise_halfway = options.praise;
    return this;
  }

  build() {
    return { ...this.data };
  }
}

/**
 * Factory for SettingsBuilder.
 */
export function settingsBuilder() {
  return new SettingsBuilder();
}

/**
 * Builder for TodaySummary test fixtures.
 */
export class TodaySummaryBuilder {
  private data = {
    sitting_secs: 0,
    standing_secs: 0,
    yesterday_sitting_secs: 0,
    yesterday_standing_secs: 0,
    position_changes: 0,
    sessions: [] as any[],
  };

  withSittingTime(seconds: number) {
    this.data.sitting_secs = seconds;
    return this;
  }

  withStandingTime(seconds: number) {
    this.data.standing_secs = seconds;
    return this;
  }

  withYesterdayTotals(sitting: number, standing: number) {
    this.data.yesterday_sitting_secs = sitting;
    this.data.yesterday_standing_secs = standing;
    return this;
  }

  withPositionChanges(count: number) {
    this.data.position_changes = count;
    return this;
  }

  addSession(session: any) {
    this.data.sessions.push(session);
    return this;
  }

  build() {
    return { ...this.data };
  }
}

/**
 * Factory for TodaySummaryBuilder.
 */
export function todaySummaryBuilder() {
  return new TodaySummaryBuilder();
}

/**
 * Predefined test scenarios using builders.
 */
export const testScenarios = {
  /**
   * User sitting for exactly half the session limit.
   */
  halfwaySitting: () =>
    sessionStateBuilder()
      .withState("sitting")
      .withSittingSeconds(1350) // 45 min limit / 2
      .build(),

  /**
   * User sitting at the exact alert threshold (85%).
   */
  criticalSitting: () =>
    sessionStateBuilder()
      .withState("sitting")
      .withSittingSeconds(2295) // 2700 * 0.85
      .build(),

  /**
   * User taking a short break (under 5 min).
   */
  shortBreak: () =>
    sessionStateBuilder()
      .withState("standing")
      .withBreakSeconds(240) // 4 minutes
      .build(),

  /**
   * User taking a medium break (5-9 min) — gets partial credit.
   */
  mediumBreak: () =>
    sessionStateBuilder()
      .withState("standing")
      .withBreakSeconds(420) // 7 minutes
      .build(),

  /**
   * User taking a long break (10+ min) — gets full reset.
   */
  longBreak: () =>
    sessionStateBuilder()
      .withState("standing")
      .withBreakSeconds(600) // 10 minutes
      .build(),

  /**
   * User walking around (high desk, no keyboard activity).
   */
  walking: () =>
    sessionStateBuilder()
      .withState("walking")
      .withBreakSeconds(120)
      .build(),

  /**
   * Full day with balanced sit/stand ratio.
   */
  balancedDay: () =>
    todaySummaryBuilder()
      .withSittingTime(14400) // 4 hours
      .withStandingTime(7200) // 2 hours
      .withPositionChanges(12)
      .build(),

  /**
   * Day with too much sitting (poor ergonomics).
   */
  overSittingDay: () =>
    todaySummaryBuilder()
      .withSittingTime(28800) // 8 hours
      .withStandingTime(3600) // 1 hour
      .withPositionChanges(4)
      .build(),
};

export default {
  sessionStateBuilder,
  settingsBuilder,
  todaySummaryBuilder,
  testScenarios,
};
