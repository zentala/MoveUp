/** The compact view of the same bounded chronology used by Analyst. */
import { type FC, useEffect, useReducer, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { SessionEntry, WidgetProps } from "@/types";
import { computeHourMarkers } from "@/utils/timeline";
import { TimelineStrip } from "@/timeline/TimelineStrip";
import { mergeTimelineIntervals, retainLiveObservationIntervals, sessionIntervals } from "@/timeline/intervals";
import { createOpenDayRequest, localCalendarDay, OPEN_ANALYST_DAY_KEY } from "@/analyst/open-day";
import type { HourMarker } from "@/utils/timeline";
import type { LiveObservation } from "@/generated/LiveObservation";
import type { QueryStatus } from "@/hooks/useDeskTypes";

let openRequestSequence = 0;

const HOUR_LABEL_GAP_PX = 8;

interface OneBarTimelineProps extends WidgetProps {
  liveObservation?: LiveObservation | null;
  dashboardRefresh?: QueryStatus;
}

function timeLabel(value: string | null | undefined): string | null {
  if (!value) return null;
  const time = Date.parse(value);
  return Number.isFinite(time) ? new Date(time).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }) : null;
}

function viewStatus(refresh?: QueryStatus): string {
  const received = timeLabel(refresh?.lastSuccessAt);
  if (refresh?.error) return `View update failed: ${refresh.error}${received ? ` · last response received ${received}` : ""}`;
  if (refresh?.loading) return "View updating";
  return received ? `View response received ${received}` : "View response not yet received";
}

export function formatObservationViewDetails(refresh?: QueryStatus): string {
  return viewStatus(refresh);
}

export function formatObservationStatus(observation: LiveObservation | null | undefined, refresh?: QueryStatus): string {
  const span = observation?.span;
  const through = timeLabel(span?.observed_through);
  const observationText = span && through
    ? `${span.state} observed through ${through}${observation?.reason === "stale" ? " · sensor observation stale" : ""}${observation?.reason === "pending_confirmation" ? " · next posture pending confirmation" : ""}`
    : observation?.quality === "known" && timeLabel(observation.last_observation_at)
      ? `Instant observation at ${timeLabel(observation.last_observation_at)} · duration unknown`
      : observation?.reason === "no_observation" ? "No sensor observations yet"
      : observation?.reason === "clock_rollback" ? "Clock moved backward · current observation unavailable"
        : observation?.reason === "pending_confirmation" ? "Next posture pending confirmation"
          : "Current observation unavailable";
  return `Device connection status unknown · ${observationText} · ${viewStatus(refresh)}`;
}

export function formatObservationAnnouncement(observation: LiveObservation | null | undefined, refresh?: QueryStatus): string {
  const span = observation?.span;
  const health = observation?.quality === "known" && span
    ? `${span.state} observation available`
    : observation?.reason === "stale" ? "Sensor observation stale"
      : observation?.reason === "pending_confirmation" ? "Next posture pending confirmation"
        : observation?.reason === "clock_rollback" ? "Clock moved backward; observation unavailable"
          : observation?.reason === "no_observation" ? "No sensor observations yet"
            : "Current observation unavailable";
  return refresh?.error ? `${health} · View update failed: ${refresh.error}` : health;
}

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

export const OneBarTimeline: FC<OneBarTimelineProps> = (props) => {
  const [currentNowMs, setNowMs] = useReducer((_: number, now: number) => now, undefined, () => Date.now());
  const [axisWidthPx, setAxisWidthPx] = useState(0);
  const [retainedObservations, rememberObservation] = useReducer(
    (retained: LiveObservation[], action: { observation: LiveObservation | null | undefined; sessions: SessionEntry[] }) => {
      const next = retainLiveObservationIntervals(retained, action.observation, action.sessions);
      return next.length === retained.length && next.every((observation, index) => observation === retained[index])
        ? retained : next;
    },
    [],
  );
  const timelineRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const timer = globalThis.setInterval(() => setNowMs(Date.now()), 60_000);
    return () => globalThis.clearInterval(timer);
  }, []);
  useEffect(() => {
    setNowMs(Date.now());
  }, [props.dashboardRefresh?.loading, props.dashboardRefresh?.lastSuccessAt,
    props.dashboardRefresh?.error, props.liveObservation?.snapshot_at]);
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
  const retained = retainLiveObservationIntervals(retainedObservations, props.liveObservation, sessions);
  useEffect(() => {
    rememberObservation({ observation: props.liveObservation, sessions });
  }, [props.liveObservation, sessions]);
  const starts = [
    ...sessions.map((row) => Date.parse(row.start)),
    ...retained.flatMap((observation) => observation.span ? [Date.parse(observation.span.started_at)] : []),
    ...(props.liveObservation?.span ? [Date.parse(props.liveObservation.span.started_at)] : []),
  ].filter((value) => Number.isFinite(value) && value <= currentNowMs);
  const midnight = new Date(currentNowMs);
  midnight.setHours(0, 0, 0, 0);
  const startMs = starts.length ? Math.max(Math.min(...starts), midnight.getTime()) : currentNowMs;
  const window = { startMs, endMs: currentNowMs };
  let intervals = sessionIntervals(sessions);
  let liveError: string | null = null;
  try {
    intervals = mergeTimelineIntervals(sessions, props.liveObservation, window, currentNowMs,
      retained);
  } catch (error) {
    liveError = error instanceof Error ? error.message : "Invalid live observation";
  }
  const openToday = () => openAnalyst(localCalendarDay(new Date()));

  if (startMs >= currentNowMs) {
    return (
      <div
        ref={timelineRef}
        className="one-bar__timeline"
        data-testid="one-bar-timeline"
        onClick={openToday}
        role="button"
        tabIndex={0}
        aria-label="Open Analyst for today's local day"
        title={liveError ? `Current observation unavailable: ${liveError}. Click to open Analyst — full timeline view` : "Click to open Analyst — full timeline view"}
        aria-description={liveError ? `Current observation unavailable: ${liveError}` : undefined}
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

  const hourMarkers = startMs < currentNowMs ? computeHourMarkers(window) : [];
  const visibleHourLabels = new Set(selectReadableHourMarkerIndices(hourMarkers, axisWidthPx));
  return (
    <div
      ref={timelineRef}
      className="one-bar__timeline"
      data-testid="one-bar-timeline"
      role="group"
      aria-label="Compact timeline. Focus and press Enter to open Analyst for today."
      tabIndex={0}
      title={liveError ? `Current observation unavailable: ${liveError}. Click the timeline background or press Enter to open Analyst for today` : "Click the timeline background or press Enter to open Analyst for today"}
      aria-description={liveError ? `Current observation unavailable: ${liveError}` : undefined}
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
        nowMs={currentNowMs}
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
