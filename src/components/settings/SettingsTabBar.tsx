/**
 * SettingsTabBar.tsx — horizontal tab bar for the settings panel.
 *
 * Renders the Settings sections as a manually activated, keyboard-operable tablist.
 * Uses design tokens: --rail-default border, --beam active accent.
 */
import { useState, type FC, type KeyboardEvent } from "react";

/** Tab definitions for the settings panel. */
export const SETTINGS_TABS = ["Profiles", "Calibr.", "Notif.", "More", "Debug"] as const;

export type SettingsTabIndex = 0 | 1 | 2 | 3 | 4;

interface SettingsTabBarProps {
  activeTab: SettingsTabIndex;
  onTabChange: (index: SettingsTabIndex) => void;
}

/** Horizontal tab bar with manual activation and a roving focus target. */
const SettingsTabBar: FC<SettingsTabBarProps> = ({ activeTab, onTabChange }) => {
  const [focusedTab, setFocusedTab] = useState<SettingsTabIndex>(activeTab);
  const onKeyDown = (event: KeyboardEvent<HTMLButtonElement>) => {
    const tabs = Array.from(
      event.currentTarget.parentElement?.querySelectorAll<HTMLButtonElement>('[role="tab"]') ?? [],
    );
    const current = tabs.indexOf(event.currentTarget);
    let target = -1;
    if (event.key === "ArrowRight") target = Math.min(tabs.length - 1, current + 1);
    if (event.key === "ArrowLeft") target = Math.max(0, current - 1);
    if (event.key === "Home") target = 0;
    if (event.key === "End") target = tabs.length - 1;
    if (target >= 0) {
      event.preventDefault();
      setFocusedTab(target as SettingsTabIndex);
      tabs[target]?.focus();
    }
  };

  return (
    <div className="settings-tabs" role="tablist" aria-label="Settings sections" aria-orientation="horizontal">
      {SETTINGS_TABS.map((label, index) => (
        <button
          key={label}
          id={`settings-tab-${index}`}
          role="tab"
          aria-controls="settings-panel"
          aria-selected={index === activeTab}
          tabIndex={index === focusedTab ? 0 : -1}
          className={`settings-tab${index === activeTab ? " settings-tab--active" : ""}`}
          onFocus={() => setFocusedTab(index as SettingsTabIndex)}
          onClick={() => {
            setFocusedTab(index as SettingsTabIndex);
            onTabChange(index as SettingsTabIndex);
          }}
          onKeyDown={onKeyDown}
        >
          {label}
        </button>
      ))}
    </div>
  );
};

export default SettingsTabBar;
