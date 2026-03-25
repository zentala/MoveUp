/**
 * ConnectionOverlay.tsx — connection status indicator for remote display mode.
 *
 * Renders one of three states:
 * 1. Both WS + sensor connected -> nothing shown
 * 2. WS connected + sensor lost -> subtle top banner
 * 3. WS disconnected -> semi-transparent overlay with "Reconnecting..."
 */
import { useState, useEffect } from "react";

/** Props for the ConnectionOverlay component. */
export interface ConnectionOverlayProps {
  /** Whether the WebSocket to the backend is connected. */
  wsConnected: boolean;
  /** Whether the physical sensor is connected to the PC. */
  sensorConnected: boolean;
}

/**
 * Shows connection status in remote display mode.
 *
 * When WS is disconnected, renders a semi-transparent full-screen overlay
 * on top of stale dashboard data. When only the sensor is lost, shows
 * a subtle banner at the top.
 */
export function ConnectionOverlay({ wsConnected, sensorConnected }: ConnectionOverlayProps) {
  const [lastConnectedTime, setLastConnectedTime] = useState<string | null>(null);

  // Track when WS was last connected
  useEffect(() => {
    if (wsConnected) {
      const now = new Date();
      const hh = String(now.getHours()).padStart(2, "0");
      const mm = String(now.getMinutes()).padStart(2, "0");
      setLastConnectedTime(`${hh}:${mm}`);
    }
  }, [wsConnected]);

  // State 1: both connected -> render nothing
  if (wsConnected && sensorConnected) {
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

  // State 2: WS connected + sensor lost -> subtle banner
  return (
    <div className="conn-banner" data-testid="conn-overlay-sensor-lost">
      <span className="conn-banner__icon">&#9888;</span>
      <span className="conn-banner__text">Sensor disconnected</span>
    </div>
  );
}
