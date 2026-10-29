/**
 * SetupHelpSection.tsx — concise, source-backed help for getting the first reading.
 */
export function SetupHelpSection() {
  return (
    <details className="setup-help">
      <summary className="setup-help__summary">Sensor and first reading help</summary>
      <div className="setup-help__body">
        <p>
          Windows can detect the USB controller even when the assembled sensor is not working. Board detection alone is not a sensor reading.
        </p>
        <ol>
          <li>Try a short, data-capable cable connected directly to the computer, without a USB hub.</li>
          <li>In Device Manager, check whether a USB serial device appears under Ports. This confirms the controller, not that the sensor is sending readings.</li>
          <li>Move the desk and confirm that the height reading in MoveUp changes before calibrating.</li>
        </ol>
        <p>
          MoveUp infers posture from sensor readings. Missing readings stay unknown. Sitting balance includes break credit; history remains recorded elapsed time.
        </p>
      </div>
    </details>
  );
}
