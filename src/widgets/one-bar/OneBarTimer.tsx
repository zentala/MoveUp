/**
 * OneBarTimer.tsx — big timer display with state label and progress bar.
 *
 * Shows limitRemaining (not sittingSeconds) as the primary number.
 * The progress bar has ONE meaning: fills when sitting, drains when standing/away.
 */
import type { FC } from "react";
import type { WidgetProps } from "@/types";
import { formatDuration, formatDurationShort } from "@/utils/format";

/** State indicator labels for each desk state. */
const STATE_LABELS: Record<string, string> = {
  Sitting: "sitting",
  Standing: "standing",
  Walking: "walking",
  Away: "away",
};

/** Renders state label, big timer, previous session info, and the progress bar. */
export const OneBarTimer: FC<WidgetProps> = (props) => {
  const stateLabel = props.state ? STATE_LABELS[props.state] ?? "—" : "—";
  const limitTotal = formatDuration(props.limitSecs);
  const remaining = formatDuration(Math.max(0, Math.abs(props.limitRemaining)));
  const isOvertime = props.limitRemaining < 0;
  const barWidth = Math.min(props.limitRatio * 100, 100);
  const isStandingOrAway =
    props.state === "Standing" ||
    props.state === "Walking" ||
    props.state === "Away";

  const barClass = isStandingOrAway
    ? "one-bar__progress-fill--draining"
    : "one-bar__progress-fill--filling";

  return (
    <div className="one-bar__timer" data-testid="one-bar-timer">
      <div className="one-bar__timer-row">
        <div className="one-bar__timer-left">
          <div className="one-bar__state-label">
            <span className="one-bar__state-dot" />
            {stateLabel}
            <span className="one-bar__session-duration">
              for {formatDurationShort(props.currentSessionSecs)}
            </span>
          </div>
          <div className="one-bar__big-number" data-testid="one-bar-big-number">
            {isOvertime && <span className="one-bar__overtime-sign">+</span>}
            {remaining}
            <span className="one-bar__limit-total"> / {limitTotal}</span>
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
      <div className="one-bar__progress" data-testid="one-bar-progress">
        <div
          className={`one-bar__progress-fill ${barClass}`}
          style={{ width: `${barWidth}%` }}
        />
      </div>
    </div>
  );
};
