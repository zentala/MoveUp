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
import { useEffect, useMemo, useRef } from "react";
import type { SnapshotRow } from "@/test/analyst-fixtures";
import { chartColors } from "./chart-utils";
import { TimelineDetailHeader } from "./TimelineDetailHeader";
import {
  buildTimelineScale,
  dayCenterPx,
  easeInOut,
  inferSnapshotInterval,
  msToPx,
  scrollDurationMs,
  type TimelineScale,
} from "./timeline-utils";

const SIDE_PAD_HOURS = 6;
const STRIP_HEIGHT = 96;
const DAY_MS = 86_400_000;
const HOUR_MS = 3_600_000;
const PX_PER_MIN_TARGET = 1.4;

const STATE_FILL: Record<string, string> = {
  Sitting: chartColors.sitting,
  Standing: chartColors.standing,
  Walking: chartColors.walking,
  Away: chartColors.away,
};

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

interface RenderedSegment {
  x: number;
  w: number;
  fill: string;
}

function buildSegments(
  snapshots: SnapshotRow[],
  scale: TimelineScale,
): RenderedSegment[] {
  if (snapshots.length === 0) return [];
  // Each snapshot represents observed state from its timestamp until the next
  // snapshot (or until median interval after if it's the last row). This
  // produces a continuous strip even when fixtures sample sparsely.
  const intervalMs = inferSnapshotInterval(snapshots) * 1000;
  const out: RenderedSegment[] = [];
  for (let i = 0; i < snapshots.length; i++) {
    const row = snapshots[i];
    const t = new Date(row.ts).getTime();
    const tNext =
      i + 1 < snapshots.length ? new Date(snapshots[i + 1].ts).getTime() : t + intervalMs;
    // Clip to scale range.
    const startMs = Math.max(t, scale.startMs);
    const endMs = Math.min(tNext, scale.endMs);
    if (endMs <= startMs) continue;
    const x = msToPx(startMs, scale);
    const w = Math.max(1, msToPx(endMs, scale) - x);
    out.push({ x, w, fill: STATE_FILL[row.state] ?? chartColors.away });
  }
  return out;
}

function* dayBoundaries(scale: TimelineScale): Generator<number> {
  const startDay = new Date(scale.startMs);
  startDay.setHours(0, 0, 0, 0);
  for (let t = startDay.getTime(); t <= scale.endMs; t += DAY_MS) {
    if (t >= scale.startMs) yield t;
  }
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
  const dayDividers = useMemo(() => Array.from(dayBoundaries(scale)), [scale]);

  const nowPx = useMemo(() => {
    const t = nowMs ?? Date.now();
    return t >= scale.startMs && t <= scale.endMs ? msToPx(t, scale) : null;
  }, [scale, nowMs]);

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

  return (
    <div data-testid="timeline-detail">
      <TimelineDetailHeader selectedDay={selectedDay} onPrev={onPrev} onNext={onNext} />
      <div
        ref={scrollerRef}
        data-testid="timeline-scroller"
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
          {dayDividers.map((t) => {
            const x = msToPx(t, scale);
            return (
              <g key={t}>
                <line
                  x1={x}
                  x2={x}
                  y1={6}
                  y2={STRIP_HEIGHT - 6}
                  stroke={chartColors.gridline}
                  strokeWidth={1}
                />
                <text x={x + 4} y={14} fontSize={9} fill={chartColors.subtext}>
                  {new Date(t).toISOString().slice(5, 10)}
                </text>
              </g>
            );
          })}
          {nowPx !== null ? (
            <g>
              <line
                x1={nowPx}
                x2={nowPx}
                y1={0}
                y2={STRIP_HEIGHT}
                stroke={chartColors.primary}
                strokeWidth={1.5}
              />
              <rect x={nowPx - 18} y={STRIP_HEIGHT - 14} width={36} height={12} fill={chartColors.primary} />
              <text
                x={nowPx}
                y={STRIP_HEIGHT - 5}
                fontSize={9}
                fill={chartColors.card}
                textAnchor="middle"
                fontWeight={600}
              >
                NOW
              </text>
            </g>
          ) : null}
        </svg>
      </div>
    </div>
  );
}

