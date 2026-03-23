/**
 * SettingsTabBar.tsx — horizontal tab bar for the settings panel.
 *
 * Renders 4 tabs: Time, Calibr., Notif., More.
 * Uses design tokens: --rail-default border, --beam active accent.
 */
import type { FC } from "react";

/** Tab definitions for the settings panel. */
export const SETTINGS_TABS = ["Time", "Calibr.", "Notif.", "More", "Debug"] as const;

export type SettingsTabIndex = 0 | 1 | 2 | 3 | 4;

interface SettingsTabBarProps {
  activeTab: SettingsTabIndex;
  onTabChange: (index: SettingsTabIndex) => void;
}

/** Horizontal tab bar with active indicator using --beam accent. */
const SettingsTabBar: FC<SettingsTabBarProps> = ({ activeTab, onTabChange }) => (
  <div className="settings-tabs" role="tablist">
    {SETTINGS_TABS.map((label, index) => (
      <button
        key={label}
        role="tab"
        aria-selected={index === activeTab}
        className={`settings-tab${index === activeTab ? " settings-tab--active" : ""}`}
        onClick={() => onTabChange(index as SettingsTabIndex)}
      >
        {label}
      </button>
    ))}
  </div>
);

export default SettingsTabBar;
