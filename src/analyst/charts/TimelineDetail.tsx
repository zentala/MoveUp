/**
 * TimelineDetail.tsx — One continuous horizontal timeline.
 *
 * Renders the entire active range as a single scrollable strip at uniform
 * pixels-per-minute. Clicking a day tab in `DateNavigator` updates
 * `selectedDay`; this component eases the scroll position so the selected
 * day's centre (12:00) lines up with the viewport centre.
 *
 * Visual:
 *   - Each minute = colored 1px segment by detected state.
 *   - Vertical dividers at every 00:00 day boundary.
 *   - "Now" indicator (blue line + label) at real time when in range.
 *   - Hour ticks every 4h beneath the strip.
 */
import { useEffect, useMemo, useRef, useState } from "react";
import type { LocalDayBounds } from "@/generated/LocalDayBounds";
import type { SnapshotRow } from "@/test/analyst-fixtures";
import { timelineColors } from "@/timeline/colors";
import type { TimelineInterval } from "@/timeline/intervals";
import { useLocalDayBoundsRange } from "../hooks/useLocalDayBoundsRange";
import { chartColors } from "./chart-utils";
import { TimelineDetailHeader } from "./TimelineDetailHeader";
import {
  DayDividerLayer,
  NowIndicator,
} from "./TimelineDetailLayers";
import {
  buildTimelineScaleForWindow,
  easeInOut,
  msToPx,
  scrollDurationMs,
} from "./timeline-utils";
import {
  buildSegments,
  buildObservationTicks,
} from "./timeline-segments";
import "./timeline-scrollbar.css";

const SIDE_PAD_HOURS = 6;
const STRIP_HEIGHT = 96;
const HOUR_MS = 3_600_000;
const PX_PER_MIN_TARGET = 1.4;

export interface TimelineDetailProps {
  range: { from: string; to: string };
  selectedDay: string;
  snapshots: SnapshotRow[];
  /** Only source-qualified interval endpoints may contribute elapsed color. */
  intervals?: readonly TimelineInterval[];
  onPrev: () => void;
  onNext: () => void;
  /** Override "now" for tests. */
  nowMs?: number;
}

function rangeWidthPx(bounds: LocalDayBounds[]): number {
  const startMs = Date.parse(bounds[0].start_utc);
  const endMs = Date.parse(bounds[bounds.length - 1].end_utc);
  const minutes = (endMs - startMs + SIDE_PAD_HOURS * HOUR_MS * 2) / 60_000;
  return Math.max(720, minutes * PX_PER_MIN_TARGET);
}


function animateScroll(
  el: HTMLElement,
  targetLeft: number,
  durationMs: number,
  cancelRef: { current: number | null },
): void {
  const startLeft = el.scrollLeft;
  const delta = targetLeft - startLeft;
  if (Math.abs(delta) < 1) {
    el.scrollLeft = targetLeft;
    return;
  }
  const startTime = performance.now();
  if (cancelRef.current !== null) cancelAnimationFrame(cancelRef.current);
  const tick = (now: number) => {
    const t = Math.min(1, (now - startTime) / durationMs);
    el.scrollLeft = startLeft + delta * easeInOut(t);
    if (t < 1) {
      cancelRef.current = requestAnimationFrame(tick);
    } else {
      cancelRef.current = null;
    }
  };
  cancelRef.current = requestAnimationFrame(tick);
}

export function TimelineDetail({
  range,
  selectedDay,
  snapshots,
  intervals = [],
  onPrev,
  onNext,
  nowMs,
}: TimelineDetailProps) {
  const scrollerRef = useRef<HTMLDivElement>(null);
  const animRef = useRef<number | null>(null);
  const boundsQuery = useLocalDayBoundsRange(range.from, range.to);
  const dayBounds = boundsQuery.status === "ready" ? boundsQuery.data : null;
  const widthPx = useMemo(() => dayBounds?.length ? rangeWidthPx(dayBounds) : 720, [dayBounds]);
  const scale = useMemo(
    () => {
      if (!dayBounds?.length) return null;
      const startMs = Date.parse(dayBounds[0].start_utc) - SIDE_PAD_HOURS * HOUR_MS;
      const endMs = Date.parse(dayBounds[dayBounds.length - 1].end_utc) + SIDE_PAD_HOURS * HOUR_MS;
      return buildTimelineScaleForWindow({ startMs, endMs }, widthPx);
    },
    [dayBounds, widthPx],
  );
  const timelineWindow = useMemo(
    () => scale && dayBounds?.length
      ? ({ startMs: Date.parse(dayBounds[0].start_utc), endMs: Date.parse(dayBounds[dayBounds.length - 1].end_utc) })
      : null,
    [scale, dayBounds],
  );
  const dayDividers = useMemo(
    () => dayBounds?.slice(1).map((day) => Date.parse(day.start_utc)) ?? [],
    [dayBounds],
  );

  // Tick state so the now-indicator updates every minute when nowMs is not
  // injected for tests. When nowMs is provided, we use it directly and skip
  // the interval entirely.
  const [nowTick, setNowTick] = useState<number>(() => nowMs ?? Date.now());
  useEffect(() => {
    if (nowMs !== undefined) {
      setNowTick(nowMs);
      return;
    }
    const id = window.setInterval(() => setNowTick(Date.now()), 60_000);
    return () => window.clearInterval(id);
  }, [nowMs]);

  const chronology = useMemo(
    () => scale && timelineWindow
      ? buildSegments(intervals, timelineWindow, scale, nowTick)
      : { intervals: [], gaps: [] },
    [intervals, timelineWindow, scale, nowTick],
  );
  const observationTicks = useMemo(
    () => scale && timelineWindow ? buildObservationTicks(snapshots, scale, timelineWindow, nowTick) : [],
    [snapshots, scale, timelineWindow, nowTick],
  );
  const [selectedDetail, setSelectedDetail] = useState("");
  const selectedObservation = selectedDetail.startsWith("observation-")
    ? observationTicks.find((item) => selectedDetail === `observation-${item.atMs}`)
    : undefined;
  const selectedInterval = selectedDetail.startsWith("interval-")
    ? chronology.intervals.find((item) => selectedDetail === `interval-${item.startMs}-${item.endMs}`)
    : undefined;
  const selectedGap = selectedDetail.startsWith("gap-")
    ? chronology.gaps.find((item) => selectedDetail === `gap-${item.kind}-${item.startMs}`)
    : undefined;

  const nowPx = useMemo(() => {
    return scale && nowTick >= scale.startMs && nowTick <= scale.endMs
      ? msToPx(nowTick, scale)
      : null;
  }, [scale, nowTick]);

  // Smooth-scroll to selected day's centre whenever selectedDay changes.
  useEffect(() => {
    const el = scrollerRef.current;
    const selected = dayBounds?.find((day) => day.date_local === selectedDay);
    if (!el || !scale || !selected) return;
    const center = (Date.parse(selected.start_utc) + Date.parse(selected.end_utc)) / 2;
    const target = msToPx(center, scale);
    const viewportCentre = el.clientWidth / 2;
    const targetLeft = Math.max(0, target - viewportCentre);
    const delta = targetLeft - el.scrollLeft;
    animateScroll(el, targetLeft, scrollDurationMs(delta), animRef);
    return () => {
      if (animRef.current !== null) cancelAnimationFrame(animRef.current);
    };
  }, [selectedDay, scale, dayBounds]);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "ArrowLeft") {
      e.preventDefault();
      onPrev();
    } else if (e.key === "ArrowRight") {
      e.preventDefault();
      onNext();
    }
  };

  return (
    <div
      data-testid="timeline-detail"
      tabIndex={0}
      onKeyDown={handleKeyDown}
      role="region"
      aria-label={`Daily timeline for ${selectedDay}`}
      style={{ outline: "none" }}
    >
      <TimelineDetailHeader selectedDay={selectedDay} onPrev={onPrev} onNext={onNext} />
      {scale ? (
      <div
        ref={scrollerRef}
        data-testid="timeline-scroller"
        className="timeline-scroller"
        style={{
          overflowX: "auto",
          border: `1px solid ${chartColors.gridline}`,
          background: chartColors.background,
          borderRadius: 6,
          position: "relative",
        }}
      >
        <svg
          width={widthPx}
          height={STRIP_HEIGHT}
          role="img"
          aria-label={`Daily timeline ${range.from} to ${range.to}`}
        >
          <rect width={widthPx} height={STRIP_HEIGHT} fill={chartColors.card} />
          <defs>
            <pattern id="timeline-unknown-hatch" width="5" height="5" patternUnits="userSpaceOnUse" patternTransform="rotate(135)">
              <line x1="0" y1="0" x2="0" y2="5" stroke={timelineColors.Unknown} strokeWidth="1" />
            </pattern>
          </defs>
          {chronology.gaps.map((gap) => (
            <rect
              key={`gap-${gap.kind}-${gap.startMs}`}
              data-testid={`timeline-${gap.kind.toLowerCase()}`}
              x={gap.x}
              y={20}
              width={gap.width}
              height={STRIP_HEIGHT - 40}
              fill={gap.kind === "Unknown" ? "url(#timeline-unknown-hatch)" : timelineColors.Future}
              fillOpacity={gap.kind === "Future" ? 0.25 : 1}
              stroke={gap.kind === "Future" ? timelineColors.Future : undefined}
            />
          ))}
          {chronology.intervals.map((s, i) => (
            <rect
              key={i}
              data-testid="timeline-segment"
              data-state={s.state}
              data-start-ms={s.startMs}
              data-end-ms={s.endMs}
              x={s.x}
              y={20}
              width={s.width}
              height={STRIP_HEIGHT - 40}
              fill={timelineColors[s.state]}
              fillOpacity={0.92}
            />
          ))}
          {observationTicks.map((tick) => (
            <rect
              key={`observation-${tick.atMs}`}
              data-testid="timeline-observation-marker"
              data-at-ms={tick.atMs}
              x={tick.x}
              y={0}
              width={2}
              height={6}
              fill={tick.color}
              aria-hidden="true"
            />
          ))}
          <DayDividerLayer dividers={dayDividers} scale={scale} stripHeight={STRIP_HEIGHT} />
          {nowPx !== null ? <NowIndicator x={nowPx} stripHeight={STRIP_HEIGHT} /> : null}
        </svg>
      </div>
      ) : (
        <p role={boundsQuery.status === "error" ? "alert" : "status"}>
          {boundsQuery.status === "loading"
            ? "Loading query-local calendar bounds…"
            : "Calendar bounds unavailable; elapsed coverage remains Unknown."}
        </p>
      )}
      <p style={{ margin: "6px 0", color: chartColors.subtext, fontSize: 11 }}>
        Calendar basis: current query-local timezone; historical capture timezone unknown. Empty dates do not prove observation coverage. Observation ticks mark instants; duration unknown. Filled spans require recorded session endpoints.
      </p>
      <label style={{ display: "block", color: chartColors.subtext, fontSize: 11 }}>
        Timeline details
        <select
          aria-label="Timeline details"
          value={selectedDetail}
          onChange={(event) => setSelectedDetail(event.currentTarget.value)}
          style={{ display: "block", maxWidth: "100%", minHeight: 28, font: "inherit" }}
        >
          <option value="">Select an interval or observation</option>
          {chronology.intervals.map((item) => (
            <option key={`interval-${item.startMs}-${item.endMs}`} value={`interval-${item.startMs}-${item.endMs}`}>
              {`${item.state}: ${new Date(item.startMs).toISOString()} – ${new Date(item.endMs).toISOString()} · ${((item.endMs - item.startMs) / 1000).toFixed(3)}s · source: ${item.provenance ?? "unknown"}`}
            </option>
          ))}
          {chronology.gaps.map((item) => (
            <option key={`gap-${item.kind}-${item.startMs}`} value={`gap-${item.kind}-${item.startMs}`}>
              {`${item.kind}: ${new Date(item.startMs).toISOString()} – ${new Date(item.endMs).toISOString()}`}
            </option>
          ))}
          {observationTicks.map((item) => (
            <option key={`observation-${item.atMs}`} value={`observation-${item.atMs}`}>
              {`Observation instant ${new Date(item.atMs).toISOString()} · ${item.state ?? "Unknown"} · quality: ${item.observationQuality ?? "unknown"} · source local date: ${item.localDate ?? "unknown"} · UTC offset: ${item.timezoneOffsetSecs == null ? "unknown" : `${item.timezoneOffsetSecs}s`} · duration unknown`}
            </option>
          ))}
        </select>
      </label>
      {selectedDetail ? (
        <div role="status" data-testid={selectedObservation ? "timeline-observation-detail" : "timeline-interval-detail"}
          style={{ color: chartColors.subtext, fontSize: 11 }}>
          {selectedObservation
            ? `Observation instant ${new Date(selectedObservation.atMs).toISOString()} · ${selectedObservation.state ?? "Unknown"} · quality: ${selectedObservation.observationQuality ?? "unknown"} · ${selectedObservation.connected === false ? "disconnected" : selectedObservation.connected === true ? "connected" : "connectivity unknown"} · source local date: ${selectedObservation.localDate ?? "unknown"} · UTC offset: ${selectedObservation.timezoneOffsetSecs == null ? "unknown" : `${selectedObservation.timezoneOffsetSecs}s`} · duration unknown`
            : selectedInterval
              ? `${selectedInterval.state}: ${new Date(selectedInterval.startMs).toISOString()} – ${new Date(selectedInterval.endMs).toISOString()} · ${((selectedInterval.endMs - selectedInterval.startMs) / 1000).toFixed(3)}s · source: ${selectedInterval.provenance ?? "unknown"}`
              : selectedGap
                ? `${selectedGap.kind}: ${new Date(selectedGap.startMs).toISOString()} – ${new Date(selectedGap.endMs).toISOString()} · ${((selectedGap.endMs - selectedGap.startMs) / 1000).toFixed(3)}s`
                : ""}
        </div>
      ) : null}
    </div>
  );
}

