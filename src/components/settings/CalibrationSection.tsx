/**
 * CalibrationSection.tsx — desk height calibration settings section.
 */
import type { FC } from "react";
import type { DeskSettings } from "./SettingsTypes";

interface CalibrationSectionProps {
  settings: DeskSettings;
  onChange: (settings: DeskSettings) => void;
  validationError: string | null;
}

const CalibrationSection: FC<CalibrationSectionProps> = ({
  settings,
  onChange,
  validationError,
}) => (
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
          onChange({ ...settings, sitting_mm: parseInt(e.target.value, 10) })
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
          onChange({ ...settings, standing_mm: parseInt(e.target.value, 10) })
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
);

export default CalibrationSection;
