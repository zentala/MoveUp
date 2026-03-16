/**
 * SessionProgress.tsx — inline progress bar for the current sitting session.
 *
 * Fills green → yellow → red as the session approaches its limit.
 * Shows a countdown or a break reminder when the limit is exceeded.
 */
import type { FC } from "react";
import { formatDuration } from "@/utils/format";

/** Props for the SessionProgress component. */
interface SessionProgressProps {
  /** Seconds elapsed in the current sitting session. */
  sittingSeconds: number;
  /** Session limit in seconds after which a break is recommended. */
  limitSeconds: number;
}

/** Color thresholds as fractions of the session limit. */
const YELLOW_THRESHOLD = 0.6;
const RED_THRESHOLD = 0.85;

/**
 * Returns the CSS color class name based on how much of the session is used.
 */
function getColorClass(ratio: number): string {
  if (ratio >= RED_THRESHOLD) return "progress-bar--red";
  if (ratio >= YELLOW_THRESHOLD) return "progress-bar--yellow";
  return "progress-bar--green";
}

/**
 * Inline session progress bar with time remaining label.
 * Shows "hh:mm remaining" while under limit, "⚠ Take a break!" once exceeded.
 */
const SessionProgress: FC<SessionProgressProps> = ({
  sittingSeconds,
  limitSeconds,
}) => {
  const ratio = limitSeconds > 0 ? Math.min(sittingSeconds / limitSeconds, 1) : 0;
  const pct = Math.round(ratio * 100);
  const colorClass = getColorClass(ratio);
  const remaining = limitSeconds - sittingSeconds;
  const isOver = remaining <= 0;

  return (
    <div className="session-progress">
      <div className="progress-track">
        <div
          className={`progress-bar ${colorClass}`}
          style={{ width: `${pct}%` }}
          role="progressbar"
          aria-valuenow={pct}
          aria-valuemin={0}
          aria-valuemax={100}
          data-testid="progress-bar"
        />
      </div>
      <span className="progress-label">
        {isOver ? "⚠ Take a break!" : `${formatDuration(remaining)} remaining`}
      </span>
    </div>
  );
};

export default SessionProgress;
