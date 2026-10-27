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
import { useState, useRef, useEffect, type FC, type RefObject } from "react";
import type { WidgetProps } from "@/types";
import StateIndicator from "@/components/StateIndicator";
import { computeTemperature } from "./one-bar/temperature";
import { useTimelineSkin, skinClassName } from "@/hooks/useTimelineSkin";
import { OneBarTimeline } from "./one-bar/OneBarTimeline";
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
  shareButtonRef: RefObject<HTMLButtonElement | null>;
}

/** Header row: StateIndicator (dot + label + height + activity) + share + gear. */
const OneBarHeader: FC<HeaderProps> = ({ shareButtonRef, ...props }) => (
  <div className="one-bar__header">
    <StateIndicator
      state={props.state}
      deskHeightCm={props.deskHeightCm}
      idleSecs={props.idleSecs}
      showActivity
    />
    <div className="one-bar__header-actions">
      <button
        ref={shareButtonRef}
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
  const shareButtonRef = useRef<HTMLButtonElement>(null);
  const wasSharing = useRef(false);
  useEffect(() => {
    if (wasSharing.current && !showShare) shareButtonRef.current?.focus();
    wasSharing.current = showShare;
  }, [showShare]);
  const temperature = computeTemperature(props);
  const [skin] = useTimelineSkin();

  return (
    <div
      className={`one-bar one-bar--${temperature} ${skinClassName(skin)}`}
      data-testid="one-bar-widget"
    >
      <OneBarHeader {...props} shareButtonRef={shareButtonRef} onOpenShare={() => setShowShare(true)} />
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
