/**
 * App.tsx — root component for the Desk ergonomics tracker.
 *
 * Core provides data via useWidgetData, OneBarWidget handles presentation.
 * Overlay progress bar is a separate native WinAPI window (not React).
 */
import { useState, useEffect, lazy, Suspense } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useWidgetData } from "@/hooks/useWidgetData";
import { ActiveWidget } from "@/widgets/registry";
import SettingsPanel from "@/components/SettingsPanel";
import { ConnectionOverlay } from "@/components/ConnectionOverlay";
import { VoiceCapture } from "@/components/VoiceCapture";
import "@/styles/globals.css";

/** Whether we are running inside Tauri (desktop) or a browser (remote display). */
const isTauri = !!window.__TAURI_INTERNALS__;

// Add remote-display class to <html> when running in browser mode
if (!isTauri) {
  document.documentElement.classList.add("remote-display");
}

const MockupGallery = lazy(() => import("@/pages/MockupGallery"));
const AnalystMockup = lazy(() => import("@/pages/AnalystMockup"));
const AnalystLive = lazy(() => import("@/pages/AnalystLive"));
const VoiceCaptureGallery = lazy(() => import("@/mockup/VoiceCaptureGallery"));

/**
 * Root router — decides which page to render based on `window.location.hash`.
 * Calls no hooks itself, so it never risks changing the hook count of a
 * conditional branch across renders (react-hooks/rules-of-hooks).
 */
export default function App() {
  // PROD/DEV: /#/analyst — live Analyst window backed by Tauri commands
  if (window.location.hash === "#/analyst") {
    return (
      <Suspense fallback={<div style={{ color: "#ccc", padding: 20 }}>Loading analyst...</div>}>
        <AnalystLive />
      </Suspense>
    );
  }
  // DEV: /#/mockup/analyst shows the Analyst dashboard mockup
  if (import.meta.env.DEV && window.location.hash === "#/mockup/analyst") {
    return (
      <Suspense fallback={<div style={{ color: "#ccc", padding: 20 }}>Loading analyst...</div>}>
        <AnalystMockup />
      </Suspense>
    );
  }
  // DEV: /#/mockup/voice shows the phone dictation panel mockups
  if (import.meta.env.DEV && window.location.hash === "#/mockup/voice") {
    return (
      <Suspense fallback={<div style={{ color: "#ccc", padding: 20 }}>Loading mockups...</div>}>
        <VoiceCaptureGallery />
      </Suspense>
    );
  }
  // DEV: /#/mockup shows the mockup gallery
  if (import.meta.env.DEV && window.location.hash === "#/mockup") {
    return (
      <Suspense fallback={<div style={{ color: "#ccc", padding: 20 }}>Loading mockups...</div>}>
        <MockupGallery />
      </Suspense>
    );
  }
  return <MainApp />;
}

/** The main desk-tracker UI — all hooks live here, called unconditionally. */
function MainApp() {
  const [showSettings, setShowSettings] = useState(false);
  const { widgetProps, wsConnected, sensorConnected } = useWidgetData(() => setShowSettings(true));
  const isRemote = !isTauri;

  // Request Wake Lock in remote display mode to keep screen on
  useEffect(() => {
    if (isTauri) return;
    if ("wakeLock" in navigator) {
      navigator.wakeLock.request("screen").catch(() => {});
    }
  }, []);

  // Listen for tray commands: show-widget resets to main, show-settings opens settings.
  // Tauri's event bridge doesn't exist in remote-display (plain browser) mode.
  useEffect(() => {
    if (!isTauri) return;
    const unWidget = listen("desk:show-widget", () => setShowSettings(false));
    const unSettings = listen("desk:show-settings", () => setShowSettings(true));
    return () => {
      unWidget.then((u) => u());
      unSettings.then((u) => u());
    };
  }, []);

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

  if (showSettings) {
    return (
      <main className="app">
        <SettingsPanel onClose={() => setShowSettings(false)} />
      </main>
    );
  }

  return (
    <main className="app">
      {isRemote && wsConnected !== undefined && (
        <ConnectionOverlay wsConnected={wsConnected} sensorConnected={sensorConnected} />
      )}
      <ActiveWidget {...widgetProps} />

      {/* Dictation is phone-only: the desktop popup already has a keyboard. */}
      {isRemote && <VoiceCapture />}

      {/* Debug: overlay state — DEV only */}
      {import.meta.env.DEV && overlayDebug && (
        <div className="app__debug">
          overlay: {String(overlayDebug.data_source)} | {String(overlayDebug.progress_pct)} | visible={String(overlayDebug.visible)} | h={String(overlayDebug.bar_height)}px
        </div>
      )}
    </main>
  );
}
