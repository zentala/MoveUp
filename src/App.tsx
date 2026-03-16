/**
 * App.tsx — root component for the Desk ergonomics tracker.
 *
 * Renders live session data (state, progress, today's stats) sourced from
 * the Rust Tauri backend via the useDesk hook. Auto-connects on start.
 */
import { invoke } from "@tauri-apps/api/core";
import { useDesk } from "@/hooks/useDesk";
import { useTimer } from "@/hooks/useTimer";
import SessionProgress from "@/components/SessionProgress";
import StateIndicator from "@/components/StateIndicator";
import TodayStats from "@/components/TodayStats";
import "@/styles/globals.css";

/** Determine status dot CSS modifier based on connection/error state. */
function statusDotClass(connected: boolean, hasError: boolean): string {
  if (hasError) return "status-dot--red";
  if (connected) return "status-dot--green";
  return "status-dot--yellow";
}

/** Describe connection state in a short human-readable phrase. */
function connectionLabel(connected: boolean, port: string | null): string {
  if (connected && port) return `Connected on ${port}`;
  if (connected) return "Connected";
  return "Scanning…";
}

export default function App() {
  const { connected, port, state, deskHeightCm, sittingSeconds, breakSeconds, sessionLimitSecs, error } =
    useDesk();

  // Live-ticking counter so the timer updates every second without waiting for events.
  // Only runs while the state is Sitting; pauses otherwise.
  const liveSitting = useTimer(sittingSeconds, state === "Sitting");
  const liveBreak = useTimer(breakSeconds, state !== "Sitting" && state !== null);

  async function handleStop() {
    await invoke("stop_reading").catch(console.error);
  }

  async function handleCalibrate() {
    await invoke("calibrate", {}).catch(console.error);
  }

  const showProgress = sessionLimitSecs > 0 && state === "Sitting";

  return (
    <main className="app">
      <h1 className="app__title">↕ Desk</h1>

      {/* State + height row */}
      <div className="card">
        <StateIndicator state={state} deskHeightCm={deskHeightCm} />
      </div>

      {/* Session progress bar — only visible while sitting */}
      {showProgress && (
        <div className="card">
          <SessionProgress
            sittingSeconds={liveSitting}
            limitSeconds={sessionLimitSecs}
          />
        </div>
      )}

      {/* Break info when not sitting */}
      {state !== "Sitting" && state !== null && liveBreak > 0 && (
        <div className="card">
          <span className="today-stats">
            Break: <strong>&nbsp;{Math.floor(liveBreak / 60)}m {liveBreak % 60}s</strong>
          </span>
        </div>
      )}

      {/* Today's totals */}
      <div className="card">
        <TodayStats />
      </div>

      <hr className="divider" />

      {/* Connection status */}
      <div className="connection-status">
        <span className={`status-dot ${statusDotClass(connected, error !== null && !connected)}`} />
        <span>{connectionLabel(connected, port)}</span>
      </div>

      {/* Error banner */}
      {error && <div className="error-banner">⚠ {error}</div>}

      {/* Action buttons */}
      <div className="actions">
        <button className="btn btn--danger" onClick={handleStop}>
          Stop
        </button>
        <button className="btn" onClick={handleCalibrate}>
          Calibrate
        </button>
      </div>
    </main>
  );
}
