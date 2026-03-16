/**
 * CalibrationWizard.tsx — step-by-step modal for first-run desk calibration.
 *
 * Step 1: User sets desk to sitting position → records sitting height.
 * Step 2: User sets desk to standing position → records standing height.
 * On finish: calls the `calibrate` Tauri command and marks calibration done.
 */
import { useState } from "react";
import type { FC } from "react";
import { invoke } from "@tauri-apps/api/core";

/** Props for CalibrationWizard. */
export interface CalibrationWizardProps {
  /** Current desk height in cm, from the sensor (live). */
  deskHeightCm: number;
  /** Called after calibration is saved successfully. */
  onComplete: () => void;
}

const DESK_THICKNESS_MM = 30;

/**
 * Modal wizard guiding the user through sitting + standing height calibration.
 * Shown on first run when localStorage `desk:calibrated` is not set.
 */
const CalibrationWizard: FC<CalibrationWizardProps> = ({ deskHeightCm, onComplete }) => {
  const [step, setStep] = useState<1 | 2>(1);
  const [sittingMm, setSittingMm] = useState<number | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function confirmSitting() {
    setSittingMm(Math.round(deskHeightCm * 10));
    setStep(2);
  }

  async function confirmStanding() {
    if (sittingMm === null) return;
    const standingMm = Math.round(deskHeightCm * 10);
    setBusy(true);
    setError(null);
    try {
      await invoke("calibrate", {
        sitting_mm: sittingMm,
        standing_mm: standingMm,
        desk_thickness_mm: DESK_THICKNESS_MM,
      });
      localStorage.setItem("desk:calibrated", "1");
      onComplete();
    } catch (err) {
      setError(String(err));
      setBusy(false);
    }
  }

  return (
    <div className="calibration-overlay">
      <div className="calibration-modal">
        <h2 className="calibration-modal__title">Desk Calibration</h2>

        {step === 1 && (
          <div className="calibration-modal__step">
            <p className="calibration-modal__instruction">
              Set your desk to the <strong>SITTING</strong> position, then click Confirm.
            </p>
            <p className="calibration-modal__height">
              Current height: <strong>{deskHeightCm > 0 ? `${deskHeightCm.toFixed(1)} cm` : "…"}</strong>
            </p>
            <button
              className="btn"
              onClick={confirmSitting}
              disabled={deskHeightCm <= 0}
            >
              Confirm Sitting Height
            </button>
          </div>
        )}

        {step === 2 && (
          <div className="calibration-modal__step">
            <p className="calibration-modal__instruction">
              Now raise your desk to the <strong>STANDING</strong> position, then click Confirm.
            </p>
            <p className="calibration-modal__height">
              Current height: <strong>{deskHeightCm > 0 ? `${deskHeightCm.toFixed(1)} cm` : "…"}</strong>
            </p>
            {sittingMm !== null && (
              <p className="calibration-modal__saved">
                Sitting height saved: {(sittingMm / 10).toFixed(1)} cm
              </p>
            )}
            <button
              className="btn"
              onClick={confirmStanding}
              disabled={deskHeightCm <= 0 || busy}
            >
              {busy ? "Saving…" : "Confirm Standing Height"}
            </button>
          </div>
        )}

        {error && <div className="error-banner">⚠ {error}</div>}

        <p className="calibration-modal__step-indicator">Step {step} of 2</p>
      </div>
    </div>
  );
};

export default CalibrationWizard;
