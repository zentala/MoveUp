/**
 * TimelineZenWidget.tsx — minimalist widget with a hero timeline.
 *
 * No coach text, no previous session card, no points.
 * Just: timeline + one timer line + one bar.
 */
import type { FC } from "react";
import type { WidgetProps } from "@/types";
import { ZenTimeline } from "./timeline-zen/ZenTimeline";
import { ZenStatus } from "./timeline-zen/ZenStatus";
import "./timeline-zen/timeline-zen.css";

/**
 * Timeline Zen widget — minimalist ergonomic dashboard.
 * The timeline IS the interface. Everything you need is in the
 * proportions and colors of the blocks.
 */
export const TimelineZenWidget: FC<WidgetProps> = (props) => {
  return (
    <div className="zen-widget" data-testid="timeline-zen-widget">
      <ZenTimeline sessions={props.todaySessions} height={48} />
      <ZenStatus
        state={props.state}
        limitRemaining={props.limitRemaining}
        limitSecs={props.limitSecs}
        limitRatio={props.limitRatio}
        deskHeightCm={props.deskHeightCm}
      />
    </div>
  );
};
