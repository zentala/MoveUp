/**
 * OneBarTimeline.tsx — timeline showing proportional session blocks for today.
 *
 * Renders each session as a colored block (red=sitting, green=standing, gray=away).
 * Current session has a glowing right edge. Hover shows tooltip with details.
 */
import { type FC, useState } from "react";
import type { SessionEntry, WidgetProps } from "@/types";
import { formatDurationShort } from "@/utils/format";

/** CSS modifier class for a session block based on state. */
function blockModifier(state: string): string {
  switch (state) {
    case "Sitting":
      return "sitting";
    case "Standing":
    case "Walking":
      return "standing";
    case "Away":
      return "away";
    default:
      return "away";
  }
}

/** Format time from ISO string to HH:MM. */
function formatTime(iso: string): string {
  const d = new Date(iso);
  const h = String(d.getHours()).padStart(2, "0");
  const m = String(d.getMinutes()).padStart(2, "0");
  return `${h}:${m}`;
}

interface TimelineTooltip {
  text: string;
  leftPct: number;
}

/** Renders the session timeline with proportional blocks. */
export const OneBarTimeline: FC<WidgetProps> = (props) => {
  const [tooltip, setTooltip] = useState<TimelineTooltip | null>(null);
  const sessions = props.todaySessions;

  if (sessions.length === 0) {
    return (
      <div className="one-bar__timeline" data-testid="one-bar-timeline">
        <div className="one-bar__timeline-empty">No sessions yet</div>
      </div>
    );
  }

  const totalSecs = sessions.reduce((s, e) => s + e.duration_secs, 0);
  const maxSecs = Math.max(totalSecs, 1);

  // Compute hour markers for the timeline
  const hourMarkers: { leftPct: number; label: string }[] = [];
  if (sessions.length > 0) {
    const firstStart = new Date(sessions[0].start);
    const firstHour = firstStart.getHours();
    const now = new Date();
    const spanMs = now.getTime() - firstStart.getTime();

    if (spanMs > 0) {
      for (let h = firstHour + 1; h <= now.getHours(); h++) {
        const hourDate = new Date(firstStart);
        hourDate.setHours(h, 0, 0, 0);
        const offsetMs = hourDate.getTime() - firstStart.getTime();
        const leftPct = (offsetMs / spanMs) * 100;
        if (leftPct >= 0 && leftPct <= 100) {
          hourMarkers.push({ leftPct, label: `${h}` });
        }
      }
    }
  }

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

  const handleMouseLeave = (): void => setTooltip(null);

  let offsetPct = 0;

  return (
    <div className="one-bar__timeline" data-testid="one-bar-timeline">
      <div className="one-bar__timeline-bar">
        {sessions.map((entry, i) => {
          const widthPct = Math.max((entry.duration_secs / maxSecs) * 100, 0.5);
          const currentOffset = offsetPct;
          offsetPct += widthPct;
          const isLast = i === sessions.length - 1 && entry.end === null;
          return (
            <div
              key={`${entry.start}-${i}`}
              className={`one-bar__timeline-block one-bar__timeline-block--${blockModifier(entry.state)}${isLast ? " one-bar__timeline-block--current" : ""}`}
              style={{ width: `${widthPct}%` }}
              onMouseEnter={() => handleMouseEnter(entry, widthPct, currentOffset)}
              onMouseLeave={handleMouseLeave}
            />
          );
        })}
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
