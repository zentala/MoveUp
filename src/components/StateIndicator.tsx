/**
 * StateIndicator.tsx — current ergonomic state with status dot and desk height.
 */
import type { FC } from "react";
import type { DeskState } from "@/types";

/** Props for the StateIndicator component. */
interface StateIndicatorProps {
  /** Current ergonomic state, or null when no data is available yet. */
  state: DeskState | null;
  /** Current desk height in centimeters. */
  deskHeightCm: number;
}

const STATE_DOT_CLASS: Record<DeskState, string> = {
  Sitting:  "state-indicator__dot--sitting",
  Standing: "state-indicator__dot--standing",
  Walking:  "state-indicator__dot--walking",
  Away:     "state-indicator__dot--away",
};

/** Displays the current desk state (dot + label) alongside the measured height. */
const StateIndicator: FC<StateIndicatorProps> = ({ state, deskHeightCm }) => {
  const dotClass = state ? STATE_DOT_CLASS[state] : "";

  return (
    <div className="state-indicator">
      <div className="state-indicator__left">
        <span className={`state-indicator__dot ${dotClass}`} />
        <span className="state-indicator__label">{state ?? "Unknown"}</span>
      </div>
      <span className="state-indicator__height">
        {deskHeightCm > 0 ? `${deskHeightCm.toFixed(0)} cm` : "— cm"}
      </span>
    </div>
  );
};

export default StateIndicator;
