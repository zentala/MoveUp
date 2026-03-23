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
import WidgetPickerSection from "./settings/WidgetPickerSection";

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
        <div className="settings-panel__header">
          <h2 className="settings-panel__title">Settings</h2>
        </div>
        <p className="settings-panel__loading">Loading…</p>
      </div>
    );
  }

  return (
    <div className="settings-panel">
      <div className="settings-panel__header">
        <h2 className="settings-panel__title">Settings</h2>
      </div>

      <SettingsTabBar activeTab={activeTab} onTabChange={setActiveTab} />

      <div className="settings-panel__body">
        {activeTab === 0 && (
          <div className="settings-panel__section">
            <h3 className="settings-panel__section-title">Time Limits</h3>
            <div className="settings-panel__field">
              <label htmlFor="sitting-limit" className="settings-panel__label">
                Remind me to stand after (minutes)
              </label>
              <div className="settings-panel__slider-row">
                <input
                  id="sitting-limit"
                  type="range"
                  min="10" max="90" step="5"
                  value={settings.sit_limit_mins}
                  onChange={(e) =>
                    setSettings({ ...settings, sit_limit_mins: parseInt(e.target.value, 10) })
                  }
                  className="settings-panel__slider"
                />
                <span className="settings-panel__value">{settings.sit_limit_mins}</span>
              </div>
            </div>
            <div className="settings-panel__field">
              <label htmlFor="standing-limit" className="settings-panel__label">
                Remind me to sit after (minutes)
              </label>
              <div className="settings-panel__slider-row">
                <input
                  id="standing-limit"
                  type="range"
                  min="5" max="60" step="5"
                  value={settings.stand_limit_mins}
                  onChange={(e) =>
                    setSettings({ ...settings, stand_limit_mins: parseInt(e.target.value, 10) })
                  }
                  className="settings-panel__slider"
                />
                <span className="settings-panel__value">{settings.stand_limit_mins}</span>
              </div>
            </div>
          </div>
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
            <WidgetPickerSection />
            <div className="settings-panel__section">
              <button className="btn btn--secondary" onClick={() => invoke("show_welcome")}>
                Show intro again
              </button>
            </div>
          </>
        )}
      </div>

      {validationError && (
        <div className="settings-panel__validation-error">{validationError}</div>
      )}

      {error && <div className="error-banner">{error}</div>}

      <div className="settings-panel__footer">
        <button className="btn btn--link" onClick={onClose} disabled={saving}>
          Back
        </button>
        <button
          className="btn"
          onClick={handleSave}
          disabled={saving || validationError !== null}
        >
          {saving ? "Saving…" : "Save"}
        </button>
      </div>
    </div>
  );
};

export default SettingsPanel;
