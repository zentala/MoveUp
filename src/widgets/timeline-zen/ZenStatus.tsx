/**
 * ZenStatus.tsx — compact status line for the Timeline Zen widget.
 *
 * Shows a colored dot, timer (remaining / limit), height, and a thin progress bar.
 * No text labels — pure numeric display.
 */
import type { FC } from "react";
import type { DeskState } from "@/types";

/** Formats seconds as mm:ss. */
function formatTimer(secs: number): string {
  const abs = Math.abs(Math.floor(secs));
  const m = Math.floor(abs / 60);
  const s = abs % 60;
  const prefix = secs < 0 ? "-" : "";
  return `${prefix}${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

interface ZenStatusProps {
  state: DeskState | null;
  limitRemaining: number;
  limitSecs: number;
  limitRatio: number;
  deskHeightCm: number;
}

/**
 * Renders a single status line with dot, timer, height and a thin progress bar.
 */
export const ZenStatus: FC<ZenStatusProps> = ({
  state,
  limitRemaining,
  limitSecs,
  limitRatio,
  deskHeightCm,
}) => {
  const stateClass = state ? state.toLowerCase() : "away";
  const ratio = Math.min(limitRatio, 1) * 100;

  return (
    <div className="zen-status" data-testid="zen-status">
      <div className="zen-status__line">
        <span className={`zen-dot zen-dot--${stateClass}`}>●</span>
        <span className="zen-timer">{formatTimer(limitRemaining)}</span>
        <span className="zen-separator">/</span>
        <span className="zen-limit">{formatTimer(limitSecs)}</span>
        <span className="zen-height">{deskHeightCm.toFixed(0)} cm</span>
      </div>
      <div className="zen-bar" data-testid="zen-bar">
        <div
          className={`zen-bar__fill zen-bar__fill--${stateClass}`}
          style={{ width: `${ratio}%` }}
        />
      </div>
    </div>
  );
};
