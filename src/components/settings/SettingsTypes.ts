/**
 * SettingsTypes.ts — shared types and defaults for the settings panel.
 */
import type { AppConfig } from "@/generated/AppConfig";

export type { AppConfig };

export interface SettingsPanelProps {
  /** Called when user clicks Back/Cancel to close settings without saving. */
  onClose: () => void;
}

/** The settings panel holds exactly the configuration returned by the backend. */
export type DeskSettings = AppConfig;

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
  // E022-T06 — these three mirror Rust's `AppConfig` defaults. The relay is
  // opt-in; the LAN display stays on, because turning the relay on must not
  // take the existing transport away.
  relay_enabled: false,
  relay_url: 'https://relay.desk.zentala.io',
  remote_lan_enabled: true,
};
