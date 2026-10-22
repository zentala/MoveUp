/** The compact view of the same bounded chronology used by Analyst. */
import { type FC, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { WidgetProps } from "@/types";
import { computeHourMarkers } from "@/utils/timeline";
import { TimelineStrip } from "@/timeline/TimelineStrip";
import { sessionIntervals } from "@/timeline/intervals";

function openAnalyst(): void {
  void invoke("open_analyst_window").catch((error) =>
    console.warn("open_analyst_window failed:", error),
  );
}

export const OneBarTimeline: FC<WidgetProps> = (props) => {
  const [nowMs, setNowMs] = useState(Date.now);
  useEffect(() => {
    const timer = globalThis.setInterval(() => setNowMs(Date.now()), 60_000);
    return () => globalThis.clearInterval(timer);
  }, []);
  const sessions = props.todaySessions;
  const starts = sessions.map((row) => Date.parse(row.start)).filter(Number.isFinite);
  const startMs = starts.length ? Math.min(...starts) : nowMs;
  const intervals = sessionIntervals(sessions);

  if (startMs >= nowMs) {
    return (
      <div
        className="one-bar__timeline"
        data-testid="one-bar-timeline"
        onClick={openAnalyst}
        role="button"
        tabIndex={0}
        title="Click to open Analyst — full timeline view"
        onKeyDown={(event) => {
          if (event.key === "Enter" || event.key === " ") {
            event.preventDefault();
            openAnalyst();
          }
        }}
        style={{ cursor: "pointer" }}
      >
        <div className="one-bar__timeline-empty">No recorded intervals yet</div>
      </div>
    );
  }

  const window = { startMs, endMs: nowMs };
  const hourMarkers = sessions.length > 0 ? computeHourMarkers(sessions[0].start) : [];
  return (
    <div className="one-bar__timeline" data-testid="one-bar-timeline">
      <TimelineStrip
        intervals={intervals}
        window={window}
        mode="compact"
        nowMs={nowMs}
        label="Compact chronological desk history; unobserved time is unknown"
      />
      {hourMarkers.map((marker) => (
        <div
          key={marker.label}
          className="one-bar__timeline-hour"
          style={{ left: `${marker.leftPct}%` }}
        >
          <span className="one-bar__timeline-hour-label">{marker.label}</span>
        </div>
      ))}
    </div>
  );
};
