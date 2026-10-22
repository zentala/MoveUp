/** The compact view of the same bounded chronology used by Analyst. */
import { type FC, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { WidgetProps } from "@/types";
import { computeHourMarkers } from "@/utils/timeline";
import { TimelineStrip } from "@/timeline/TimelineStrip";
import { sessionIntervals } from "@/timeline/intervals";
import { createOpenDayRequest, localCalendarDay, OPEN_ANALYST_DAY_KEY } from "@/analyst/open-day";
import type { HourMarker } from "@/utils/timeline";

let openRequestSequence = 0;

const HOUR_LABEL_GAP_PX = 8;

/** Keep the existing hour labels that fit at a readable spacing in the actual strip width. */
function selectReadableHourMarkerIndices(markers: HourMarker[], availableWidthPx: number): number[] {
  if (markers.length <= 1) return markers.map((_, index) => index);
  if (availableWidthPx <= 0) return [0, markers.length - 1];

  const boxes = markers.map((marker, index) => {
    const center = marker.leftPct / 100 * availableWidthPx;
    const width = marker.label.length * 6 + 4;
    const left = index === 0 ? center : index === markers.length - 1 ? center - width : center - width / 2;
    return { left, right: left + width };
  });
  const selected = [0];
  let right = boxes[0].right;

  for (let index = 1; index < markers.length - 1; index += 1) {
    const box = boxes[index];
    if (box.left >= right + HOUR_LABEL_GAP_PX && box.right <= boxes[boxes.length - 1].left - HOUR_LABEL_GAP_PX) {
      selected.push(index);
      right = box.right;
    }
  }

  const last = markers.length - 1;
  if (boxes[last].left >= right + HOUR_LABEL_GAP_PX) selected.push(last);
  return selected;
}

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
  const [axisWidthPx, setAxisWidthPx] = useState(0);
  const timelineRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const timer = globalThis.setInterval(() => setNowMs(Date.now()), 60_000);
    return () => globalThis.clearInterval(timer);
  }, []);
  useEffect(() => {
    const element = timelineRef.current;
    if (!element || typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver((entries) => {
      setAxisWidthPx(entries[0]?.contentRect.width ?? element.clientWidth);
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, []);
  const sessions = props.todaySessions;
  const starts = sessions.map((row) => Date.parse(row.start)).filter(Number.isFinite);
  const startMs = starts.length ? Math.min(...starts) : nowMs;
  const intervals = sessionIntervals(sessions);
  const openToday = () => openAnalyst(localCalendarDay(new Date()));

  if (startMs >= nowMs) {
    return (
      <div
        ref={timelineRef}
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
        style={{ cursor: "pointer", flex: "0 0 24px", height: 24, minHeight: 24, maxHeight: 24 }}
      >
        <div className="one-bar__timeline-empty">No recorded intervals yet</div>
      </div>
    );
  }

  const window = { startMs, endMs: nowMs };
  const hourMarkers = sessions.length > 0 ? computeHourMarkers(sessions[0].start) : [];
  const visibleHourLabels = new Set(selectReadableHourMarkerIndices(hourMarkers, axisWidthPx));
  return (
    <div
      ref={timelineRef}
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
      style={{ flex: "0 0 24px", height: 24, minHeight: 24, maxHeight: 24 }}
    >
      <TimelineStrip
        intervals={intervals}
        window={window}
        mode="compact"
        nowMs={nowMs}
        label="Compact chronological desk history; unobserved time is unknown"
      />
      {hourMarkers.map((marker, index) => (
        <div
          key={`${index}-${marker.label}`}
          className="one-bar__timeline-hour"
          data-hour-marker-index={index}
          aria-hidden="true"
          style={{ left: `${marker.leftPct}%`, top: 10, height: 14 }}
        >
          {visibleHourLabels.has(index) ? (
            <span
              className={`one-bar__timeline-hour-label${index === 0 ? " one-bar__timeline-hour-label--start" : index === hourMarkers.length - 1 ? " one-bar__timeline-hour-label--end" : ""}`}
              style={{
                top: -10,
                left: index === 0 || index === hourMarkers.length - 1 ? 0 : "50%",
                transform: index === 0 ? "none" : index === hourMarkers.length - 1 ? "translateX(-100%)" : "translateX(-50%)",
                fontSize: 10,
                lineHeight: "10px",
              }}
            >
              {marker.label}
            </span>
          ) : null}
        </div>
      ))}
    </div>
  );
};
