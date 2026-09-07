/**
 * SettingsPanel.tsx — unified settings configuration panel.
 *
 * Covers: time limits, calibration, notifications, misc actions.
 * Layout: horizontal tabs (Time, Calibr., Notif., More) — no scrolling.
 * Design system: uses tokens from system.md (--beam, --panel-*, --rail-*, --ink-*)
 */
import { useState, useEffect } from "react";
import type { FC } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { SettingsPanelProps, DeskSettings } from "./settings/SettingsTypes";
import { DEFAULT_SETTINGS } from "./settings/SettingsTypes";
import SettingsTabBar from "./settings/SettingsTabBar";
import type { SettingsTabIndex } from "./settings/SettingsTabBar";
import CalibrationSection from "./settings/CalibrationSection";
import NotificationsSection from "./settings/NotificationsSection";
import ProfileSelector from "./settings/ProfileSelector";
import TimelineSkinSection from "./settings/TimelineSkinSection";
import DebugSection from "./settings/DebugSection";
import TelemetrySection from "./settings/TelemetrySection";
import WebhookSection from "./settings/WebhookSection";
import RemoteSection from "./settings/RemoteSection";
import { AutostartToggle } from "./AutostartToggle";

export type { SettingsPanelProps, DeskSettings } from "./settings/SettingsTypes";

const SettingsPanel: FC<SettingsPanelProps> = ({ onClose }) => {
  const [settings, setSettings] = useState<DeskSettings>(DEFAULT_SETTINGS);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [validationError, setValidationError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<SettingsTabIndex>(0);

  useEffect(() => {
    async function loadSettings() {
      try {
        const loaded = await invoke<DeskSettings>("get_settings");
        setSettings(loaded);
      } catch {
        // If no settings exist yet, use defaults
      } finally {
        setLoading(false);
      }
    }
    loadSettings();

    const unsubscribe = listen("desk:db-error", (event) => {
      setError(String(event.payload));
    });
    return () => { unsubscribe.then((u) => u()); };
  }, []);

  function validateSettings(): boolean {
    setValidationError(null);
    if (settings.sitting_mm >= settings.standing_mm) {
      setValidationError("Standing height must be greater than sitting height");
      return false;
    }
    return true;
  }

  async function handleSave() {
    if (!validateSettings()) return;
    setSaving(true);
    setError(null);
    try {
      await invoke("save_settings", { settings });
      onClose();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  }

  if (loading) {
    return (
      <div className="settings-panel">
        <div className="settings-toolbar">
          <button className="settings-toolbar__back" onClick={onClose} aria-label="Back">←</button>
          <span className="settings-toolbar__title">Settings</span>
        </div>
        <p className="settings-panel__loading">Loading…</p>
      </div>
    );
  }

  return (
    <div className="settings-panel">
      <div className="settings-toolbar">
        <button className="settings-toolbar__back" onClick={onClose} disabled={saving} aria-label="Back">←</button>
        <span className="settings-toolbar__title">Settings</span>
        <SettingsTabBar activeTab={activeTab} onTabChange={setActiveTab} />
        <button
          className="settings-toolbar__save"
          onClick={handleSave}
          disabled={saving || validationError !== null}
        >
          {saving ? "…" : "Save"}
        </button>
      </div>

      <div className="settings-panel__body">
        {activeTab === 0 && (
          <>
            <ProfileSelector type="ergonomic" label="Ergonomic Profile" />
            <ProfileSelector type="communication" label="Communication Profile" />
          </>
        )}

        {activeTab === 1 && (
          <CalibrationSection
            settings={settings}
            onChange={setSettings}
            validationError={validationError}
          />
        )}

        {activeTab === 2 && (
          <NotificationsSection settings={settings} onChange={setSettings} />
        )}

        {activeTab === 3 && (
          <>
            <TimelineSkinSection />
            <div className="settings-panel__section">
              <h3 className="settings-panel__section-title">Startup</h3>
              <AutostartToggle appName="MoveUp" />
            </div>
            <div className="settings-panel__section">
              <h3 className="settings-panel__section-title">Display</h3>
              <label className="settings-panel__toggle">
                <input
                  type="checkbox"
                  checked={settings.show_activity_status}
                  onChange={(e) => setSettings({ ...settings, show_activity_status: e.target.checked })}
                />
                Show activity status (Active/Idle)
              </label>
            </div>
            <RemoteSection settings={settings} onChange={setSettings} />
            <WebhookSection settings={settings} onChange={setSettings} />
            <TelemetrySection settings={settings} onChange={setSettings} />
            <div className="settings-panel__section">
              <button className="btn btn--secondary" onClick={() => invoke("show_welcome")}>
                Show intro again
              </button>
            </div>
          </>
        )}

        {activeTab === 4 && <DebugSection />}
      </div>

      {validationError && (
        <div className="settings-panel__validation-error">{validationError}</div>
      )}

      {error && <div className="error-banner">{error}</div>}

    </div>
  );
};

export default SettingsPanel;
