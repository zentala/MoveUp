/**
 * ProgressBar.tsx — unified progress bar for overlay and inline usage.
 *
 * Two variants:
 * - "overlay" — fixed 4px bar at top of screen (replaces ScreenProgressBar)
 * - "inline" — embedded bar within a widget (replaces one-bar__progress)
 */
import type { FC } from "react";
import "./ProgressBar.css";

interface ProgressBarProps {
  elapsed: number;
  total: number;
  variant: "overlay" | "inline";
  colorScheme?: "sitting" | "standing" | "gray";
  shimmer?: boolean;
}

export const ProgressBar: FC<ProgressBarProps> = ({
  elapsed,
  total,
  variant,
  colorScheme = "sitting",
  shimmer = false,
}) => {
  const ratio = total > 0 ? Math.min(elapsed / total, 1) : 0;
  const pct = Math.max(ratio * 100, ratio > 0 ? 1 : 0);

  const containerClass = [
    "progress-bar",
    `progress-bar--${variant}`,
    `progress-bar--${colorScheme}`,
    shimmer ? "progress-bar--shimmer" : "",
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <div className={containerClass} data-testid="progress-bar-container">
      <div
        className="progress-bar__fill"
        style={{ width: `${pct}%` }}
        data-testid="progress-bar-fill"
      />
    </div>
  );
};
