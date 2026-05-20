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
import type { SnapshotRow } from "@/test/analyst-fixtures";
import { chartColors } from "./chart-utils";
import { TimelineDetailHeader } from "./TimelineDetailHeader";
import {
  DayDividerLayer,
  HourTickLayer,
  NowIndicator,
} from "./TimelineDetailLayers";
import {
  buildTimelineScale,
  dayCenterPx,
  easeInOut,
  msToPx,
  scrollDurationMs,
} from "./timeline-utils";
import {
  buildSegments,
  dayBoundaries,
  enumerateHourTicks,
} from "./timeline-segments";
import "./timeline-scrollbar.css";

const SIDE_PAD_HOURS = 6;
const STRIP_HEIGHT = 96;
const DAY_MS = 86_400_000;
const HOUR_MS = 3_600_000;
const PX_PER_MIN_TARGET = 1.4;

export interface TimelineDetailProps {
  range: { from: string; to: string };
  selectedDay: string;
  snapshots: SnapshotRow[];
  onPrev: () => void;
  onNext: () => void;
  /** Override "now" for tests. */
  nowMs?: number;
}

function rangeWidthPx(rangeFrom: string, rangeTo: string): number {
  const minutes =
    ((new Date(`${rangeTo}T00:00:00`).getTime() +
      DAY_MS +
      SIDE_PAD_HOURS * HOUR_MS -
      (new Date(`${rangeFrom}T00:00:00`).getTime() - SIDE_PAD_HOURS * HOUR_MS)) /
      60_000) |
    0;
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
  onPrev,
  onNext,
  nowMs,
}: TimelineDetailProps) {
  const scrollerRef = useRef<HTMLDivElement>(null);
  const animRef = useRef<number | null>(null);

  const widthPx = useMemo(() => rangeWidthPx(range.from, range.to), [range.from, range.to]);
  const scale = useMemo(
    () => buildTimelineScale(range.from, range.to, SIDE_PAD_HOURS, widthPx),
    [range.from, range.to, widthPx],
  );
  const segments = useMemo(() => buildSegments(snapshots, scale), [snapshots, scale]);
  const dayDividers = useMemo(() => dayBoundaries(scale), [scale]);
  const hourTicks = useMemo(() => enumerateHourTicks(scale), [scale]);

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

  const nowPx = useMemo(() => {
    return nowTick >= scale.startMs && nowTick <= scale.endMs
      ? msToPx(nowTick, scale)
      : null;
  }, [scale, nowTick]);

  // Smooth-scroll to selected day's centre whenever selectedDay changes.
  useEffect(() => {
    const el = scrollerRef.current;
    if (!el) return;
    const target = dayCenterPx(selectedDay, scale);
    const viewportCentre = el.clientWidth / 2;
    const targetLeft = Math.max(0, target - viewportCentre);
    const delta = targetLeft - el.scrollLeft;
    animateScroll(el, targetLeft, scrollDurationMs(delta), animRef);
    return () => {
      if (animRef.current !== null) cancelAnimationFrame(animRef.current);
    };
  }, [selectedDay, scale]);

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
          {segments.map((s, i) => (
            <rect
              key={i}
              x={s.x}
              y={20}
              width={s.w}
              height={STRIP_HEIGHT - 40}
              fill={s.fill}
              fillOpacity={0.92}
            />
          ))}
          <HourTickLayer ticks={hourTicks} scale={scale} stripHeight={STRIP_HEIGHT} />
          <DayDividerLayer dividers={dayDividers} scale={scale} stripHeight={STRIP_HEIGHT} />
          {nowPx !== null ? <NowIndicator x={nowPx} stripHeight={STRIP_HEIGHT} /> : null}
        </svg>
      </div>
    </div>
  );
}

