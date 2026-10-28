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
import { formatObservationAnnouncement, formatObservationStatus, formatObservationViewDetails, OneBarTimeline } from "./one-bar/OneBarTimeline";
import { OneBarTimer } from "./one-bar/OneBarTimer";
import { KpiStrip } from "./one-bar/KpiStrip";
import { HealthWidget } from "@/components/HealthWidget";
import { ShareStats } from "@/components/ShareStats";
import { Settings, Share2 } from "lucide-react";
import { UiIcon } from "@/components/UiIcon";
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
    <span className="one-bar__observation-status" aria-live="off" tabIndex={0} data-testid="timeline-observation-status"
      aria-label={formatObservationStatus(props.liveObservation, props.dashboardRefresh)}
      title={formatObservationStatus(props.liveObservation, props.dashboardRefresh)}>
      {formatObservationStatus(props.liveObservation, props.dashboardRefresh).replace(/ · View .*/, "")}
      <span aria-live="off" data-testid="view-response-details"> · {formatObservationViewDetails(props.dashboardRefresh)}</span>
    </span>
    <span role="status" aria-live="polite" data-testid="timeline-observation-announcement"
      style={{ position: "absolute", width: 1, height: 1, padding: 0, margin: -1, overflow: "hidden", clip: "rect(0, 0, 0, 0)", whiteSpace: "nowrap", border: 0 }}>
      Device connection status unknown · {formatObservationAnnouncement(props.liveObservation, props.dashboardRefresh)}
    </span>
    <div className="one-bar__header-actions">
      <button
        className="one-bar__share-btn"
        type="button"
        onClick={props.onOpenShare}
        title="Share my stats"
        aria-label="Share my stats"
      >
        <UiIcon icon={Share2} />
      </button>
      <button
        className="one-bar__settings-btn"
        type="button"
        onClick={props.onOpenSettings}
        title="Settings"
        aria-label="Settings"
      >
        <UiIcon icon={Settings} />
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
        <HealthWidget />
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
