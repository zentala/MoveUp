/**
 * ProgressBar.tsx — unified progress bar with overlay and inline variants.
 *
 * Replaces ScreenProgressBar (overlay) and inline one-bar progress divs.
 * Supports sitting/standing/gray color schemes, shimmer animation,
 * and 2-layer lap visual for standing mode.
 */
import type { FC } from "react";
import { sittingColorForRatio, STANDING_LAP_BASE } from "@/utils/colors";
import "./ProgressBar.css";

interface ProgressBarProps {
  elapsed: number;
  total: number;
  variant: "overlay" | "inline";
  colorScheme: "sitting" | "standing" | "gray";
  shimmer?: boolean;
  /** Number of completed standing laps (shows dark gold base layer). */
  completedLaps?: number;
}

export const ProgressBar: FC<ProgressBarProps> = ({
  elapsed,
  total,
  variant,
  colorScheme,
  shimmer = false,
  completedLaps = 0,
}) => {
  const ratio = total > 0 ? Math.min(elapsed / total, 1) : 0;
  const pct = Math.round(ratio * 100);

  const containerClasses = [
    "progress-bar",
    `progress-bar--${variant}`,
    shimmer ? "progress-bar--shimmer" : "",
  ]
    .filter(Boolean)
    .join(" ");

  const fillClasses = [
    "progress-bar__fill",
    `progress-bar__fill--${colorScheme}`,
  ].join(" ");

  const fillStyle: React.CSSProperties = {
    width: elapsed > 0 && total > 0 ? `${Math.max(pct, 1)}%` : "0%",
  };

  if (colorScheme === "sitting") {
    fillStyle.background = sittingColorForRatio(ratio);
  }

  const containerStyle: React.CSSProperties | undefined =
    variant === "overlay" ? { pointerEvents: "none" } : undefined;

  const showLapBase = colorScheme === "standing" && completedLaps > 0;

  return (
    <div
      className={containerClasses}
      style={containerStyle}
      data-testid="progress-bar-container"
    >
      {showLapBase && (
        <div
          className="progress-bar__lap-base"
          style={{ background: STANDING_LAP_BASE }}
          data-testid="progress-bar-lap-base"
        />
      )}
      <div
        className={fillClasses}
        style={fillStyle}
        data-testid="progress-bar-fill"
      />
    </div>
  );
};
