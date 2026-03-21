/**
 * App.tsx — root component for the Desk ergonomics tracker.
 *
 * Uses a pluggable widget system: core provides data via useWidgetData,
 * the active widget handles presentation. ScreenProgressBar (overlay)
 * stays outside the widget system as a separate concern.
 */
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useWidgetData } from "@/hooks/useWidgetData";
import { useActiveWidget } from "@/hooks/useActiveWidget";
import { resolveWidget } from "@/widgets/registry";
import SettingsPanel from "@/components/SettingsPanel";
import ScreenProgressBar from "@/components/ScreenProgressBar";
import "@/styles/globals.css";

export default function App() {
  const [showSettings, setShowSettings] = useState(false);
  const [activeWidgetId] = useActiveWidget();
  const widgetProps = useWidgetData(() => setShowSettings(true));

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

  // Show settings on first run: only if welcome_on_startup is still true
  // (welcome popup handles first-run experience, not settings)
  // Removed: auto-opening settings based on default calibration values
  // was false-positive for users with valid default calibration.

  const showOverlay = widgetProps.limitSecs > 0 && widgetProps.state === "Sitting";
  const ActiveWidget = resolveWidget(activeWidgetId);

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
          sittingSeconds={widgetProps.currentSessionSecs}
          limitSeconds={widgetProps.limitSecs}
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
