/**
 * OneBarTimeline.tsx — timeline showing proportional session blocks for today.
 *
 * Renders each session as a colored block (red=sitting, green=standing, gray=away).
 * Current session has a glowing right edge. Hover shows tooltip with details.
 */
import { type FC, useState } from "react";
import type { SessionEntry, WidgetProps } from "@/types";
import { formatDurationShort } from "@/utils/format";

/** Color for a session block based on state. */
function blockColor(state: string): string {
  switch (state) {
    case "Sitting":
      return "#8a2a2a";
    case "Standing":
    case "Walking":
      return "#1a6630";
    case "Away":
      return "#333";
    default:
      return "#222";
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
  x: number;
  y: number;
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

  const handleMouseEnter = (
    entry: SessionEntry,
    event: React.MouseEvent,
  ): void => {
    const rect = event.currentTarget.getBoundingClientRect();
    const label = entry.state.toLowerCase();
    const time = formatTime(entry.start);
    const dur = formatDurationShort(entry.duration_secs);
    setTooltip({
      text: `${time} \u2014 ${label} ${dur}`,
      x: rect.left + rect.width / 2,
      y: rect.top,
    });
  };

  const handleMouseLeave = (): void => setTooltip(null);

  return (
    <div className="one-bar__timeline" data-testid="one-bar-timeline">
      <div className="one-bar__timeline-bar">
        {sessions.map((entry, i) => {
          const widthPct = (entry.duration_secs / maxSecs) * 100;
          const isLast = i === sessions.length - 1 && entry.end === null;
          return (
            <div
              key={`${entry.start}-${i}`}
              className={`one-bar__timeline-block${isLast ? " one-bar__timeline-block--current" : ""}`}
              style={{
                width: `${Math.max(widthPct, 0.5)}%`,
                backgroundColor: blockColor(entry.state),
              }}
              onMouseEnter={(e) => handleMouseEnter(entry, e)}
              onMouseLeave={handleMouseLeave}
            />
          );
        })}
      </div>
      {tooltip && (
        <div
          className="one-bar__timeline-tooltip"
          style={{ left: tooltip.x, top: tooltip.y }}
        >
          {tooltip.text}
        </div>
      )}
    </div>
  );
};
