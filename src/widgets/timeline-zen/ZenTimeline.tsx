/**
 * ZenTimeline.tsx — minimalist session timeline for the Timeline Zen widget.
 *
 * Renders today's sessions as colored blocks on a proportional timeline.
 * The timeline is the hero element (48px tall) — no legend, no labels.
 */
import type { FC } from "react";
import type { SessionEntry } from "@/types";

/** Maps desk state to CSS color class suffix. */
function stateColor(state: string): string {
  switch (state) {
    case "Sitting":
      return "sitting";
    case "Standing":
      return "standing";
    case "Walking":
      return "walking";
    default:
      return "away";
  }
}

/** Returns hour label for a given hour number. */
function hourLabel(hour: number): string {
  return String(hour).padStart(2, "0");
}

interface ZenTimelineProps {
  sessions: SessionEntry[];
}

/**
 * Renders a proportional timeline bar of today's sessions.
 * Each session is a colored block whose width reflects its duration.
 */
export const ZenTimeline: FC<ZenTimelineProps> = ({
  sessions,
}) => {
  const now = new Date();
  const dayStart = new Date(now);
  dayStart.setHours(0, 0, 0, 0);
  const totalDaySecs = (now.getTime() - dayStart.getTime()) / 1000;

  const startHour = sessions.length > 0
    ? new Date(sessions[0].start).getHours()
    : now.getHours();
  const endHour = now.getHours() + 1;
  const hours: number[] = [];
  for (let h = startHour; h <= endHour && h < 24; h++) {
    hours.push(h);
  }

  return (
    <div
      className="zen-timeline"
      data-testid="zen-timeline"
    >
      <div className="zen-timeline__blocks">
        {sessions.map((session, i) => {
          const start = new Date(session.start);
          const startSecs = (start.getTime() - dayStart.getTime()) / 1000;
          const duration = session.duration_secs;
          const left = totalDaySecs > 0 ? (startSecs / totalDaySecs) * 100 : 0;
          const width = totalDaySecs > 0 ? (duration / totalDaySecs) * 100 : 0;

          return (
            <div
              key={i}
              className={`zen-timeline__block zen-timeline__block--${stateColor(session.state)}`}
              style={{ left: `${left}%`, width: `${Math.max(width, 0.3)}%` }}
              title={`${session.state} — ${Math.round(duration / 60)}m`}
            />
          );
        })}
        <div className="zen-timeline__now" />
      </div>
      <div className="zen-timeline__hours">
        {hours.map((h) => {
          const pos = totalDaySecs > 0
            ? (((h - 0) * 3600 - (dayStart.getHours() * 3600)) / totalDaySecs) * 100
            : 0;
          return (
            <span
              key={h}
              className="zen-timeline__hour"
              style={{ left: `${Math.min(Math.max(pos, 0), 95)}%` }}
            >
              {hourLabel(h)}
            </span>
          );
        })}
      </div>
    </div>
  );
};
