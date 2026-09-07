/**
 * ConnectionOverlay.tsx — connection status indicator for remote display mode.
 *
 * Renders one of four states:
 * 1. WS + desk + sensor all fine -> nothing shown
 * 2. WS connected + sensor lost -> subtle top banner
 * 3. WS disconnected -> semi-transparent overlay with "Reconnecting..."
 * 4. WS connected + desk offline -> overlay with the last known state (E022)
 */
import { useState, useEffect } from "react";

/** Local wall-clock `HH:MM`, the only time format this overlay shows. */
function clockNow(): string {
  const now = new Date();
  const hh = String(now.getHours()).padStart(2, "0");
  const mm = String(now.getMinutes()).padStart(2, "0");
  return `${hh}:${mm}`;
}

/** Props for the ConnectionOverlay component. */
export interface ConnectionOverlayProps {
  /** Whether the WebSocket to the backend is connected. */
  wsConnected: boolean;
  /** Whether the physical sensor is connected to the PC. */
  sensorConnected: boolean;
  /**
   * Whether the desk itself is reachable.
   *
   * Only the relay transport can report `false` here — through it the phone
   * holds a perfectly healthy socket to a room whose desk is asleep, and
   * "Reconnecting…" would be a lie about a connection that is fine. Defaults
   * to `true` so the LAN and Tauri callers keep their three states.
   */
  deskOnline?: boolean;
}

/**
 * Shows connection status in remote display mode.
 *
 * When WS is disconnected, renders a semi-transparent full-screen overlay
 * on top of stale dashboard data. When only the sensor is lost, shows
 * a subtle banner at the top.
 */
export function ConnectionOverlay({
  wsConnected,
  sensorConnected,
  deskOnline = true,
}: ConnectionOverlayProps) {
  const [lastConnectedTime, setLastConnectedTime] = useState<string | null>(null);
  const [deskSeenTime, setDeskSeenTime] = useState<string | null>(null);

  // Track when WS was last connected
  useEffect(() => {
    if (wsConnected) {
      setLastConnectedTime(clockNow());
    }
  }, [wsConnected]);

  // Track when the desk itself was last online.
  useEffect(() => {
    if (deskOnline) {
      setDeskSeenTime(clockNow());
    }
  }, [deskOnline]);

  // State 1: everything up -> render nothing
  if (wsConnected && sensorConnected && deskOnline) {
    return null;
  }

  // State 3: WS disconnected -> full overlay (takes precedence over sensor banner)
  if (!wsConnected) {
    return (
      <div className="conn-overlay" data-testid="conn-overlay-reconnecting">
        <div className="conn-overlay__box">
          <div className="conn-overlay__spinner">&#8635;</div>
          <div className="conn-overlay__text">Reconnecting...</div>
          {lastConnectedTime && (
            <div className="conn-overlay__subtext">
              Last connected: {lastConnectedTime}
            </div>
          )}
        </div>
      </div>
    );
  }

  // State 4: relay up but the desk is not -> the data is real, just stale.
  if (!deskOnline) {
    return (
      <div className="conn-overlay conn-overlay--desk" data-testid="conn-overlay-desk-offline">
        <div className="conn-overlay__box">
          <div className="conn-overlay__text">
            {deskSeenTime
              ? `Desk offline since ${deskSeenTime}`
              : "Desk offline"}
          </div>
          <div className="conn-overlay__subtext">Showing last known state</div>
        </div>
      </div>
    );
  }

  // State 2: WS connected + sensor lost -> subtle banner
  return (
    <div className="conn-banner" data-testid="conn-overlay-sensor-lost">
      <span className="conn-banner__icon">&#9888;</span>
      <span className="conn-banner__text">Sensor disconnected</span>
    </div>
  );
}
