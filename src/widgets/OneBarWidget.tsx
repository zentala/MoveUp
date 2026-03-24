/**
 * OneBarWidget.tsx — horizontal popup widget with unified progress bar,
 * temperature escalation, timeline, and one-line coach.
 *
 * Visual hierarchy (top to bottom):
 * 1. Header — state dot + label + height + gear
 * 2. Timer — big number + progress bar
 * 3. KPI strip — today's metrics
 * 4. Timeline — session history at bottom
 */
import type { FC } from "react";
import type { WidgetProps } from "@/types";
import { computeTemperature } from "./one-bar/temperature";
import { OneBarTimeline } from "./one-bar/OneBarTimeline";
import { OneBarTimer } from "./one-bar/OneBarTimer";
import { KpiStrip } from "./one-bar/KpiStrip";
import { stateColor } from "@/utils/colors";
import "./one-bar/one-bar.css";

/** State labels for each desk state. */
const STATE_LABELS: Record<string, string> = {
  Sitting: "sitting",
  Standing: "standing",
  Walking: "walking",
  Away: "away",
};

/** Header row: colored state dot + label + desk height + settings gear. */
const OneBarHeader: FC<WidgetProps> = (props) => {
  const label = STATE_LABELS[props.state] ?? props.state;
  const heightLabel =
    props.deskHeightCm > 0 ? `(${props.deskHeightCm.toFixed(0)} cm)` : "";
  const dotColor = stateColor(props.state, props.limitRatio);
  const tooltip = `Current state: ${label}${heightLabel ? ` — desk at ${props.deskHeightCm.toFixed(0)} cm` : ""}`;

  return (
    <div className="one-bar__header">
      <div className="one-bar__header-left" title={tooltip}>
        <span
          className="one-bar__header-dot"
          style={{ backgroundColor: dotColor }}
        />
        <span
          className="one-bar__header-state"
          style={{ color: dotColor }}
        >
          {label}
        </span>
        <span className="one-bar__header-context">@ desk</span>
        {heightLabel && (
          <span className="one-bar__height">{heightLabel}</span>
        )}
      </div>
      <button
        className="one-bar__settings-btn"
        onClick={props.onOpenSettings}
        title="Settings"
      >
        {"\u2699"}
      </button>
    </div>
  );
};

/** One Bar widget — horizontal layout with header, timer, KPIs, timeline. */
export const OneBarWidget: FC<WidgetProps> = (props) => {
  const temperature = computeTemperature(props);

  return (
    <div
      className={`one-bar one-bar--${temperature}`}
      data-testid="one-bar-widget"
    >
      <OneBarHeader {...props} />
      <OneBarTimer {...props} />
      <KpiStrip metrics={props.metrics} />
      <OneBarTimeline {...props} />
    </div>
  );
};
