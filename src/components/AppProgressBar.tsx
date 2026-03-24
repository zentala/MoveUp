/**
 * AppProgressBar.tsx — Debug version: 14px progress bar at the top of the app window.
 *
 * Shows sitting session progress as a colored bar:
 * - Green (0-60%)
 * - Amber (60-85%)
 * - Red (85%+)
 */
import type { FC } from "react";
import { sittingColorForRatio } from "@/utils/colors";

interface AppProgressBarProps {
  sittingSeconds: number;
  limitSeconds: number;
}

const AppProgressBar: FC<AppProgressBarProps> = ({ sittingSeconds, limitSeconds }) => {
  const ratio = limitSeconds > 0 ? Math.min(sittingSeconds / limitSeconds, 1) : 0;
  const pct = Math.round(ratio * 100);
  const color = sittingColorForRatio(ratio);

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
