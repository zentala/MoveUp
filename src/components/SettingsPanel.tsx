/**
 * SettingsPanel.tsx — unified settings configuration panel.
 *
 * Covers: profiles, calibration, notifications, misc actions.
 * Layout: horizontal tabs (Time, Calibr., Notif., More) — no scrolling.
 * Design system: uses tokens from system.md (--beam, --panel-*, --rail-*, --ink-*)
 */
import { useState, useEffect, useCallback, useRef } from "react";
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
import HealthHelpSection from "./settings/HealthHelpSection";
import { AutostartToggle } from "./AutostartToggle";

export type { SettingsPanelProps, DeskSettings } from "./settings/SettingsTypes";

const SettingsPanel: FC<SettingsPanelProps> = ({ onClose }) => {
  const [settings, setSettings] = useState<DeskSettings>(DEFAULT_SETTINGS);
  const [loading, setLoading] = useState(true);
  const [loaded, setLoaded] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<SettingsTabIndex>(0);
  const loadRequest = useRef<Promise<void> | null>(null);

  const loadSettings = useCallback(async () => {
    if (loadRequest.current) return loadRequest.current;
    setLoading(true);
    setError(null);
    const request = (async () => {
      try {
        const loadedSettings = await invoke<DeskSettings>("get_settings");
        setSettings(loadedSettings);
        setLoaded(true);
      } catch (err) {
        setLoaded(false);
        setError(String(err));
      } finally {
        loadRequest.current = null;
        setLoading(false);
      }
    })();
    loadRequest.current = request;
    return request;
  }, []);

  useEffect(() => {
    void loadSettings();
    const unsubscribe = listen("desk:db-error", (event) => {
      setError(String(event.payload));
    });
    return () => { unsubscribe.then((u) => u()); };
  }, [loadSettings]);

  const sittingHeightInvalid = !Number.isFinite(settings.sitting_mm) || settings.sitting_mm < 400 || settings.sitting_mm > 900;
  const standingHeightInvalid = !Number.isFinite(settings.standing_mm) || settings.standing_mm < 900 || settings.standing_mm > 1400;
  const relationshipInvalid = !sittingHeightInvalid && !standingHeightInvalid && settings.sitting_mm >= settings.standing_mm;
  const calibrationError = sittingHeightInvalid && standingHeightInvalid
    ? "Sitting height must be between 400 and 900 mm; standing height must be between 900 and 1400 mm"
    : sittingHeightInvalid
    ? "Sitting height must be between 400 and 900 mm"
    : standingHeightInvalid
      ? "Standing height must be between 900 and 1400 mm"
      : relationshipInvalid
        ? "Standing height must be greater than sitting height"
        : null;

  async function handleSave() {
    if (!loaded || saving || calibrationError !== null) return;
    setSaving(true);
    setError(null);
    try {
      await invoke("save_settings", { config: settings });
      window.dispatchEvent(new CustomEvent("moveup:settings-saved", {
        detail: { show_activity_status: settings.show_activity_status },
      }));
      onClose();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  }

  if (loading || !loaded) {
    return (
      <div className="settings-panel">
        <div className="settings-toolbar">
          <div className="settings-toolbar__identity">
            <button className="settings-toolbar__back" onClick={onClose} disabled={saving} aria-label="Back">←</button>
            <span className="settings-toolbar__title">Settings</span>
          </div>
          <button className="settings-toolbar__save" disabled>Save</button>
        </div>
        <div className="settings-panel__body" aria-busy={loading}>
          {loading ? (
            <p className="settings-panel__loading" role="status">Loading settings…</p>
          ) : (
            <div className="settings-panel__load-error" role="alert">
              <p className="error-banner">Could not load settings: {error}</p>
              <button className="btn btn--secondary" type="button" onClick={() => void loadSettings()}>
                Retry
              </button>
            </div>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="settings-panel">
      <div className="settings-toolbar">
        <div className="settings-toolbar__identity">
          <button className="settings-toolbar__back" onClick={onClose} disabled={saving} aria-label="Back">←</button>
          <span className="settings-toolbar__title">Settings</span>
        </div>
        <button
          className="settings-toolbar__save"
          onClick={handleSave}
          disabled={saving || !loaded || calibrationError !== null}
        >
          {saving ? "…" : "Save"}
        </button>
      </div>
      <SettingsTabBar activeTab={activeTab} onTabChange={setActiveTab} />

      <div
        className="settings-panel__body"
        id="settings-panel"
        role="tabpanel"
        aria-labelledby={`settings-tab-${activeTab}`}
        tabIndex={0}
      >
        <p className="settings-panel__helper" role="note">
          Changes here are drafts until Save. Back discards these settings changes; profile and theme choices apply immediately and are not undone by Back.
        </p>
        {activeTab === 0 && (
          <>
            <ProfileSelector type="ergonomic" label="Ergonomic Profile" />
          </>
        )}

        {activeTab === 1 && (
          <CalibrationSection
            settings={settings}
            onChange={setSettings}
            validationError={calibrationError}
            sittingInvalid={sittingHeightInvalid || relationshipInvalid}
            standingInvalid={standingHeightInvalid || relationshipInvalid}
          />
        )}

        {activeTab === 2 && (
          <NotificationsSection />
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
                Show keyboard/mouse activity (Recent input / No recent input)
              </label>
              <p className="settings-panel__helper">No recent input can also mean reading, listening, thinking, or AI work; it does not prove you left the desk.</p>
            </div>
            <RemoteSection settings={settings} onChange={setSettings} />
            <HealthHelpSection />
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

      {calibrationError && activeTab !== 1 && (
        <div className="settings-panel__validation-error" role="alert">{calibrationError}</div>
      )}
      {error && <div className="error-banner" role="alert">{error}</div>}

    </div>
  );
};

export default SettingsPanel;
