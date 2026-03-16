/**
 * ScreenProgressBar.tsx — Fullscreen overlay progress bar (4px × screen width).
 *
 * Fixed position on top of EVERYTHING using CSS `fixed` positioning.
 * No window decorations, no shadows — just a pure 4px colored bar.
 */
import type { FC } from "react";

interface ScreenProgressBarProps {
  sittingSeconds: number;
  limitSeconds: number;
}

function getColor(ratio: number): string {
  if (ratio >= 0.85) return "#b91c1c"; // red
  if (ratio >= 0.60) return "#c2762d"; // amber
  return "#65a30d"; // green
}

const ScreenProgressBar: FC<ScreenProgressBarProps> = ({ sittingSeconds, limitSeconds }) => {
  const ratio = limitSeconds > 0 ? Math.min(sittingSeconds / limitSeconds, 1) : 0;
  const pct = Math.round(ratio * 100);
  const color = getColor(ratio);

  return (
    <div
      style={{
        position: "fixed",
        top: 0,
        left: 0,
        right: 0,
        height: "4px",
        width: "100vw",
        background: "#000000",
        zIndex: 999999,
        overflow: "hidden",
        margin: 0,
        padding: 0,
        border: "none",
        pointerEvents: "none", // Don't interfere with clicks
      }}
    >
      <div
        style={{
          width: `${pct}%`,
          height: "100%",
          background: color,
          transition: "width 0.5s ease, background 0.3s ease",
        }}
      />
    </div>
  );
};

export default ScreenProgressBar;
