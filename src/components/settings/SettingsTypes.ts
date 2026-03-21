/**
 * SettingsTypes.ts — shared types and defaults for settings panel.
 */

export interface SettingsPanelProps {
  /** Called when user clicks Back/Cancel to close settings without saving. */
  onClose: () => void;
}

export interface DeskSettings {
  sit_limit_mins: number;
  stand_limit_mins: number;
  sitting_mm: number;
  standing_mm: number;
  notify_inactivity: boolean;
  notify_daily_posture_balance: boolean;
  notify_praise_halfway: boolean;
}

export const DEFAULT_SETTINGS: DeskSettings = {
  sit_limit_mins: 45,
  stand_limit_mins: 15,
  sitting_mm: 720,
  standing_mm: 1050,
  notify_inactivity: true,
  notify_daily_posture_balance: true,
  notify_praise_halfway: false,
};
