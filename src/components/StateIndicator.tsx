/**
 * StateIndicator.tsx — displays current ergonomic state with icon and desk height.
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

/** Maps each DeskState to an icon and label. */
const STATE_DISPLAY: Record<DeskState, { icon: string; label: string }> = {
  Sitting: { icon: "🪑", label: "Sitting" },
  Standing: { icon: "🧍", label: "Standing" },
  Walking: { icon: "🚶", label: "Walking" },
  Away: { icon: "💤", label: "Away" },
};

/**
 * Shows the current desk state (icon + label) alongside the measured desk height.
 */
const StateIndicator: FC<StateIndicatorProps> = ({ state, deskHeightCm }) => {
  const display = state ? STATE_DISPLAY[state] : null;

  return (
    <div className="state-indicator">
      <span className="state-icon-label">
        {display ? `${display.icon} ${display.label}` : "— Unknown"}
      </span>
      <span className="state-height">
        {deskHeightCm > 0 ? `${deskHeightCm.toFixed(1)} cm` : "— cm"}
      </span>
    </div>
  );
};

export default StateIndicator;
