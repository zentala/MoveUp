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
 *     .withState("Standing")
 *     .build();
 */
export class SessionStateBuilder {
  private data = {
    state: "Sitting" as const,
    sitting_seconds: 0,
    standing_seconds: 0,
    break_seconds: 0,
    session_limit_secs: 2700,
    stand_limit_secs: 0,
    desk_height_cm: 75.0,
    position_changes: 0,
  };

  withState(state: "Sitting" | "Standing" | "Walking" | "Away") {
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

export { testScenarios } from "./scenarios";

export default {
  sessionStateBuilder,
  settingsBuilder,
  todaySummaryBuilder,
};
