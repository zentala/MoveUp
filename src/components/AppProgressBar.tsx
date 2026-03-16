/**
 * AppProgressBar.tsx — Debug version: 14px progress bar at the top of the app window.
 *
 * Shows sitting session progress as a colored bar:
 * - Green (0-60%)
 * - Amber (60-85%)
 * - Red (85%+)
 */
import type { FC } from "react";

interface AppProgressBarProps {
  sittingSeconds: number;
  limitSeconds: number;
}

function getColor(ratio: number): string {
  if (ratio >= 0.85) return "#b91c1c"; // red
  if (ratio >= 0.60) return "#c2762d"; // amber
  return "#65a30d"; // green
}

const AppProgressBar: FC<AppProgressBarProps> = ({ sittingSeconds, limitSeconds }) => {
  const ratio = limitSeconds > 0 ? Math.min(sittingSeconds / limitSeconds, 1) : 0;
  const pct = Math.round(ratio * 100);
  const color = getColor(ratio);

  return (
    <div
      style={{
        width: "100%",
        height: "14px",
        background: "#1a1a1a",
        overflow: "hidden",
        marginBottom: "8px",
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

export default AppProgressBar;
