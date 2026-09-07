/**
 * types.ts — shared TypeScript types for the Desk application.
 *
 * Two kinds of type live here, and the difference matters:
 *
 * - **Wire types** cross the Tauri IPC boundary. Rust owns their shape, so
 *   they are re-exported from `src/generated/`, which `ts-rs` writes from the
 *   real structs (ADR 017). Never hand-edit them — rename a field in Rust and
 *   `pnpm typecheck` is what tells you.
 *   Regenerate: `cargo test --manifest-path src-tauri/Cargo.toml --lib -- ts_export`.
 * - **Frontend types** (`WidgetProps`, `PreviousSession`, `CalibrateParams`)
 *   have no Rust counterpart and stay hand-written.
 */
import type { DeskState } from "./generated/DeskState";
import type { MetricSnapshot } from "./generated/MetricSnapshot";
import type { SessionRow } from "./generated/SessionRow";

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

// ─── Wire types — generated from Rust ───────────────────────────────────────

/** Possible ergonomic states detected by the desk sensor. */
export type { DeskState };
/** Break credit type applied on a Standing->Sitting transition. */
export type { BreakCredit } from "./generated/BreakCredit";
/** Payload for the `desk:state-changed` event. */
export type { StateChangedPayload } from "./generated/StateChangedPayload";
/**
 * Return type of `get_session_state()`. Reflects the live ergonomic session.
 *
 * `limit_used_secs` is the only counter a timer, progress bar, colour band or
 * notification may read; `secs_since_last_break` is Debug-tab only (ADR 008).
 */
export type { SessionStateDto } from "./generated/SessionStateDto";
/** Severity level for a KPI metric. */
export type { MetricLevel } from "./generated/MetricLevel";
/** Result of computing a single metric. */
export type { MetricResult } from "./generated/MetricResult";
/** A named metric snapshot for IPC transport. */
export type { MetricSnapshot };
/** Combined dashboard response from `get_dashboard_state`. */
export type { DashboardState } from "./generated/DashboardState";
/** Serial port info returned by the `list_ports` command. */
export type { PortInfo } from "./generated/PortInfo";
/**
 * A single tracked session within a day.
 *
 * `duration_secs` and `break_credit` are nullable because a row may predate
 * the column — null means unknown, never zero (ADR 008).
 */
export type { SessionRow as SessionEntry };
/** Return type of the `get_today_summary()` Tauri command. */
export type { TodaySummary as TodaySummaryDto } from "./generated/TodaySummary";

// ─── Remote protocol payloads (E022) ────────────────────────────────────────
// The runtime schemas that validate these live in `src/remote/protocol.ts`;
// only the shapes are generated. Import the schemas from there, not the bare
// types, whenever the value arrived over a socket.

/** Which end of a relay room a socket is. */
export type { Role as RemoteRole } from "./generated/Role";
/** Self-description a client sends in `hello`. */
export type { ClientInfo as RemoteClientInfo } from "./generated/ClientInfo";
/** First message on every remote socket. */
export type { Hello as RemoteHello } from "./generated/Hello";
/** The relay's answer to a valid `hello`. */
export type { Welcome as RemoteWelcome } from "./generated/Welcome";
/** Desk presence, pushed to viewers on connect and disconnect. */
export type { DeskStatus as RemoteDeskStatus } from "./generated/DeskStatus";
/** An allowlisted command a viewer sends to the desk. */
export type { Command as RemoteCommand } from "./generated/Command";
/** The desk's answer to one command. */
export type { CommandResult as RemoteCommandResult } from "./generated/CommandResult";
/** Error body of a failed remote message or REST call. */
export type { ErrorBody as RemoteErrorBody } from "./generated/ErrorBody";

// ─── Relay commands (E022-T06) — what Settings calls and renders ────────────

/** What the relay connection is doing, from `get_relay_status`. */
export type { RelayStatus } from "./generated/RelayStatus";
/**
 * One relay connection state.
 *
 * `unentitled`, `revoked` and `replaced` are terminal — the desk stopped on
 * purpose and the UI must say so instead of showing "reconnecting".
 */
export type { RelayState } from "./generated/RelayState";
/** The public half of this desk's registration. */
export type { DeskRecord } from "./generated/DeskRecord";
/** A pairing code plus the QR deep link, from `relay_start_pairing`. */
export type { PairingCode } from "./generated/PairingCode";
/** One paired phone, from `relay_list_viewers`. */
export type { Viewer as RelayViewer } from "./generated/Viewer";

// ─── Frontend-only types ────────────────────────────────────────────────────

/** Payload for `desk:device-connected` event. */
export interface DeviceConnectedPayload {
  port: string;
}

/** Payload for `desk:distance` event. */
export interface DistancePayload {
  mm: number;
  cm: number;
  timestamp: string;
}

/** Payload for `desk:sensor-error` event. */
export interface SensorErrorPayload {
  message: string;
  timestamp: string;
}

/** Parameters for the `calibrate()` Tauri command. All are optional. */
export interface CalibrateParams {
  sitting_mm?: number;
  standing_mm?: number;
  desk_thickness_mm?: number;
}

/** Legacy distance reading — kept for compatibility. */
export interface DistanceReading {
  mm: number;
  cm: number;
  timestamp: string;
}

// ─── Widget System Types ────────────────────────────────────────────────────

/** Info about the most recent completed session (before the current one). */
export interface PreviousSession {
  state: DeskState;
  durationSecs: number;
  /** True if the break was long enough for credit (partial or full). */
  wasEffective: boolean;
}

/** Data provided by core to every widget — pure presentation contract. */
export interface WidgetProps {
  connected: boolean;
  port: string | null;
  state: DeskState;
  deskHeightCm: number;
  /**
   * Seconds of the sitting limit consumed, credited for breaks.
   * Drives the big timer, the progress bar and the colour band — one value.
   */
  limitUsedSecs: number;
  limitSecs: number;
  /** Standing target in seconds (e.g. 900 = 15 min). */
  standLimitSecs: number;
  /** limitSecs - limitUsedSecs. Goes negative when over limit. */
  limitRemaining: number;
  /** limitUsedSecs / limitSecs. Can exceed 1.0. 0 when limitSecs=0. */
  limitRatio: number;
  breakSecs: number;
  breakResetThreshold: number;
  breakResetProgress: number;
  previousSession: PreviousSession | null;
  todaySessions: SessionRow[];
  todayChanges: number;
  todayStandingSecs: number;
  todaySittingSecs: number;
  todayScore: number;
  /** Current system idle time in seconds. */
  idleSecs: number;
  /** Continuous seconds at the computer. Resets after 5+ min Away. */
  continuousComputerSecs: number;
  /** KPI metric snapshots from MetricEngine. */
  metrics: MetricSnapshot[];
  error: string | null;
  onOpenSettings: () => void;
}

/** A widget is a pure presentation component receiving WidgetProps. */
export type DeskWidget = React.FC<WidgetProps>;
