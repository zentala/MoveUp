/**
 * SettingsTypes.ts — shared types and defaults for the settings panel.
 */
import type { AppConfig } from "@/generated/AppConfig";

export type { AppConfig };

export interface SettingsPanelProps {
  /** Called when user clicks Back/Cancel to close settings without saving. */
  onClose: () => void;
}

/**
 * Toggles the settings UI still renders but the backend no longer sends.
 *
 * They moved to the ergonomic and communication profiles when those were
 * introduced; `get_settings` returns `AppConfig`, which has none of them, so
 * at runtime every one of these reads `undefined` today. They are optional
 * and quarantined here so `AppConfig` stays the sole description of what the
 * backend actually sends — deleting the dead UI that reads them is a
 * follow-up, not part of the codegen change (ADR 017).
 */
interface LegacySettingsFields {
  sit_limit_mins: number;
  stand_limit_mins: number;
  notify_inactivity: boolean;
  notify_daily_posture_balance: boolean;
  notify_praise_halfway: boolean;
}

/**
 * What the settings panel holds in state: the real `get_settings` payload
 * (generated from Rust's `AppConfig`) plus the legacy leftovers above.
 */
export type DeskSettings = AppConfig & Partial<LegacySettingsFields>;

export const DEFAULT_SETTINGS: DeskSettings = {
  sitting_mm: 720,
  standing_mm: 1050,
  desk_thickness_mm: 30,
  active_widget: 'one-bar',
  timeline_skin: 'semantic',
  show_welcome_on_startup: true,
  show_activity_status: true,
  telemetry_enabled: false,
  telemetry_device_id: '',
  notify_webhook_enabled: false,
  notify_webhook_url: null,
  voice_ai_model: null,
};
