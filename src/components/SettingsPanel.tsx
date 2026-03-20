/**
 * SettingsPanel.tsx — unified settings configuration panel.
 *
 * Replaces the CalibrationWizard with a single comprehensive panel covering:
 * - Time limits (sitting, standing)
 * - Calibration (height thresholds)
 * - Notifications (three toggle options)
 *
 * Design system: uses tokens from system.md (--beam, --panel-*, --rail-*, --ink-*)
 */
import { useState, useEffect } from "react";
import type { FC } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

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

const DEFAULT_SETTINGS: DeskSettings = {
  sit_limit_mins: 45,
  stand_limit_mins: 15,
  sitting_mm: 720,
  standing_mm: 1050,
  notify_inactivity: true,
  notify_daily_posture_balance: true,
  notify_praise_halfway: false,
};

/**
 * Unified settings panel for desk ergonomics configuration.
 * Loads current settings on mount, allows editing, validates, and saves.
 */
const SettingsPanel: FC<SettingsPanelProps> = ({ onClose }) => {
  const [settings, setSettings] = useState<DeskSettings>(DEFAULT_SETTINGS);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [validationError, setValidationError] = useState<string | null>(null);

  // Load settings on mount
  useEffect(() => {
    async function loadSettings() {
      try {
        const loaded = await invoke<DeskSettings>("get_settings");
        setSettings(loaded);
        setLoading(false);
      } catch (err) {
        // If no settings exist yet, use defaults and allow editing
        setLoading(false);
      }
    }
    loadSettings();

    // Listen for backend errors
    const unsubscribe = listen("desk:db-error", (event) => {
      setError(String(event.payload));
    });

    return () => {
      unsubscribe.then((u) => u());
    };
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
      // Mark setup as complete so first-run detection does not trigger again
      localStorage.setItem("desk_setup_done", "1");
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
      {/* Header */}
      <div className="settings-panel__header">
        <h2 className="settings-panel__title">Settings</h2>
      </div>

      {/* Section 1: Time Limits */}
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
              min="10"
              max="90"
              step="5"
              value={settings.sit_limit_mins}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  sit_limit_mins: parseInt(e.target.value, 10),
                })
              }
              className="settings-panel__slider"
            />
            <span className="settings-panel__value">
              {settings.sit_limit_mins}
            </span>
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
              min="5"
              max="60"
              step="5"
              value={settings.stand_limit_mins}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  stand_limit_mins: parseInt(e.target.value, 10),
                })
              }
              className="settings-panel__slider"
            />
            <span className="settings-panel__value">
              {settings.stand_limit_mins}
            </span>
          </div>
        </div>
      </div>

      {/* Section 2: Calibration */}
      <div className="settings-panel__section">
        <h3 className="settings-panel__section-title">Calibration</h3>
        <p className="settings-panel__helper">
          Set desk to sitting position and enter sensor reading in mm.
        </p>

        <div className="settings-panel__field">
          <label htmlFor="sitting-height" className="settings-panel__label">
            Sitting height (mm)
          </label>
          <input
            id="sitting-height"
            type="number"
            min="400"
            max="900"
            value={settings.sitting_mm}
            onChange={(e) =>
              setSettings({
                ...settings,
                sitting_mm: parseInt(e.target.value, 10),
              })
            }
            className="settings-panel__input"
            required
          />
        </div>

        <div className="settings-panel__field">
          <label htmlFor="standing-height" className="settings-panel__label">
            Standing height (mm)
          </label>
          <input
            id="standing-height"
            type="number"
            min="900"
            max="1400"
            value={settings.standing_mm}
            onChange={(e) =>
              setSettings({
                ...settings,
                standing_mm: parseInt(e.target.value, 10),
              })
            }
            className="settings-panel__input"
            required
          />
        </div>

        {validationError && (
          <div className="settings-panel__validation-error">
            {validationError}
          </div>
        )}
      </div>

      {/* Section 3: Notifications */}
      <div className="settings-panel__section">
        <h3 className="settings-panel__section-title">Notifications</h3>

        <div className="settings-panel__toggle-group">
          <label className="settings-panel__toggle-label">
            <input
              type="checkbox"
              checked={settings.notify_inactivity}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  notify_inactivity: e.target.checked,
                })
              }
              className="settings-panel__checkbox"
            />
            <span>Alert if no position change for 90 min</span>
          </label>
        </div>

        <div className="settings-panel__toggle-group">
          <label className="settings-panel__toggle-label">
            <input
              type="checkbox"
              checked={settings.notify_daily_posture_balance}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  notify_daily_posture_balance: e.target.checked,
                })
              }
              className="settings-panel__checkbox"
            />
            <span>Alert if sitting dominates today</span>
          </label>
        </div>

        <div className="settings-panel__toggle-group">
          <label className="settings-panel__toggle-label">
            <input
              type="checkbox"
              checked={settings.notify_praise_halfway}
              onChange={(e) =>
                setSettings({
                  ...settings,
                  notify_praise_halfway: e.target.checked,
                })
              }
              className="settings-panel__checkbox"
            />
            <span>Praise when halfway through standing goal</span>
          </label>
        </div>
      </div>

      {/* Error banner */}
      {error && <div className="error-banner">{error}</div>}

      {/* Footer with buttons */}
      <div className="settings-panel__footer">
        <button
          className="btn btn--danger"
          onClick={onClose}
          disabled={saving}
        >
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
