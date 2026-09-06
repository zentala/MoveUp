/**
 * OneBarTimeline.tsx — timeline showing proportional session blocks for today.
 *
 * Renders each session as a colored block (red=sitting, green=standing, gray=away).
 * Current session has a glowing right edge. Hover shows tooltip with details.
 */
import { type FC, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { SessionEntry, WidgetProps } from "@/types";
import { formatDurationShort, formatTime } from "@/utils/format";
import { computeHourMarkers } from "@/utils/timeline";

/** Open the full Analyst window — the "magnified" version of this strip. */
function openAnalyst(): void {
  void invoke("open_analyst_window").catch((err) =>
    console.warn("open_analyst_window failed:", err),
  );
}

/** CSS modifier class for a session block based on state. */
function blockModifier(state: string): string {
  switch (state) {
    case "Sitting":
      return "sitting";
    case "Standing":
      return "standing";
    case "Walking":
      return "walking";
    case "Away":
      return "away";
    default:
      return "away";
  }
}

interface TimelineTooltip {
  text: string;
  leftPct: number;
}

/** Current session duration: sitting uses limitUsedSecs, others use breakSecs. */
function currentDuration(props: WidgetProps): number {
  return props.state === "Sitting"
    ? props.limitUsedSecs
    : props.breakSecs;
}

/** Renders the session timeline with proportional blocks. */
export const OneBarTimeline: FC<WidgetProps> = (props) => {
  const [tooltip, setTooltip] = useState<TimelineTooltip | null>(null);
  const sessions = props.todaySessions;
  const liveSecs = currentDuration(props);

  const completedTotal = sessions.reduce((s, e) => s + e.duration_secs, 0);
  const totalSecs = completedTotal + liveSecs;
  const maxSecs = Math.max(totalSecs, 1);

  if (sessions.length === 0 && liveSecs === 0) {
    return (
      <div
        className="one-bar__timeline"
        data-testid="one-bar-timeline"
        onClick={openAnalyst}
        role="button"
        tabIndex={0}
        title="Click to open Analyst — full timeline view"
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            openAnalyst();
          }
        }}
        style={{ cursor: "pointer" }}
      >
        <div className="one-bar__timeline-empty">No sessions yet</div>
      </div>
    );
  }

  // Hour markers: start time, full hour boundaries, current time.
  const hourMarkers = sessions.length > 0
    ? computeHourMarkers(sessions[0].start)
    : [];

  const handleMouseEnter = (
    entry: SessionEntry,
    widthPct: number,
    offsetPct: number,
  ): void => {
    const label = entry.state.toLowerCase();
    const time = formatTime(entry.start);
    const dur = formatDurationShort(entry.duration_secs);
    setTooltip({
      text: `${time} \u2014 ${label} ${dur}`,
      leftPct: offsetPct + widthPct / 2,
    });
  };

  const handleLiveMouseEnter = (
    widthPct: number,
    offsetPct: number,
  ): void => {
    const label = props.state.toLowerCase();
    const dur = formatDurationShort(liveSecs);
    setTooltip({
      text: `now \u2014 ${label} ${dur}`,
      leftPct: offsetPct + widthPct / 2,
    });
  };

  const handleMouseLeave = (): void => setTooltip(null);

  let offsetPct = 0;

  return (
    <div className="one-bar__timeline" data-testid="one-bar-timeline">
      <div className="one-bar__timeline-bar">
        {sessions.map((entry, i) => {
          const widthPct = Math.max((entry.duration_secs / maxSecs) * 100, 0.5);
          const currentOffset = offsetPct;
          offsetPct += widthPct;
          return (
            <div
              key={`${entry.start}-${i}`}
              className={`one-bar__timeline-block one-bar__timeline-block--${blockModifier(entry.state)}`}
              style={{ width: `${widthPct}%` }}
              onMouseEnter={() => handleMouseEnter(entry, widthPct, currentOffset)}
              onMouseLeave={handleMouseLeave}
            />
          );
        })}
        {liveSecs > 0 && (
          <div
            key="live-current"
            className={`one-bar__timeline-block one-bar__timeline-block--${blockModifier(props.state)} one-bar__timeline-block--current`}
            style={{ width: `${Math.max((liveSecs / maxSecs) * 100, 0.5)}%` }}
            data-testid="timeline-live-block"
            onMouseEnter={() => {
              const w = Math.max((liveSecs / maxSecs) * 100, 0.5);
              handleLiveMouseEnter(w, offsetPct);
            }}
            onMouseLeave={handleMouseLeave}
          />
        )}
      </div>
      {hourMarkers.map((marker) => (
        <div
          key={marker.label}
          className="one-bar__timeline-hour"
          style={{ left: `${marker.leftPct}%` }}
        >
          <span className="one-bar__timeline-hour-label">{marker.label}</span>
        </div>
      ))}
      {tooltip && (
        <div
          className="one-bar__timeline-tooltip"
          style={{ left: `${tooltip.leftPct}%` }}
        >
          {tooltip.text}
        </div>
      )}
    </div>
  );
};
