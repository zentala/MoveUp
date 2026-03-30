/**
 * StateIndicator.tsx — current ergonomic state with status dot, desk height, and activity.
 */
import type { FC } from "react";
import type { DeskState } from "@/types";

interface StateIndicatorProps {
  state: DeskState | null;
  deskHeightCm: number;
  /** Current system idle seconds. */
  idleSecs?: number;
  /** Whether to show activity status. */
  showActivity?: boolean;
}

const STATE_DOT_CLASS: Record<DeskState, string> = {
  Sitting:  "state-indicator__dot--sitting",
  Standing: "state-indicator__dot--standing",
  Walking:  "state-indicator__dot--walking",
  Away:     "state-indicator__dot--away",
};

/** Format seconds into "Xm Ys" string. */
function formatIdleTime(secs: number): string {
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${m}m ${s}s`;
}

const StateIndicator: FC<StateIndicatorProps> = ({
  state,
  deskHeightCm,
  idleSecs = 0,
  showActivity = false,
}) => {
  const dotClass = state ? STATE_DOT_CLASS[state] : "";
  const isAway = state === "Away";
  const isIdle = idleSecs >= 30;

  return (
    <div className="state-indicator">
      <div className="state-indicator__left">
        <span className={`state-indicator__dot ${dotClass}`} />
        <span className="state-indicator__label">{state ?? "Unknown"}</span>
        {showActivity && (
          <span className={`state-indicator__activity ${isIdle ? "state-indicator__activity--idle" : ""}`}>
            {isIdle ? `Idle ${formatIdleTime(idleSecs)}` : "Active"}
          </span>
        )}
      </div>
      {!isAway && (
        <span className="state-indicator__height">
          {deskHeightCm > 0 ? `${deskHeightCm.toFixed(0)} cm` : "— cm"}
        </span>
      )}
    </div>
  );
};

export default StateIndicator;
