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
import type { PairingCode } from "@/generated/PairingCode";
import type { RelayStatus } from "@/generated/RelayStatus";
import type { Viewer } from "@/generated/Viewer";

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

// ─── Settings → Remote, on the desktop (E022-T10) ───────────────────────────

/**
 * One state of Settings → Remote.
 *
 * The desktop section is driven entirely by three IPC reads, so a scenario is
 * exactly those three answers. Feeding them to a mocked `invoke` renders the
 * section as the user would see it — no widget data involved.
 */
export interface RelaySettingsScenario {
  id: string;
  name: string;
  description: string;
  context: string;
  /** What `get_relay_status` returns. */
  status: RelayStatus;
  /** What `relay_list_viewers` returns. Empty is a state, not a blank panel. */
  viewers: Viewer[];
  /** A code from `relay_start_pairing`, or null when none was asked for. */
  pairing: PairingCode | null;
}

const NOW = Date.parse("2026-09-07T09:00:00Z");

/** Nothing set up: the section offers the license form and nothing else. */
export const relayDisabled: RelaySettingsScenario = {
  id: "relay-disabled",
  name: "Remote — off",
  description: "Never registered. One field, one button, no device list.",
  context: "Fresh install; the user just found the Remote section.",
  status: { enabled: false, state: "disabled", since: NOW, viewers_online: 0, last_error: null },
  viewers: [],
  pairing: null,
};

/** The steady state: connected, two phones paired, one of them watching. */
export const relayOnlineTwoViewers: RelaySettingsScenario = {
  id: "relay-online-2-viewers",
  name: "Remote — online, 2 phones",
  description: "Connected desk, two paired devices, one currently online.",
  context: "Everyday state once the phone and the tablet are both paired.",
  status: { enabled: true, state: "online", since: NOW, viewers_online: 1, last_error: null },
  viewers: [
    {
      viewer_id: "v-phone",
      device_name: "Pixel 8",
      paired_at: "2026-09-01T18:20:00Z",
      last_seen: "2026-09-07T08:59:00Z",
      online: true,
    },
    {
      viewer_id: "v-tablet",
      device_name: "Kitchen tablet",
      paired_at: "2026-09-03T07:05:00Z",
      last_seen: "2026-09-06T21:10:00Z",
      online: false,
    },
  ],
  pairing: null,
};

/** The licence stopped covering this desk — pairing must not look available. */
export const relayUnentitled: RelaySettingsScenario = {
  id: "relay-unentitled",
  name: "Remote — licence expired",
  description: "Relay closed the socket with 4402; the section says why.",
  context: "A Founder licence lapsed while the desk stayed registered.",
  status: {
    enabled: true,
    state: "unentitled",
    since: NOW,
    viewers_online: 0,
    last_error: "licence expired",
  },
  viewers: [],
  pairing: null,
};

/** A code was just issued: card on screen, QR beside it, minutes ticking. */
export const relayPairingCodeShown: RelaySettingsScenario = {
  id: "relay-pairing-code-shown",
  name: "Remote — pairing code shown",
  description: "Code, desk id and QR visible, with the expiry spelled out.",
  context: "User pressed “Pair a phone” and is holding the phone up to it.",
  status: { enabled: true, state: "online", since: NOW, viewers_online: 0, last_error: null },
  viewers: [],
  pairing: {
    code: "ABCD2345",
    expires_at: "2026-09-07T09:05:00Z",
    qr_payload: "https://relay.desk.zentala.io/app/#/pair?d=desk-42&c=ABCD2345",
    desk_id: "desk-42",
  },
};

/** Relay settings scenarios, in gallery order. */
export const RELAY_SETTINGS_SCENARIOS: RelaySettingsScenario[] = [
  relayDisabled,
  relayOnlineTwoViewers,
  relayUnentitled,
  relayPairingCodeShown,
];
