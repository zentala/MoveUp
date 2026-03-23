/**
 * OneBarWidget.tsx — horizontal popup widget with unified progress bar,
 * temperature escalation, timeline, and one-line coach.
 *
 * One bar, one meaning: the progress bar represents limit usage.
 * Fills when sitting, drains when standing/away.
 */
import type { FC } from "react";
import type { WidgetProps } from "@/types";
import { computeTemperature } from "./one-bar/temperature";
import { OneBarTimeline } from "./one-bar/OneBarTimeline";
import { OneBarTimer } from "./one-bar/OneBarTimer";
import { KpiStrip } from "./one-bar/KpiStrip";
import "./one-bar/one-bar.css";

/** Header row: title with desk height and settings gear. */
const OneBarHeader: FC<WidgetProps> = (props) => {
  const heightLabel =
    props.deskHeightCm > 0 ? `${props.deskHeightCm.toFixed(0)} cm` : "";

  return (
    <div className="one-bar__header">
      <div>
        <span className="one-bar__title">{"\u2195"} desk</span>
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

/** One Bar widget — horizontal layout with timeline, timer, and coach. */
export const OneBarWidget: FC<WidgetProps> = (props) => {
  const temperature = computeTemperature(props);

  return (
    <div
      className={`one-bar one-bar--${temperature}`}
      data-testid="one-bar-widget"
    >
      <OneBarHeader {...props} />
      <KpiStrip metrics={props.metrics} />
      <OneBarTimeline {...props} />
      <OneBarTimer {...props} />
    </div>
  );
};
