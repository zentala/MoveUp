/**
 * Canonical names of the Tauri events the backend emits.
 *
 * Mirrors `src-tauri/src/desk_events.rs`; `events.test.ts` reads that file and
 * fails if the two drift apart. Consumers are rewired to import from here in
 * E018 — this module is deliberately standalone for now.
 */
export const DESK_DISTANCE = 'desk:distance';
export const DESK_STATE_CHANGED = 'desk:state-changed';
export const DESK_DEVICE_CONNECTED = 'desk:device-connected';
export const DESK_DEVICE_LOST = 'desk:device-lost';
export const DESK_DEVICE_MISSING = 'desk:device-missing';
export const DESK_SENSOR_ERROR = 'desk:sensor-error';
export const DESK_DAILY_RESET = 'desk:daily-reset';
export const DESK_SHOW_WIDGET = 'desk:show-widget';
export const DESK_SHOW_SETTINGS = 'desk:show-settings';
export const DESK_POPUP_THEME = 'desk:popup-theme';

/** Every event name above, keyed by the same identifier the Rust module uses. */
export const DESK_EVENTS = {
  DESK_DISTANCE,
  DESK_STATE_CHANGED,
  DESK_DEVICE_CONNECTED,
  DESK_DEVICE_LOST,
  DESK_DEVICE_MISSING,
  DESK_SENSOR_ERROR,
  DESK_DAILY_RESET,
  DESK_SHOW_WIDGET,
  DESK_SHOW_SETTINGS,
  DESK_POPUP_THEME,
} as const;

/** Union of all desk event names. */
export type DeskEventName = (typeof DESK_EVENTS)[keyof typeof DESK_EVENTS];
