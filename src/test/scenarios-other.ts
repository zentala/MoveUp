/**
 * scenarios-other.ts — Standing, Away, and edge case scenarios (S05-S09).
 */
import type { Scenario } from "./scenarios";
import type { VoiceAck } from "@/hooks/useRemoteDesk";
import { noop, mkMetric, mkSessions } from "./scenario-helpers";

/** S05: Standing for 8 min — gold bar filling toward 15 min target. */
export const S05_STANDING_MID: Scenario = {
  id: "S05",
  name: "Standing — midway",
  description: "Standing for 8 min. Gold bar at 53% toward 15-min target.",
  context: "User stood up, taking a break. Timer counts standing time.",
  props: {
    connected: true, port: "COM3", state: "Standing", deskHeightCm: 105,
    limitUsedSecs: 0, limitSecs: 2400, standLimitSecs: 900,
    limitRemaining: 2400, limitRatio: 0, breakSecs: 480,
    breakResetThreshold: 600, breakResetProgress: 0.8,
    previousSession: { state: "Sitting", durationSecs: 2400, wasEffective: false },
    todaySessions: mkSessions([{ state: "Sitting", mins: 40 }]),
    todayChanges: 1, todayStandingSecs: 480, todaySittingSecs: 2400, todayScore: 8,
    metrics: [
      mkMetric("standing_pct", "\u2195 Standing", "17%", "green"),
      mkMetric("position_rate", "\u21c4 Changes", "1.0/h", "green"),
      mkMetric("hourly_breaks", "\u2615 Breaks", "0/1h", "red"),
      mkMetric("longest_session", "\ud83d\udc41 Screen", "48m", "yellow"),
    ],
    error: null, idleSecs: 0, continuousComputerSecs: 0, onOpenSettings: noop,
  },
};

/** S06: Away — user left the desk, app detected inactivity. */
export const S06_AWAY: Scenario = {
  id: "S06",
  name: "Away from desk",
  description: "User left 12 min ago. Desk low. Gray UI, no counting.",
  context: "User went to kitchen/bathroom. App should NOT alert.",
  props: {
    connected: true, port: "COM3", state: "Away", deskHeightCm: 72,
    limitUsedSecs: 0, limitSecs: 2400, standLimitSecs: 900,
    limitRemaining: 2400, limitRatio: 0, breakSecs: 720,
    breakResetThreshold: 600, breakResetProgress: 1.0,
    previousSession: { state: "Sitting", durationSecs: 1800, wasEffective: false },
    todaySessions: mkSessions([{ state: "Sitting", mins: 30 }]),
    todayChanges: 0, todayStandingSecs: 0, todaySittingSecs: 1800, todayScore: -5,
    metrics: [
      mkMetric("standing_pct", "\u2195 Standing", "0%", "red"),
      mkMetric("position_rate", "\u21c4 Changes", "0/h", "red"),
      mkMetric("hourly_breaks", "\u2615 Breaks", "0/1h", "red"),
      mkMetric("longest_session", "\ud83d\udc41 Screen", "30m", "green"),
    ],
    error: null, idleSecs: 0, continuousComputerSecs: 0, onOpenSettings: noop,
  },
};

/** S07: Back from long away — fresh session, reset credits applied. */
export const S07_BACK_FROM_AWAY: Scenario = {
  id: "S07",
  name: "Just returned from break",
  description: "User returned after 15 min away. Session reset. Fresh start.",
  context: "After lunch break. Sitting timer starts from 0.",
  props: {
    connected: true, port: "COM3", state: "Sitting", deskHeightCm: 72,
    limitUsedSecs: 30, limitSecs: 2400, standLimitSecs: 900,
    limitRemaining: 2370, limitRatio: 0.0125, breakSecs: 0,
    breakResetThreshold: 600, breakResetProgress: 0,
    previousSession: { state: "Away", durationSecs: 900, wasEffective: true },
    todaySessions: mkSessions([
      { state: "Sitting", mins: 40 }, { state: "Standing", mins: 10 },
      { state: "Sitting", mins: 35 }, { state: "Away", mins: 15 },
    ]),
    todayChanges: 3, todayStandingSecs: 600, todaySittingSecs: 4530, todayScore: 10,
    metrics: [
      mkMetric("standing_pct", "\u2195 Standing", "12%", "yellow"),
      mkMetric("position_rate", "\u21c4 Changes", "1.5/h", "green"),
      mkMetric("hourly_breaks", "\u2615 Breaks", "1/2h", "yellow"),
      mkMetric("longest_session", "\ud83d\udc41 Screen", "40m", "green"),
    ],
    error: null, idleSecs: 0, continuousComputerSecs: 0, onOpenSettings: noop,
  },
};

/** S08: Good day — afternoon, healthy pattern. */
export const S08_GOOD_DAY: Scenario = {
  id: "S08",
  name: "Healthy afternoon",
  description: "4h into work. Regular position changes. All KPIs green.",
  context: "The ideal day we're designing for.",
  props: {
    connected: true, port: "COM3", state: "Sitting", deskHeightCm: 72,
    limitUsedSecs: 900, limitSecs: 2400, standLimitSecs: 900,
    limitRemaining: 1500, limitRatio: 0.375, breakSecs: 0,
    breakResetThreshold: 600, breakResetProgress: 0,
    previousSession: { state: "Standing", durationSecs: 900, wasEffective: true },
    todaySessions: mkSessions([
      { state: "Sitting", mins: 35 }, { state: "Standing", mins: 15 },
      { state: "Sitting", mins: 30 }, { state: "Away", mins: 10 },
      { state: "Sitting", mins: 25 }, { state: "Standing", mins: 15 },
      { state: "Sitting", mins: 35 }, { state: "Standing", mins: 15 },
    ]),
    todayChanges: 7, todayStandingSecs: 2700, todaySittingSecs: 7500, todayScore: 45,
    metrics: [
      mkMetric("standing_pct", "\u2195 Standing", "26%", "green"),
      mkMetric("position_rate", "\u21c4 Changes", "1.8/h", "green"),
      mkMetric("hourly_breaks", "\u2615 Breaks", "3/4h", "green"),
      mkMetric("longest_session", "\ud83d\udc41 Screen", "35m", "green"),
    ],
    error: null, idleSecs: 0, continuousComputerSecs: 0, onOpenSettings: noop,
  },
};

/** S09: Sensor disconnected. */
export const S09_DISCONNECTED: Scenario = {
  id: "S09",
  name: "Sensor disconnected",
  description: "Sensor lost. App can't track position.",
  context: "USB unplugged, cable issue, sensor failure.",
  props: {
    connected: false, port: null, state: "Away", deskHeightCm: 0,
    limitUsedSecs: 0, limitSecs: 2400, standLimitSecs: 900,
    limitRemaining: 2400, limitRatio: 0, breakSecs: 0,
    breakResetThreshold: 600, breakResetProgress: 0,
    previousSession: null, todaySessions: [], todayChanges: 0,
    todayStandingSecs: 0, todaySittingSecs: 0, todayScore: 0,
    metrics: [], error: "Sensor disconnected", idleSecs: 0, continuousComputerSecs: 0, onOpenSettings: noop,
  },
};

/**
 * VoiceCapture states (E021-T05).
 *
 * VoiceCapture takes no props — its token comes from `localStorage` and its
 * acknowledgement from the WS stream — so these scenarios describe the two
 * inputs the mockup gallery drives rather than a `WidgetProps` payload.
 */
export interface VoiceCaptureScenario {
  id: string;
  name: string;
  description: string;
  context: string;
  /** Token to seed into `localStorage`; null renders the token sheet. */
  token: string | null;
  /** Ack to push onto the bus, or null to leave the panel bare. */
  ack: VoiceAck | null;
}

/** VC1: first run on the phone — no token yet. */
export const voiceCaptureTokenSheet: VoiceCaptureScenario = {
  id: "VC1",
  name: "Voice — token sheet",
  description: "No desk_token stored. Only the token form is shown.",
  context: "First time the phone opens /display.",
  token: null,
  ack: null,
};

/** VC2: ready to dictate, nothing sent yet. */
export const voiceCaptureIdle: VoiceCaptureScenario = {
  id: "VC2",
  name: "Voice — ready",
  description: "Token stored. Textarea empty, Send disabled.",
  context: "Normal resting state on the phone.",
  token: "demo-token",
  ack: null,
};

/** VC3: a snooze intent came back with an AI reply. */
export const voiceCaptureAck: VoiceCaptureScenario = {
  id: "VC3",
  name: "Voice — acknowledged",
  description: "Backend parsed a snooze intent and answered.",
  context: 'User said "drzemka 5" and the ack arrived over the WS stream.',
  token: "demo-token",
  ack: { transcript: "drzemka 5", intent: "Snooze(5)", reply: "Ok, cisza przez 5 minut." },
};

/** All VoiceCapture scenarios, in gallery order. */
export const VOICE_CAPTURE_SCENARIOS: VoiceCaptureScenario[] = [
  voiceCaptureTokenSheet,
  voiceCaptureIdle,
  voiceCaptureAck,
];
