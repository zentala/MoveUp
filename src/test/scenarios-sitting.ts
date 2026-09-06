/**
 * scenarios-sitting.ts — Sitting state scenarios (S01-S04).
 */
import type { Scenario } from "./scenarios";
import { noop, mkMetric, mkSessions } from "./scenario-helpers";

/** S01: App just launched, sensor connected, desk low, no history. */
export const S01_FRESH_START: Scenario = {
  id: "S01",
  name: "Fresh start",
  description: "App just launched. Sensor connected. Desk is low. No sessions yet.",
  context: "Morning, first launch of the day.",
  props: {
    connected: true, port: "COM3", state: "Sitting", deskHeightCm: 72,
    limitUsedSecs: 0, limitSecs: 2400, standLimitSecs: 900,
    limitRemaining: 2400, limitRatio: 0, breakSecs: 0,
    breakResetThreshold: 600, breakResetProgress: 0,
    previousSession: null, todaySessions: [], todayChanges: 0,
    todayStandingSecs: 0, todaySittingSecs: 0, todayScore: 0,
    metrics: [], error: null, idleSecs: 0, continuousComputerSecs: 0, onOpenSettings: noop,
  },
};

/** S02: Sitting for 20 min — green zone, everything calm. */
export const S02_SITTING_GREEN: Scenario = {
  id: "S02",
  name: "Sitting — green zone",
  description: "Sitting for 20 min. Half of 40-min limit. All good.",
  context: "Mid-morning, focused work.",
  props: {
    connected: true, port: "COM3", state: "Sitting", deskHeightCm: 72,
    limitUsedSecs: 1200, limitSecs: 2400, standLimitSecs: 900,
    limitRemaining: 1200, limitRatio: 0.5, breakSecs: 0,
    breakResetThreshold: 600, breakResetProgress: 0,
    previousSession: null,
    todaySessions: mkSessions([{ state: "Standing", mins: 10 }]),
    todayChanges: 1, todayStandingSecs: 600, todaySittingSecs: 1200, todayScore: 10,
    metrics: [
      mkMetric("standing_pct", "\u2195 Standing", "—", "green"),
      mkMetric("position_rate", "\u21c4 Changes", "—", "green"),
      mkMetric("hourly_breaks", "\u2615 Breaks", "—", "green"),
      mkMetric("longest_session", "\ud83d\udc41 Screen", "20m", "green"),
    ],
    error: null, idleSecs: 0, continuousComputerSecs: 0, onOpenSettings: noop,
  },
};

/** S03: Sitting for 35 min — yellow zone, limit approaching. */
export const S03_SITTING_YELLOW: Scenario = {
  id: "S03",
  name: "Sitting — yellow zone",
  description: "35 min sitting. 87% of limit. Yellow warning.",
  context: "User should think about standing soon.",
  props: {
    connected: true, port: "COM3", state: "Sitting", deskHeightCm: 72,
    limitUsedSecs: 2100, limitSecs: 2400, standLimitSecs: 900,
    limitRemaining: 300, limitRatio: 0.875, breakSecs: 0,
    breakResetThreshold: 600, breakResetProgress: 0,
    previousSession: { state: "Standing", durationSecs: 600, wasEffective: true },
    todaySessions: mkSessions([
      { state: "Sitting", mins: 40 }, { state: "Standing", mins: 10 },
      { state: "Sitting", mins: 35 },
    ]),
    todayChanges: 2, todayStandingSecs: 600, todaySittingSecs: 4500, todayScore: 5,
    metrics: [
      mkMetric("standing_pct", "\u2195 Standing", "12%", "yellow"),
      mkMetric("position_rate", "\u21c4 Changes", "0.8/h", "yellow"),
      mkMetric("hourly_breaks", "\u2615 Breaks", "1/3h", "yellow"),
      mkMetric("longest_session", "\ud83d\udc41 Screen", "35m", "green"),
    ],
    error: null, idleSecs: 0, continuousComputerSecs: 0, onOpenSettings: noop,
  },
};

/** S04: Sitting limit exceeded — red, overtime. */
export const S04_SITTING_OVERTIME: Scenario = {
  id: "S04",
  name: "Sitting — overtime!",
  description: "45 min sitting. 5 min past 40-min limit. Red alert.",
  context: "User should stand NOW. Alert popup may be showing.",
  props: {
    connected: true, port: "COM3", state: "Sitting", deskHeightCm: 72,
    limitUsedSecs: 2700, limitSecs: 2400, standLimitSecs: 900,
    limitRemaining: -300, limitRatio: 1.125, breakSecs: 0,
    breakResetThreshold: 600, breakResetProgress: 0,
    previousSession: { state: "Standing", durationSecs: 300, wasEffective: true },
    todaySessions: mkSessions([
      { state: "Sitting", mins: 40 }, { state: "Standing", mins: 5 },
      { state: "Sitting", mins: 45 },
    ]),
    todayChanges: 2, todayStandingSecs: 300, todaySittingSecs: 5100, todayScore: -5,
    metrics: [
      mkMetric("standing_pct", "\u2195 Standing", "6%", "red"),
      mkMetric("position_rate", "\u21c4 Changes", "0.7/h", "yellow"),
      mkMetric("hourly_breaks", "\u2615 Breaks", "0/3h", "red"),
      mkMetric("longest_session", "\ud83d\udc41 Screen", "45m", "yellow"),
    ],
    error: null, idleSecs: 0, continuousComputerSecs: 0, onOpenSettings: noop,
  },
};
