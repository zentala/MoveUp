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
import { useState, type FC } from "react";
import type { WidgetProps } from "@/types";
import StateIndicator from "@/components/StateIndicator";
import { computeTemperature } from "./one-bar/temperature";
import { useTimelineSkin, skinClassName } from "@/hooks/useTimelineSkin";
import { OneBarTimeline } from "./one-bar/OneBarTimeline";
import { OneBarTimer } from "./one-bar/OneBarTimer";
import { KpiStrip } from "./one-bar/KpiStrip";
import { StepsWidget } from "@/components/StepsWidget";
import { ShareStats } from "@/components/ShareStats";
import "./one-bar/one-bar.css";

/** Header props with share callback. */
interface HeaderProps extends WidgetProps {
  onOpenShare: () => void;
}

/** Header row: StateIndicator (dot + label + height + activity) + share + gear. */
const OneBarHeader: FC<HeaderProps> = (props) => (
  <div className="one-bar__header">
    <StateIndicator
      state={props.state}
      deskHeightCm={props.deskHeightCm}
      idleSecs={props.idleSecs}
      showActivity
    />
    <div className="one-bar__header-actions">
      <button
        className="one-bar__share-btn"
        onClick={props.onOpenShare}
        title="Share my stats"
      >
        {"\u{2197}"}
      </button>
      <button
        className="one-bar__settings-btn"
        onClick={props.onOpenSettings}
        title="Settings"
      >
        {"\u2699"}
      </button>
    </div>
  </div>
);

/** One Bar widget — horizontal layout with header, timer, KPIs, timeline. */
export const OneBarWidget: FC<WidgetProps> = (props) => {
  const [showShare, setShowShare] = useState(false);
  const temperature = computeTemperature(props);
  const [skin] = useTimelineSkin();

  return (
    <div
      className={`one-bar one-bar--${temperature} ${skinClassName(skin)}`}
      data-testid="one-bar-widget"
    >
      <OneBarHeader {...props} onOpenShare={() => setShowShare(true)} />
      <OneBarTimer {...props} />
      <KpiStrip metrics={props.metrics}>
        <StepsWidget />
      </KpiStrip>
      <OneBarTimeline {...props} />

      {showShare && (
        <ShareStats
          metrics={props.metrics}
          todayChanges={props.todayChanges}
          todayStandingSecs={props.todayStandingSecs}
          todaySittingSecs={props.todaySittingSecs}
          todayScore={props.todayScore}
          onClose={() => setShowShare(false)}
        />
      )}
    </div>
  );
};
