/**
 * TelemetrySection.tsx — Telemetry opt-in toggle and data transparency.
 *
 * Shows the opt-in preference and what the local telemetry payload contains.
 * This build does not upload telemetry.
 */
import { useState } from "react";
import type { FC } from "react";
import type { DeskSettings } from "./SettingsTypes";

interface TelemetrySectionProps {
  settings: DeskSettings;
  onChange: (settings: DeskSettings) => void;
}

/** Telemetry preference with expandable local-payload details. */
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
            Enable telemetry preference
          </span>
        </label>
      </div>

      <p className="settings-panel__hint">
        Off by default. Telemetry is separate from LAN display and cloud relay.
      </p>
      <p className="settings-panel__hint">
        This build does not upload usage data; enabling this preference only
        prepares and logs a daily aggregate locally.
      </p>

      <button
        className="btn btn--ghost btn--sm"
        onClick={() => setExpanded(!expanded)}
        aria-expanded={expanded}
      >
        {expanded ? "Hide details" : "What data is prepared?"}
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
