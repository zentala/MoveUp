/**
 * App.tsx — root component for the Desk ergonomics tracker.
 *
 * Uses a pluggable widget system: core provides data via useDesk,
 * the active widget handles presentation. ScreenProgressBar (overlay)
 * stays outside the widget system as a separate concern.
 */
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useDesk } from "@/hooks/useDesk";
import { useTimer } from "@/hooks/useTimer";
import { useActiveWidget } from "@/hooks/useActiveWidget";
import { resolveWidget } from "@/widgets/registry";
import SettingsPanel from "@/components/SettingsPanel";
import ScreenProgressBar from "@/components/ScreenProgressBar";
import type { WidgetProps } from "@/types";
import "@/styles/globals.css";

export default function App() {
  const [showSettings, setShowSettings] = useState(false);
  const desk = useDesk();
  const [activeWidgetId] = useActiveWidget();

  const liveSitting = useTimer(desk.sittingSeconds, desk.state === "Sitting");
  const liveBreak = useTimer(desk.breakSeconds, desk.state !== "Sitting" && desk.state !== null);

  // Debug: poll overlay state every 2s — DEV only
  const [overlayDebug, setOverlayDebug] = useState<Record<string, unknown> | null>(null);
  useEffect(() => {
    if (!import.meta.env.DEV) return;
    const pollOverlay = () => {
      invoke("get_overlay_state")
        .then((s) => setOverlayDebug(s as Record<string, unknown>))
        .catch((e) => console.warn("overlay debug:", e));
    };
    pollOverlay();
    const id = setInterval(pollOverlay, 2000);
    return () => clearInterval(id);
  }, []);

  // Show settings on first run: detect uncalibrated state
  useEffect(() => {
    async function checkFirstRun() {
      try {
        const config = await invoke<{ sitting_mm: number; standing_mm: number }>("get_settings");
        if (config.sitting_mm === 720 && config.standing_mm === 1050) {
          setShowSettings(true);
        }
      } catch {
        // Cannot determine calibration status
      }
    }
    checkFirstRun();
  }, []);

  const showOverlay = desk.sessionLimitSecs > 0 && desk.state === "Sitting";
  const ActiveWidget = resolveWidget(activeWidgetId);

  // Build WidgetProps from useDesk + useTimer
  const widgetProps: WidgetProps = {
    connected: desk.connected,
    port: desk.port,
    state: desk.state,
    deskHeightCm: desk.deskHeightCm,
    currentSessionSecs: liveSitting,
    limitSecs: desk.sessionLimitSecs,
    limitRemaining: desk.limitRemaining,
    limitRatio: desk.limitRatio,
    breakSecs: liveBreak,
    breakResetThreshold: desk.breakResetThreshold,
    breakResetProgress: desk.breakResetProgress,
    previousSession: desk.previousSession,
    todaySessions: desk.todaySessions,
    todayChanges: desk.todayChanges,
    todayStandingSecs: desk.todayStandingSecs,
    todaySittingSecs: desk.todaySittingSecs,
    todayScore: desk.dailyScore,
    error: desk.error,
    onOpenSettings: () => setShowSettings(true),
  };

  if (showSettings) {
    return (
      <main className="app">
        <SettingsPanel onClose={() => setShowSettings(false)} />
      </main>
    );
  }

  return (
    <>
      {showOverlay && (
        <ScreenProgressBar
          sittingSeconds={liveSitting}
          limitSeconds={desk.sessionLimitSecs}
        />
      )}

      <main className="app">
        <ActiveWidget {...widgetProps} />

        {/* Debug: overlay state — DEV only */}
        {import.meta.env.DEV && overlayDebug && (
          <div style={{ fontSize: "10px", opacity: 0.7, padding: "4px 8px", fontFamily: "monospace" }}>
            overlay: {String(overlayDebug.data_source)} | {String(overlayDebug.progress_pct)} | visible={String(overlayDebug.visible)} | h={String(overlayDebug.bar_height)}px
          </div>
        )}
      </main>
    </>
  );
}
