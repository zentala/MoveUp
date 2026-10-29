/**
 * StateIndicator.tsx — current ergonomic state with status dot, desk height, and activity.
 */
import type { FC } from "react";
import type { DeskState } from "@/types";
import { formatIdleTime } from "@/utils/format";

interface StateIndicatorProps {
  state: DeskState | null;
  deskHeightCm: number;
  /** Current system idle seconds. */
  idleSecs?: number;
  /** Whether to show activity status. */
  showActivity?: boolean;
  /** Source-qualified label for the state shown in the daily header. */
  stateLabel?: string | null;
  /** Whether this render has a current observed height to display. */
  showHeight?: boolean;
}

const STATE_DOT_CLASS: Record<DeskState, string> = {
  Sitting:  "state-indicator__dot--sitting",
  Standing: "state-indicator__dot--standing",
  Walking:  "state-indicator__dot--walking",
  Away:     "state-indicator__dot--away",
};

const StateIndicator: FC<StateIndicatorProps> = ({
  state,
  deskHeightCm,
  idleSecs = 0,
  showActivity = false,
  stateLabel,
  showHeight = true,
}) => {
  const dotClass = state ? STATE_DOT_CLASS[state] : "";
  const isAway = state === "Away";
  const isIdle = idleSecs >= 30;
  const label = stateLabel === undefined ? state ?? "Unknown" : stateLabel ?? "Unknown";

  return (
    <div className="state-indicator">
      <div className="state-indicator__left">
        <span className={`state-indicator__dot ${dotClass}`} />
        <span className="state-indicator__label">{label}</span>
        {showActivity && (
          <span className={`state-indicator__activity ${isIdle ? "state-indicator__activity--idle" : ""}`}>
            {isIdle ? `No recent input · ${formatIdleTime(idleSecs)}` : "Recent input"}
          </span>
        )}
      </div>
      {!isAway && showHeight && (
        <span className="state-indicator__height">
          {deskHeightCm > 0 ? `${deskHeightCm.toFixed(0)} cm` : "— cm"}
        </span>
      )}
    </div>
  );
};

export default StateIndicator;
