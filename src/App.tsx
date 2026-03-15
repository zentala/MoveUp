/**
 * App.tsx — root component for the Desk application.
 *
 * Connects to XIAO ESP32-C3 via serial port and displays
 * live VL53L1X distance readings.
 */
import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { DistanceReading, SensorError, PortInfo } from "@/types";

const MAX_HISTORY = 50;

export default function App() {
  const [ports, setPorts] = useState<PortInfo[]>([]);
  const [selectedPort, setSelectedPort] = useState<string>("COM3");
  const [connected, setConnected] = useState(false);
  const [latest, setLatest] = useState<DistanceReading | null>(null);
  const [history, setHistory] = useState<DistanceReading[]>([]);
  const [error, setError] = useState<string | null>(null);
  const unlistenRef = useRef<(() => void)[]>([]);

  useEffect(() => {
    invoke<PortInfo[]>("list_ports").then(setPorts).catch(console.error);
  }, []);

  useEffect(() => {
    const fns = unlistenRef.current;
    return () => fns.forEach((fn) => fn());
  }, []);

  async function connect() {
    setError(null);
    try {
      await invoke("start_reading", { port: selectedPort });
      setConnected(true);

      const unDist = await listen<DistanceReading>("desk:distance", ({ payload }) => {
        setLatest(payload);
        setHistory((prev) => [payload, ...prev].slice(0, MAX_HISTORY));
      });

      const unErr = await listen<SensorError>("desk:sensor-error", ({ payload }) => {
        setError(payload.message);
      });

      unlistenRef.current = [unDist, unErr];
    } catch (e) {
      setError(String(e));
    }
  }

  async function disconnect() {
    await invoke("stop_reading");
    unlistenRef.current.forEach((fn) => fn());
    unlistenRef.current = [];
    setConnected(false);
    setLatest(null);
  }

  return (
    <div style={{ fontFamily: "monospace", padding: 24, maxWidth: 480 }}>
      <h2 style={{ marginTop: 0 }}>Desk — ToF Distance</h2>

      <div style={{ display: "flex", gap: 8, marginBottom: 16 }}>
        <select
          value={selectedPort}
          onChange={(e) => setSelectedPort(e.target.value)}
          disabled={connected}
        >
          {ports.length > 0
            ? ports.map((p) => (
                <option key={p.name} value={p.name}>
                  {p.name}{p.description ? ` — ${p.description}` : ""}
                </option>
              ))
            : <option value="COM3">COM3</option>}
        </select>
        <button onClick={connected ? disconnect : connect}>
          {connected ? "Disconnect" : "Connect"}
        </button>
        <button
          onClick={() => invoke<PortInfo[]>("list_ports").then(setPorts)}
          disabled={connected}
        >
          Refresh
        </button>
      </div>

      {error && (
        <div style={{ color: "crimson", marginBottom: 12 }}>⚠ {error}</div>
      )}

      {latest && (
        <div style={{ fontSize: 48, fontWeight: "bold", marginBottom: 24 }}>
          {latest.mm} <span style={{ fontSize: 24, fontWeight: "normal" }}>mm</span>
          <span style={{ fontSize: 20, marginLeft: 16, color: "#888" }}>
            {latest.cm.toFixed(1)} cm
          </span>
        </div>
      )}

      {history.length > 0 && (
        <table style={{ width: "100%", borderCollapse: "collapse", fontSize: 13 }}>
          <thead>
            <tr>
              <th style={{ textAlign: "left" }}>Time</th>
              <th style={{ textAlign: "right" }}>mm</th>
              <th style={{ textAlign: "right" }}>cm</th>
            </tr>
          </thead>
          <tbody>
            {history.map((r, i) => (
              <tr key={i} style={{ opacity: 1 - i * 0.015 }}>
                <td>{new Date(r.timestamp).toLocaleTimeString()}</td>
                <td style={{ textAlign: "right" }}>{r.mm}</td>
                <td style={{ textAlign: "right" }}>{r.cm.toFixed(1)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
