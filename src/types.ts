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

/** Payload for `desk:state-changed` event. */
export interface StateChangedPayload {
  state: DeskState;
  sitting_seconds: number;
  break_seconds: number;
  desk_height_cm: number;
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
  break_seconds: number;
  session_limit_secs: number;
  desk_height_cm: number;
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
