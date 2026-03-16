/**
 * overlay/main.tsx — Entry point for the top-of-screen progress bar overlay.
 *
 * Renders a 4-pixel-tall coloured bar that grows from left to right based on
 * the `overlay:progress` event payload emitted by the Rust backend.
 */

import { listen } from "@tauri-apps/api/event";
import { useState, useEffect } from "react";
import { createRoot } from "react-dom/client";

/** Payload shape emitted by `overlay::update_overlay`. */
interface OverlayPayload {
  progress: number;
  color: string;
}

/** Renders the animated progress bar. */
function OverlayBar() {
  const [payload, setPayload] = useState<OverlayPayload>({
    progress: 0,
    color: "#4caf50",
  });

  useEffect(() => {
    const unlisten = listen<OverlayPayload>("overlay:progress", ({ payload: p }) => {
      setPayload(p);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <div
      style={{
        width: `${Math.min(payload.progress * 100, 100)}%`,
        height: "4px",
        background: payload.color,
        transition: "width 0.5s ease, background 0.3s ease",
      }}
    />
  );
}

createRoot(document.getElementById("root")!).render(<OverlayBar />);
