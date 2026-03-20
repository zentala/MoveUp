/**
 * App.tsx — root component for the Desk ergonomics tracker.
 *
 * Renders live session data (state, progress, today's stats) sourced from
 * the Rust Tauri backend via the useDesk hook. Auto-connects on start.
 * Shows SettingsPanel on first run until settings are configured.
 */
import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useDesk } from "@/hooks/useDesk";
import { useTimer } from "@/hooks/useTimer";
import { formatDuration } from "@/utils/format";
import SettingsPanel from "@/components/SettingsPanel";
import HeightRail from "@/components/HeightRail";
import SessionProgress from "@/components/SessionProgress";
import StateIndicator from "@/components/StateIndicator";
import TodayStats from "@/components/TodayStats";
import AppProgressBar from "@/components/AppProgressBar";
import ScreenProgressBar from "@/components/ScreenProgressBar";
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
  const [showSettings, setShowSettings] = useState(false);
  const { connected, port, state, deskHeightCm, sittingSeconds, breakSeconds, sessionLimitSecs, error } =
    useDesk();

  const liveSitting = useTimer(sittingSeconds, state === "Sitting");
  const liveBreak   = useTimer(breakSeconds,   state !== "Sitting" && state !== null);

  // Debug: poll overlay state every 2s
  const [overlayDebug, setOverlayDebug] = useState<Record<string, unknown> | null>(null);
  const pollOverlay = useCallback(() => {
    invoke("get_overlay_state").then((s) => setOverlayDebug(s as Record<string, unknown>)).catch((e) => console.warn("overlay debug:", e));
  }, []);
  useEffect(() => {
    pollOverlay();
    const id = setInterval(pollOverlay, 2000);
    return () => clearInterval(id);
  }, [pollOverlay]);

  // Show settings on first run (before user has completed setup)
  useEffect(() => {
    function checkSettings() {
      const done = localStorage.getItem("desk_setup_done");
      if (!done) {
        setShowSettings(true);
      }
    }
    checkSettings();
  }, []);

  async function handleStop() {
    await invoke("stop_reading").catch(console.error);
  }

  const showProgress = sessionLimitSecs > 0 && state === "Sitting";
  const showBreak    = state !== "Sitting" && state !== null && liveBreak > 0;

  // If settings panel is open, show only that
  if (showSettings) {
    return (
      <main className="app">
        <SettingsPanel onClose={() => setShowSettings(false)} />
      </main>
    );
  }

  return (
    <>
      {/* SCREEN OVERLAY: 4px progress bar on top of EVERYTHING */}
      {showProgress && (
        <ScreenProgressBar
          sittingSeconds={liveSitting}
          limitSeconds={sessionLimitSecs}
        />
      )}

      <main className="app">

        {/* DEBUG: V2 Progress bar — 14px inside app window */}
        {showProgress && (
          <AppProgressBar
            sittingSeconds={liveSitting}
            limitSeconds={sessionLimitSecs}
          />
        )}

      {/* Header — app identity + connection status + settings button */}
      <div className="app__header">
        <span className="app__title">↕ desk</span>
        <div className="app__header-right">
          <div className="connection-status">
            <span className={`status-dot ${statusDotClass(connected, error !== null && !connected)}`} />
            <span>{connectionLabel(connected, port)}</span>
          </div>
          <button
            className="btn btn--link"
            onClick={() => setShowSettings(true)}
            title="Open settings"
          >
            ⚙
          </button>
        </div>
      </div>

      {/* Main state card — HeightRail is the left accent */}
      <div className="panel-row">
        <HeightRail deskHeightCm={deskHeightCm} state={state}>
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

      {/* Debug: overlay state */}
      {overlayDebug && (
        <div style={{ fontSize: "10px", opacity: 0.7, padding: "4px 8px", fontFamily: "monospace" }}>
          overlay: {String(overlayDebug.data_source)} | {String(overlayDebug.progress_pct)} | visible={String(overlayDebug.visible)} | h={String(overlayDebug.bar_height)}px
        </div>
      )}

      {/* Error banner */}
      {error && <div className="error-banner">{error}</div>}

      {/* Actions */}
      <div className="actions">
        <button className="btn btn--danger" onClick={handleStop}>stop</button>
      </div>

      </main>
    </>
  );
}
