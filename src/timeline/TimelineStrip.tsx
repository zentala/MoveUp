import { useEffect, useState } from "react";
import { timelineColors } from "./colors";
import {
  intervalPosition,
  normalizeIntervals,
  timelineGaps,
  type TimelineGap,
  type TimelineInterval,
  type TimelineObservationMarker,
  type TimelineWindow,
} from "./intervals";

function timestamp(ms: number): string {
  return new Date(ms).toISOString();
}

function durationLabel(startMs: number, endMs: number): string {
  const seconds = (endMs - startMs) / 1000;
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const remainder = Number((seconds % 60).toFixed(3));
  return `${hours ? `${hours}h ` : ""}${minutes ? `${minutes}m ` : ""}${remainder}s`;
}

export function intervalTooltip(interval: TimelineInterval | TimelineGap): string {
  const label = "kind" in interval ? interval.kind : interval.state;
  const source = "provenance" in interval && interval.provenance ? ` · source: ${interval.provenance}` : "";
  return `${label}: ${timestamp(interval.startMs)} – ${timestamp(interval.endMs)} · ` +
    `${durationLabel(interval.startMs, interval.endMs)}${source}`;
}

function markerTooltip(marker: TimelineObservationMarker): string {
  const state = marker.state ?? "Unknown";
  const local = marker.localDate ? ` · source local date: ${marker.localDate}` : "";
  const offset = marker.timezoneOffsetSecs == null ? "" : ` · UTC offset: ${marker.timezoneOffsetSecs}s`;
  const connection = marker.connected === false ? "disconnected"
    : marker.connected === true ? "connected" : "connectivity unknown";
  return `Observation instant ${timestamp(marker.atMs)} · ${state} · quality: ` +
    `${marker.observationQuality ?? "unknown"} · ${connection}${local}${offset} · duration unknown`;
}

export interface TimelineStripProps {
  intervals: readonly TimelineInterval[];
  markers?: readonly TimelineObservationMarker[];
  window: TimelineWindow;
  mode?: "compact" | "detail";
  label?: string;
  nowMs?: number;
}

/** Absolute timestamp geometry keeps Unknown gaps and actual endpoints visible. */
export function TimelineStrip({
  intervals,
  markers = [],
  window,
  mode = "detail",
  label = "Chronological desk states; unobserved time is unknown; point ticks are observation instants, duration unknown",
  nowMs,
}: TimelineStripProps) {
  const [clockNowMs, setClockNowMs] = useState(Date.now);
  useEffect(() => {
    if (nowMs !== undefined) return;
    const timer = globalThis.setInterval(() => setClockNowMs(Date.now()), 60_000);
    return () => globalThis.clearInterval(timer);
  }, [nowMs]);
  const currentNowMs = nowMs ?? clockNowMs;
  const [selected, setSelected] = useState<string | null>(null);
  let segments: TimelineInterval[];
  let gaps: TimelineGap[];
  try {
    segments = normalizeIntervals(intervals, {
      startMs: window.startMs,
      endMs: Math.min(window.endMs, currentNowMs),
    });
    gaps = timelineGaps(segments, window, currentNowMs);
  } catch (error) {
    return <div role="alert">Timeline data error: {error instanceof Error ? error.message : "Invalid intervals"}</div>;
  }

  const selectedItem = [...segments, ...gaps].find((item) => {
    const key = "kind" in item
      ? `gap:${item.kind}:${item.startMs}:${item.endMs}`
      : `interval:${item.startMs}:${item.endMs}`;
    return key === selected;
  });
  const selectedMarker = markers.find((marker) => `observation:${marker.atMs}` === selected);
  const sorted = [...segments, ...gaps].sort((a, b) => a.startMs - b.startMs);
  const compact = mode === "compact";

  return (
    <div
      data-testid="timeline-strip-root"
      style={{
        position: "relative",
        minWidth: 0,
        flex: compact ? 1 : undefined,
        fontFamily: 'var(--font-mono, ui-monospace, "Cascadia Code", "Consolas", monospace)',
      }}
    >
      <div
        data-testid="timeline-strip"
        role="group"
        aria-label={`${label}; ${markers.length} point observations`}
        data-start-ms={window.startMs}
        data-end-ms={window.endMs}
        className={compact ? "one-bar__timeline-bar" : undefined}
        style={{
          position: "relative",
          display: "block",
          width: "100%",
          height: compact ? 24 : 56,
          minHeight: compact ? 24 : 56,
          boxSizing: "border-box",
          overflow: "hidden",
          background: "var(--panel-raised, #262626)",
          border: "1px solid var(--rail-emphasis, #6f6f6f)",
          borderRadius: 3,
        }}
      >
        {gaps.map((gap) => {
          const position = intervalPosition(gap, window);
          return (
            <div
              key={`${gap.kind}-${gap.startMs}`}
              data-testid={`timeline-${gap.kind.toLowerCase()}`}
              aria-label={intervalTooltip(gap)}
              title={intervalTooltip(gap)}
              style={{
                position: "absolute",
                insetBlock: 0,
                left: `${position.leftPct}%`,
                width: `${position.widthPct}%`,
                boxSizing: "border-box",
                background: gap.kind === "Unknown"
                  ? `repeating-linear-gradient(135deg, transparent 0 4px, ${timelineColors.Unknown} 4px 5px)`
                  : "var(--panel-base, #161616)",
                outline: gap.kind === "Future" ? `1px solid ${timelineColors.Future}` : undefined,
                outlineOffset: -1,
              }}
            />
          );
        })}
        {markers.filter((marker) => marker.atMs >= window.startMs && marker.atMs < window.endMs).map((marker) => {
          const left = ((marker.atMs - window.startMs) / (window.endMs - window.startMs)) * 100;
          return (
            <span
              key={`observation-${marker.atMs}`}
              data-testid="timeline-observation-marker"
              data-at-ms={marker.atMs}
              title={markerTooltip(marker)}
              aria-hidden="true"
              style={{
                position: "absolute",
                zIndex: 2,
                top: 0,
                left: `${left}%`,
                width: 2,
                height: 6,
                background: marker.state ? timelineColors[marker.state] : "var(--ink-secondary, #c6c6c6)",
                pointerEvents: "none",
              }}
            />
          );
        })}
        {segments.map((interval) => {
          const position = intervalPosition(interval, window);
          const key = `interval:${interval.startMs}:${interval.endMs}`;
          const text = intervalTooltip(interval);
          const selectedNow = selected === key;
          return (
            <button
              key={`${key}:${interval.state}`}
              type="button"
              data-timeline-item
              data-testid={interval.current ? "timeline-live-block" : "timeline-segment"}
              data-state={interval.state}
              className={compact ? `one-bar__timeline-block one-bar__timeline-block--${interval.state.toLowerCase()}` : undefined}
              data-start-ms={interval.startMs}
              data-end-ms={interval.endMs}
              aria-label={text}
              aria-pressed={selectedNow}
              title={text}
              onClick={() => setSelected(selectedNow ? null : key)}
              onFocus={() => setSelected(key)}
              onKeyDown={(event) => {
                const index = segments.findIndex((item) => `interval:${item.startMs}:${item.endMs}` === key);
                const nextIndex = event.key === "Home" ? 0
                  : event.key === "End" ? segments.length - 1
                    : event.key === "ArrowRight" ? Math.min(index + 1, segments.length - 1)
                      : event.key === "ArrowLeft" ? Math.max(index - 1, 0) : -1;
                if (event.key === "Escape") {
                  setSelected(null);
                  event.currentTarget.blur();
                  return;
                }
                if (nextIndex < 0) return;
                event.preventDefault();
                const next = segments[nextIndex];
                if (!next) return;
                const selector = `[data-timeline-item][data-start-ms="${next.startMs}"][data-end-ms="${next.endMs}"]`;
                event.currentTarget.parentElement?.querySelector<HTMLButtonElement>(selector)?.focus();
                const nextKey = `interval:${next.startMs}:${next.endMs}`;
                setSelected(nextKey);
              }}
              style={{
                position: "absolute",
                top: 0,
                bottom: 0,
                left: `${position.leftPct}%`,
                width: `${position.widthPct}%`,
                minWidth: 0,
                padding: 0,
                border: 0,
                borderRadius: 0,
                outline: selectedNow ? "2px solid var(--rail-active, #78a9ff)" : undefined,
                outlineOffset: -2,
                background: timelineColors[interval.state],
                boxSizing: "border-box",
                cursor: "pointer",
                zIndex: selectedNow ? 2 : 1,
              }}
            >
              <span aria-hidden="true" data-testid="timeline-hit-target" style={{
                position: "absolute",
                insetBlock: 0,
                left: "50%",
                width: 24,
                minHeight: 24,
                transform: "translateX(-50%)",
              }} />
            </button>
          );
        })}
      </div>
      {!compact && (
        <label style={{ display: "block", marginTop: 8, color: "var(--ink-secondary, #c6c6c6)", fontSize: 11 }}>
          Interval details — observation ticks mark instants; duration unknown
          <select
            aria-label="Interval details"
            value={selected ?? ""}
            onChange={(event) => setSelected(event.currentTarget.value || null)}
            style={{ display: "block", maxWidth: "100%", minHeight: 28, font: "inherit" }}
          >
            <option value="">Select an interval or observation</option>
            {sorted.map((item) => {
              const key = "kind" in item
                ? `gap:${item.kind}:${item.startMs}:${item.endMs}`
                : `interval:${item.startMs}:${item.endMs}`;
              return (
              <option key={key} value={key}>
                {intervalTooltip(item)}
              </option>
              );
            })}
            {markers.map((marker) => (
              <option key={`observation:${marker.atMs}`} value={`observation:${marker.atMs}`}>
                {markerTooltip(marker)}
              </option>
            ))}
          </select>
        </label>
      )}
      {(selectedItem || selectedMarker) && (
        <div
          role="tooltip"
          data-testid="timeline-interval-detail"
          style={{
            position: "absolute",
            zIndex: 3,
            left: 0,
            bottom: "100%",
            maxWidth: "100%",
            padding: "3px 8px",
            background: "var(--panel-overlay, #393939)",
            color: "var(--ink-primary, #f4f4f4)",
            border: "1px solid var(--panel-elevated, #303030)",
            fontSize: 10,
          }}
        >
          {selectedItem ? intervalTooltip(selectedItem) : markerTooltip(selectedMarker!)}
        </div>
      )}
    </div>
  );
}
