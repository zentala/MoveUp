/**
 * TelemetrySection.tsx — Telemetry opt-in toggle and data transparency.
 *
 * Shows what data is collected and lets the user toggle telemetry on/off.
 * Default: OFF (privacy-first, opt-in only).
 */
import { useState } from "react";
import type { FC } from "react";
import type { DeskSettings } from "./SettingsTypes";

interface TelemetrySectionProps {
  settings: DeskSettings;
  onChange: (settings: DeskSettings) => void;
}

/** Telemetry opt-in toggle with expandable "What data?" section. */
const TelemetrySection: FC<TelemetrySectionProps> = ({ settings, onChange }) => {
  const [expanded, setExpanded] = useState(false);

  return (
    <div className="settings-panel__section">
      <h3 className="settings-panel__section-title">Telemetry</h3>

      <div className="settings-panel__field">
        <label className="settings-panel__toggle-row">
          <input
            type="checkbox"
            checked={settings.telemetry_enabled}
            onChange={(e) =>
              onChange({ ...settings, telemetry_enabled: e.target.checked })
            }
          />
          <span className="settings-panel__label">
            Share anonymized usage data
          </span>
        </label>
      </div>

      <p className="settings-panel__hint">
        Default: OFF. Your data stays local unless you choose to share.
      </p>
      <p className="settings-panel__hint">
        This helps us understand what features actually change behavior.
      </p>

      <button
        className="btn btn--ghost btn--sm"
        onClick={() => setExpanded(!expanded)}
        aria-expanded={expanded}
      >
        {expanded ? "Hide details" : "What data?"}
      </button>

      {expanded && (
        <ul className="settings-panel__telemetry-details">
          <li>Daily standing percentage</li>
          <li>Position changes count</li>
          <li>Longest session duration</li>
          <li>Daily score</li>
          <li>Total active minutes</li>
          <li>App version and OS</li>
          <li>Random device ID (not linked to you)</li>
        </ul>
      )}
    </div>
  );
};

export default TelemetrySection;
