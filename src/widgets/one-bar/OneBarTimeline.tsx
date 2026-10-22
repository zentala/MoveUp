/** The compact view of the same bounded chronology used by Analyst. */
import { type FC, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { WidgetProps } from "@/types";
import { computeHourMarkers } from "@/utils/timeline";
import { TimelineStrip } from "@/timeline/TimelineStrip";
import { sessionIntervals } from "@/timeline/intervals";
import { createOpenDayRequest, localCalendarDay, OPEN_ANALYST_DAY_KEY } from "@/analyst/open-day";

let openRequestSequence = 0;

function openAnalyst(day: string): void {
  let serializedRequest: string | null = null;
  try {
    const request = createOpenDayRequest(
      day,
      Date.now(),
      `${Date.now()}-${++openRequestSequence}`,
    );
    serializedRequest = JSON.stringify(request);
    localStorage.setItem(OPEN_ANALYST_DAY_KEY, serializedRequest);
  } catch (error) {
    console.warn("Could not persist selected Analyst day:", error);
  }
  void invoke("open_analyst_window").catch((error) => {
    console.warn("open_analyst_window failed:", error);
    if (!serializedRequest) return;
    try {
      if (localStorage.getItem(OPEN_ANALYST_DAY_KEY) === serializedRequest) {
        localStorage.removeItem(OPEN_ANALYST_DAY_KEY);
      }
    } catch {
      /* failed invocation cleanup is best-effort when storage is restricted */
    }
  });
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
  const openToday = () => openAnalyst(localCalendarDay(new Date()));

  if (startMs >= nowMs) {
    return (
      <div
        className="one-bar__timeline"
        data-testid="one-bar-timeline"
        onClick={openToday}
        role="button"
        tabIndex={0}
        aria-label="Open Analyst for today's local day"
        title="Click to open Analyst — full timeline view"
        onKeyDown={(event) => {
          if (event.key === "Enter" || event.key === " ") {
            event.preventDefault();
            openToday();
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
    <div
      className="one-bar__timeline"
      data-testid="one-bar-timeline"
      role="group"
      aria-label="Compact timeline. Focus and press Enter to open Analyst for today."
      tabIndex={0}
      title="Click the timeline background or press Enter to open Analyst for today"
      onClick={(event) => {
        if (event.target instanceof Element && event.target.closest("button, select, input, a")) return;
        openToday();
      }}
      onKeyDown={(event) => {
        if (event.target !== event.currentTarget) return;
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          openToday();
        }
      }}
    >
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
