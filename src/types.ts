/**
 * types.ts — shared TypeScript types for the Desk application.
 *
 * Covers all Tauri event payloads, command return types, and domain enums.
 */

/** Possible ergonomic states detected by the desk sensor. */
export type DeskState = "Sitting" | "Standing" | "Walking" | "Away";

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

/** Break credit type applied on a Standing->Sitting transition. */
export type BreakCredit = "none" | "partial" | "full";

/** Payload for `desk:state-changed` event. */
export interface StateChangedPayload {
  state: DeskState;
  sitting_seconds: number;
  standing_seconds: number;
  break_seconds: number;
  desk_height_cm: number;
  position_changes: number;
  /** Duration of the last standing/break session in seconds. */
  last_break_secs: number;
  /** Duration of the last sitting session in seconds. */
  last_sitting_secs: number;
  /** Break credit applied on this transition. */
  break_credit: BreakCredit;
  /** Current sitting session seconds (resets after break credit). */
  current_session_secs: number;
}

/** Payload for `desk:sensor-error` event. */
export interface SensorErrorPayload {
  message: string;
  timestamp: string;
}

/**
 * Return type of `get_session_state()` Tauri command.
 * Reflects the current live ergonomic session.
 */
export interface SessionStateDto {
  state: DeskState;
  sitting_seconds: number;
  standing_seconds: number;
  break_seconds: number;
  session_limit_secs: number;
  desk_height_cm: number;
  position_changes: number;
  /** Seconds of sitting limit consumed (accounts for break credits). */
  limit_used_secs: number;
  /** Daily posture score (in-memory, resets at midnight). */
  daily_score: number;
  /** Current continuous standing session seconds (resets on sit). */
  standing_session_secs: number;
  /** Current sitting session seconds (resets after break credit). */
  current_session_secs: number;
}

/** A single tracked session within a day. */
export interface SessionEntry {
  start: string;
  end: string | null;
  state: DeskState;
  duration_secs: number;
}

/** Return type of `get_today_summary()` Tauri command. */
export interface TodaySummaryDto {
  sitting_secs: number;
  standing_secs: number;
  sessions: SessionEntry[];
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

/** Serial port info returned by list_ports command. */
export interface PortInfo {
  name: string;
  description: string | null;
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
  currentSessionSecs: number;
  limitSecs: number;
  /** limitSecs - limitUsedSecs. Goes negative when over limit. */
  limitRemaining: number;
  /** limitUsedSecs / limitSecs. Can exceed 1.0. 0 when limitSecs=0. */
  limitRatio: number;
  breakSecs: number;
  breakResetThreshold: number;
  breakResetProgress: number;
  previousSession: PreviousSession | null;
  todaySessions: SessionEntry[];
  todayChanges: number;
  todayStandingSecs: number;
  todaySittingSecs: number;
  todayScore: number;
  error: string | null;
  onOpenSettings: () => void;
  /** Elapsed seconds for the current timer context (sitting session or break). */
  elapsed: number;
  /** Total seconds for the current timer context (limit or break threshold). */
  total: number;
  /** Color scheme based on current state. */
  colorScheme: "sitting" | "standing" | "gray";
}

/** A widget is a pure presentation component receiving WidgetProps. */
export type DeskWidget = React.FC<WidgetProps>;

/** Entry in the widget registry. */
export interface WidgetRegistration {
  id: string;
  name: string;
  component: DeskWidget;
}
