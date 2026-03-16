/**
 * App.tsx — root component for the Desk ergonomics tracker.
 *
 * Renders live session data (state, progress, today's stats) sourced from
 * the Rust Tauri backend via the useDesk hook. Auto-connects on start.
 * Shows CalibrationWizard on first run until calibration is complete.
 */
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useDesk } from "@/hooks/useDesk";
import { useTimer } from "@/hooks/useTimer";
import { formatDuration } from "@/utils/format";
import CalibrationWizard from "@/components/CalibrationWizard";
import HeightRail from "@/components/HeightRail";
import SessionProgress from "@/components/SessionProgress";
import StateIndicator from "@/components/StateIndicator";
import TodayStats from "@/components/TodayStats";
import "@/styles/globals.css";

function statusDotClass(connected: boolean, hasError: boolean): string {
  if (hasError) return "status-dot--red";
  if (connected) return "status-dot--green";
  return "status-dot--yellow";
}

function connectionLabel(connected: boolean, port: string | null): string {
  if (connected && port) return port;
  if (connected) return "connected";
  return "scanning…";
}

export default function App() {
  const [calibrated, setCalibrated] = useState(() => localStorage.getItem("desk:calibrated") === "1");
  const { connected, port, state, deskHeightCm, sittingSeconds, breakSeconds, sessionLimitSecs, error } =
    useDesk();

  const liveSitting = useTimer(sittingSeconds, state === "Sitting");
  const liveBreak   = useTimer(breakSeconds,   state !== "Sitting" && state !== null);

  async function handleStop() {
    await invoke("stop_reading").catch(console.error);
  }

  const showProgress = sessionLimitSecs > 0 && state === "Sitting";
  const showBreak    = state !== "Sitting" && state !== null && liveBreak > 0;

  return (
    <main className="app">

      {/* Header — app identity + connection status */}
      <div className="app__header">
        <span className="app__title">↕ desk</span>
        <div className="connection-status">
          <span className={`status-dot ${statusDotClass(connected, error !== null && !connected)}`} />
          <span>{connectionLabel(connected, port)}</span>
        </div>
      </div>

      {/* First-run calibration wizard */}
      {!calibrated && connected && (
        <CalibrationWizard deskHeightCm={deskHeightCm} onComplete={() => setCalibrated(true)} />
      )}

      {/* Main state card — HeightRail is the left accent */}
      <div className="panel-row">
        <HeightRail deskHeightCm={deskHeightCm}>
          <StateIndicator state={state} deskHeightCm={deskHeightCm} />

          {/* Session timer — visible while sitting */}
          {showProgress && (
            <SessionProgress
              sittingSeconds={liveSitting}
              limitSeconds={sessionLimitSecs}
            />
          )}

          {/* Break duration — visible when not sitting */}
          {showBreak && (
            <div className="break-info">
              <span className="break-info__duration">{formatDuration(liveBreak)}</span>
              <span className="break-info__label">break</span>
            </div>
          )}
        </HeightRail>
      </div>

      {/* Today's totals */}
      <div className="panel-row">
        <TodayStats />
      </div>

      {/* Error banner */}
      {error && <div className="error-banner">{error}</div>}

      {/* Actions */}
      <div className="actions">
        <button className="btn btn--danger" onClick={handleStop}>stop</button>
        <button className="btn" onClick={() => setCalibrated(false)}>re-calibrate</button>
      </div>

    </main>
  );
}
