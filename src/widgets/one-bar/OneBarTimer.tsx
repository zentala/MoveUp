/**
 * OneBarTimer.tsx — big timer display with progress bar.
 *
 * Shows elapsed / total as the primary number (e.g., "25:00 / 40:00").
 * Uses the unified ProgressBar component for the inline progress bar.
 * State label has moved to OneBarHeader — this component focuses on numbers.
 */
import type { FC } from "react";
import type { WidgetProps } from "@/types";
import { formatDuration, formatDurationShort, colorSchemeFor } from "@/utils/format";
import { ProgressBar } from "@/components/ProgressBar";
import { useTimerAnimations } from "@/hooks/useTimerAnimations";

/** State labels used in tooltip text. */
const STATE_LABELS: Record<string, string> = {
  Sitting: "Sitting",
  Standing: "Standing",
  Walking: "Walking",
  Away: "Away",
};

/** Renders big timer, previous session info, and the progress bar. */
export const OneBarTimer: FC<WidgetProps> = (props) => {
  const stateLabel = STATE_LABELS[props.state] ?? props.state;
  const isSitting = props.state === "Sitting";

  // Standing/Walking/Away: show break duration & standing target
  const elapsed = isSitting ? props.limitUsedSecs : props.breakSecs;
  const total = isSitting ? props.limitSecs : props.standLimitSecs;

  const elapsedStr = formatDuration(elapsed);
  const totalStr = formatDuration(total);
  const isOvertime = props.limitRemaining < 0 && isSitting;
  const { shimmer } = useTimerAnimations(props.state, props.limitRatio);

  const tooltipText = `${stateLabel} for ${elapsedStr} out of ${totalStr} limit`;

  return (
    <div className="one-bar__timer" data-testid="one-bar-timer">
      <div className="one-bar__timer-row">
        <div className="one-bar__timer-left" title={tooltipText}>
          <div className="one-bar__big-number" data-testid="one-bar-big-number">
            {isOvertime && <span className="one-bar__overtime-sign">+</span>}
            {elapsedStr}
            <span className="one-bar__limit-total"> / {totalStr}</span>
          </div>
        </div>
        {props.previousSession && (
          <div className="one-bar__timer-right">
            <span className="one-bar__prev-label">previously:</span>
            <span className="one-bar__prev-detail">
              {props.previousSession.state === "Standing" ? "stood" : "away"}{" "}
              {formatDurationShort(props.previousSession.durationSecs)}
              {props.previousSession.wasEffective && " \u2713"}
            </span>
          </div>
        )}
      </div>
      <ProgressBar
        elapsed={elapsed}
        total={total}
        variant="inline"
        colorScheme={colorSchemeFor(props.state)}
        shimmer={shimmer}
        completedLaps={
          total > 0 ? Math.floor(props.breakSecs / props.standLimitSecs) : 0
        }
      />
    </div>
  );
};
