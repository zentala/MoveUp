/**
 * SessionProgress.tsx — sitting session timer with inline progress bar.
 *
 * Leads with the elapsed time (large) so you get the answer at a glance.
 * The thin 3px bar fills green → warn → alert as the session approaches limit.
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

/** Fill thresholds as fractions of the session limit. */
const WARN_THRESHOLD  = 0.60;
const ALERT_THRESHOLD = 0.85;

function getFillClass(ratio: number): string {
  if (ratio >= ALERT_THRESHOLD) return "progress-fill--alert";
  if (ratio >= WARN_THRESHOLD)  return "progress-fill--warn";
  return "progress-fill--ok";
}

/**
 * Sitting session progress — elapsed time prominent, bar + remaining time below.
 */
const SessionProgress: FC<SessionProgressProps> = ({ sittingSeconds, limitSeconds }) => {
  const ratio     = limitSeconds > 0 ? Math.min(sittingSeconds / limitSeconds, 1) : 0;
  const pct       = Math.round(ratio * 100);
  const fillClass = getFillClass(ratio);
  const remaining = limitSeconds - sittingSeconds;
  const isOver    = remaining <= 0;

  return (
    <div className="session-progress">
      <div className="session-progress__meta">
        <span className="session-progress__elapsed">{formatDuration(sittingSeconds)}</span>
        <span className={`session-progress__remaining${isOver ? " session-progress__remaining--alert" : ""}`}>
          {isOver ? "take a break" : `${formatDuration(remaining)} left`}
        </span>
      </div>
      <div className="progress-track">
        <div
          className={`progress-fill ${fillClass}`}
          style={{ width: `${pct}%` }}
          role="progressbar"
          aria-valuenow={pct}
          aria-valuemin={0}
          aria-valuemax={100}
          data-testid="progress-bar"
        />
      </div>
    </div>
  );
};

export default SessionProgress;
