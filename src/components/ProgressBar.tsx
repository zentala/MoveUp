/**
 * ProgressBar.tsx — unified progress bar with overlay and inline variants.
 *
 * Replaces ScreenProgressBar (overlay) and inline one-bar progress divs.
 * Supports sitting/standing/gray color schemes and shimmer animation.
 */
import type { FC } from "react";
import { sittingColorForRatio } from "@/utils/colors";
import "./ProgressBar.css";

interface ProgressBarProps {
  elapsed: number;
  total: number;
  variant: "overlay" | "inline";
  colorScheme: "sitting" | "standing" | "gray";
  shimmer?: boolean;
}

export const ProgressBar: FC<ProgressBarProps> = ({
  elapsed,
  total,
  variant,
  colorScheme,
  shimmer = false,
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

  return (
    <div
      className={containerClasses}
      style={containerStyle}
      data-testid="progress-bar-container"
    >
      <div
        className={fillClasses}
        style={fillStyle}
        data-testid="progress-bar-fill"
      />
    </div>
  );
};
