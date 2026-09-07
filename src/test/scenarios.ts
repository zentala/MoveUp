/**
 * scenarios.ts — Predefined app states for mockups and tests.
 *
 * Each scenario represents a realistic moment in the user's day.
 * Used by: /mockup dev route, unit tests, integration tests.
 *
 * Scenario data lives in scenarios-sitting.ts and scenarios-other.ts.
 * This file re-exports everything + defines collection arrays.
 */
import type { WidgetProps } from "@/types";

/** A named scenario with description and full widget state. */
export interface Scenario {
  id: string;
  name: string;
  description: string;
  /** When this typically happens in the user's day. */
  context: string;
  props: WidgetProps;
}

// Re-export all individual scenarios
export {
  S01_FRESH_START,
  S02_SITTING_GREEN,
  S03_SITTING_YELLOW,
  S04_SITTING_OVERTIME,
} from "./scenarios-sitting";

export {
  S05_STANDING_MID,
  S06_AWAY,
  S07_BACK_FROM_AWAY,
  S08_GOOD_DAY,
  S09_DISCONNECTED,
  voiceCaptureTokenSheet,
  voiceCaptureIdle,
  voiceCaptureAck,
  VOICE_CAPTURE_SCENARIOS,
} from "./scenarios-other";
export type { VoiceCaptureScenario } from "./scenarios-other";

// Import for array construction
import {
  S01_FRESH_START, S02_SITTING_GREEN,
  S03_SITTING_YELLOW, S04_SITTING_OVERTIME,
} from "./scenarios-sitting";

import {
  S05_STANDING_MID, S06_AWAY, S07_BACK_FROM_AWAY,
  S08_GOOD_DAY, S09_DISCONNECTED,
} from "./scenarios-other";

/** Primary scenarios — must look right before any release. */
export const PRIMARY_SCENARIOS: Scenario[] = [
  S01_FRESH_START,
  S02_SITTING_GREEN,
  S03_SITTING_YELLOW,
  S04_SITTING_OVERTIME,
  S05_STANDING_MID,
  S06_AWAY,
  S07_BACK_FROM_AWAY,
];

/** Secondary scenarios — edge cases. */
export const SECONDARY_SCENARIOS: Scenario[] = [
  S08_GOOD_DAY,
  S09_DISCONNECTED,
];

/** All scenarios. */
export const ALL_SCENARIOS: Scenario[] = [
  ...PRIMARY_SCENARIOS,
  ...SECONDARY_SCENARIOS,
];

// ─── Remote pairing and controls (E022-T09) ─────────────────────────────────

/**
 * One state of the phone's relay surfaces.
 *
 * These do not carry `WidgetProps`: the pairing screen renders before any
 * desk data exists, and the controls render beside the widget rather than
 * inside it. Same purpose as `Scenario` though — one named moment the mockup
 * gallery and the unit tests can both point at.
 */
export interface RemoteScenario {
  id: string;
  name: string;
  description: string;
  context: string;
  /** Hash the phone opens with. */
  hash: string;
  /** Relay error to simulate, or null for a clean run. */
  error: { code: string; message: string; retry_after_secs?: number } | null;
  /** Whether a command is waiting for its `command_result`. */
  commandPending: boolean;
}

/** No parameters — the manual form, which is what a typed URL gives you. */
export const remotePairEmpty: RemoteScenario = {
  id: "pair-empty",
  name: "Pair — empty form",
  description: "Desk ID and code blank, nothing sent yet.",
  context: "Phone opened /app by hand, without scanning the QR.",
  hash: "#/pair",
  error: null,
  commandPending: false,
};

/** The QR path: both fields prefilled, request already on its way. */
export const remotePairDeeplink: RemoteScenario = {
  id: "pair-deeplink",
  name: "Pair — from the QR",
  description: "Deep link prefilled both fields and submitted on its own.",
  context: "User scanned the code shown in Settings → Remote.",
  hash: "#/pair?d=desk-42&c=ABCD2345",
  error: null,
  commandPending: false,
};

/** Ten wrong codes: the desk stops answering for fifteen minutes. */
export const remotePairErrorLocked: RemoteScenario = {
  id: "pair-error-locked",
  name: "Pair — locked out",
  description: "Too many wrong codes; the screen says when to come back.",
  context: "Someone guessing codes at the pairing endpoint.",
  hash: "#/pair?d=desk-42&c=ABCD2345",
  error: { code: "pairing_locked", message: "locked", retry_after_secs: 900 },
  commandPending: false,
};

/** A command sent, no result yet — every button disabled, one says "…". */
export const remoteControlsPending: RemoteScenario = {
  id: "controls-pending",
  name: "Controls — waiting for the desk",
  description: "Dismiss pressed; the panel waits for command_result.",
  context: "Phone on the relay, desk answering within a second or two.",
  hash: "#/",
  error: null,
  commandPending: true,
};

/** Remote scenarios, in gallery order. */
export const REMOTE_SCENARIOS: RemoteScenario[] = [
  remotePairEmpty,
  remotePairDeeplink,
  remotePairErrorLocked,
  remoteControlsPending,
];
