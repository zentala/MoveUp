/**
 * CalibrationSection.tsx — desk height calibration settings section.
 */
import type { FC } from "react";
import type { DeskSettings } from "./SettingsTypes";
import { SetupHelpSection } from "./SetupHelpSection";

interface CalibrationSectionProps {
  settings: DeskSettings;
  onChange: (settings: DeskSettings) => void;
  validationError: string | null;
  sittingInvalid: boolean;
  standingInvalid: boolean;
}

const CalibrationSection: FC<CalibrationSectionProps> = ({
  settings,
  onChange,
  validationError,
  sittingInvalid,
  standingInvalid,
}) => (
  <div className="settings-panel__section">
    <h3 className="settings-panel__section-title">Calibration</h3>
    <p className="settings-panel__helper">
      Confirm that a real sensor reading changes as the desk moves, then record the sitting and standing heights in millimeters.
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
        aria-invalid={sittingInvalid}
        aria-describedby={sittingInvalid && validationError ? "calibration-validation-error" : undefined}
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
        aria-invalid={standingInvalid}
        aria-describedby={standingInvalid && validationError ? "calibration-validation-error" : undefined}
        onChange={(e) =>
          onChange({ ...settings, standing_mm: parseInt(e.target.value, 10) })
        }
        className="settings-panel__input"
        required
      />
    </div>

    {validationError && (
      <div id="calibration-validation-error" className="settings-panel__validation-error" role="alert">
        {validationError}
      </div>
    )}

    <SetupHelpSection />
  </div>
);

export default CalibrationSection;
