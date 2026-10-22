/**
 * TimelineDetailLayers.tsx — SVG layers rendered inside TimelineDetail.
 *
 * Split out of TimelineDetail.tsx to keep that file under the project's
 * 250-line cap. Each component receives pre-computed data (no math here);
 * the parent computes scale + hour ticks + day boundaries once and passes
 * them down.
 */
import { chartColors } from "./chart-utils";
import { timelineColors } from "@/timeline/colors";
import type { LocalHourTick } from "@/generated/LocalHourTick";
import { msToPx, type TimelineScale } from "./timeline-utils";

export interface HourTickLayerProps {
  ticks: LocalHourTick[];
  scale: TimelineScale;
  stripHeight: number;
}

export function HourTickLayer({ ticks, scale, stripHeight }: HourTickLayerProps) {
  const pxPerHour = scale.pxPerMinute * 60;
  const labelStep = pxPerHour >= 40 ? 2 : pxPerHour >= 20 ? 6 : 12;
  const counts = new Map<string, number>();
  const occupiedRows: number[] = [];
  const localKey = (tick: LocalHourTick) => {
    const localDate = new Date(Date.parse(tick.at_utc) + tick.utc_offset_secs * 1000).toISOString().slice(0, 10);
    return `${localDate}T${tick.wall_time}`;
  };
  ticks.forEach((tick) => counts.set(localKey(tick), (counts.get(localKey(tick)) ?? 0) + 1));
  const labelLayout = new Map<string, { anchor: "start" | "middle" | "end"; row: number }>();
  ticks.forEach((tick) => {
    const x = msToPx(Date.parse(tick.at_utc), scale);
    const hour = Number(tick.wall_time.slice(0, 2));
    const duplicated = (counts.get(localKey(tick)) ?? 0) > 1;
    if (!duplicated && hour % labelStep !== 0) return;
    const sign = tick.utc_offset_secs < 0 ? "−" : "+";
    const absoluteOffset = Math.abs(tick.utc_offset_secs);
    const offset = `${sign}${String(Math.floor(absoluteOffset / 3600)).padStart(2, "0")}:${String(Math.floor((absoluteOffset % 3600) / 60)).padStart(2, "0")}`;
    const label = `${tick.wall_time}${duplicated ? ` ${offset}` : ""}`;
    const width = label.length * 6;
    const anchor = duplicated
      ? scale.widthPx - x < width ? "end" : "start"
      : x < width / 2 ? "start" : scale.widthPx - x < width / 2 ? "end" : "middle";
    const left = anchor === "start" ? x : anchor === "end" ? x - width : x - width / 2;
    const right = left + width;
    const preferredRow = 0;
    const row = [preferredRow, 1 - preferredRow].find((candidate) =>
      occupiedRows[candidate] === undefined || occupiedRows[candidate] <= left - 1,
    ) ?? preferredRow;
    occupiedRows[row] = right;
    labelLayout.set(tick.at_utc, { anchor, row });
  });
  return (
    <g data-testid="hour-ticks">
      {ticks.map((tick) => {
        const ms = Date.parse(tick.at_utc);
        const x = msToPx(ms, scale);
        const duplicated = (counts.get(localKey(tick)) ?? 0) > 1;
        const sign = tick.utc_offset_secs < 0 ? "−" : "+";
        const absoluteOffset = Math.abs(tick.utc_offset_secs);
        const offset = `${sign}${String(Math.floor(absoluteOffset / 3600)).padStart(2, "0")}:${String(Math.floor((absoluteOffset % 3600) / 60)).padStart(2, "0")}`;
        const label = `${tick.wall_time}${duplicated ? ` ${offset}` : ""}`;
        const isLabel = labelLayout.has(tick.at_utc);
        const layout = labelLayout.get(tick.at_utc);
        return (
          <g key={tick.at_utc}>
            <line
              x1={x}
              x2={x}
              y1={stripHeight - 14}
              y2={stripHeight - 6}
              stroke={chartColors.gridline}
              strokeWidth={1}
            />
            {isLabel ? (
              <text
                x={x}
                y={stripHeight - 1 - (layout?.row ?? 0) * 12}
                fontSize={10}
                fill={chartColors.subtext}
                textAnchor={layout?.anchor ?? "middle"}
                data-at-utc={tick.at_utc}
                data-utc-offset-secs={tick.utc_offset_secs}
                data-label-row={layout?.row ?? 0}
                aria-label={label}
                fontFamily='var(--font-mono, ui-monospace, "Cascadia Code", "Consolas", monospace)'
              >
                {label}
              </text>
            ) : null}
          </g>
        );
      })}
    </g>
  );
}

const LEGEND_ITEMS = [
  { label: "Sitting", kind: "posture", color: timelineColors.Sitting },
  { label: "Standing", kind: "posture", color: timelineColors.Standing },
  { label: "Walking", kind: "posture", color: timelineColors.Walking },
  { label: "Away", kind: "posture", color: timelineColors.Away },
  { label: "Unknown", kind: "unknown", color: timelineColors.Unknown },
  { label: "Future", kind: "future", color: timelineColors.Future },
  { label: "Observation points", kind: "points", color: chartColors.subtext },
] as const;

export function TimelineLegend() {
  return (
    <ul
      data-testid="timeline-legend"
      aria-label="Timeline legend"
      style={{
        display: "flex",
        flexWrap: "wrap",
        alignItems: "center",
        gap: "4px 12px",
        listStyle: "none",
        margin: "6px 0",
        padding: 0,
        color: chartColors.subtext,
        fontSize: 11,
        lineHeight: 1.5,
      }}
    >
      {LEGEND_ITEMS.map(({ label, kind, color }) => (
        <li key={label} style={{ display: "inline-flex", alignItems: "center", gap: 5, whiteSpace: "nowrap" }}>
          <span
            aria-hidden="true"
            style={{
              display: "inline-block",
              boxSizing: "border-box",
              width: kind === "points" ? 2 : 11,
              height: kind === "points" ? 10 : 9,
              flex: "0 0 auto",
              background: kind === "unknown"
                ? `repeating-linear-gradient(135deg, transparent 0 4px, ${color} 4px 5px)`
                : kind === "future" || kind === "points" ? "transparent" : color,
              border: kind === "future" ? `1px solid ${color}` : undefined,
              borderRadius: kind === "posture" ? 1 : undefined,
            }}
          />
          {label}
        </li>
      ))}
    </ul>
  );
}

export interface DayDividerLayerProps {
  dividers: number[];
  scale: TimelineScale;
  stripHeight: number;
}

export function DayDividerLayer({ dividers, scale, stripHeight }: DayDividerLayerProps) {
  return (
    <>
      {dividers.map((t) => {
        const x = msToPx(t, scale);
        return (
          <g key={t}>
            <line
              x1={x}
              x2={x}
              y1={6}
              y2={stripHeight - 6}
              stroke={chartColors.gridline}
              strokeWidth={1}
            />
            <text x={x + 4} y={14} fontSize={9} fill={chartColors.subtext}>
              {new Date(t).toISOString().slice(5, 10)}
            </text>
          </g>
        );
      })}
    </>
  );
}

export interface NowIndicatorProps {
  x: number;
  stripHeight: number;
}

export function NowIndicator({ x, stripHeight }: NowIndicatorProps) {
  return (
    <g>
      <line
        x1={x}
        x2={x}
        y1={0}
        y2={stripHeight}
        stroke={chartColors.primary}
        strokeWidth={1.5}
      />
      <rect
        x={x - 18}
        y={stripHeight - 14}
        width={36}
        height={12}
        fill={chartColors.primary}
      />
      <text
        x={x}
        y={stripHeight - 5}
        fontSize={9}
        fill={chartColors.card}
        textAnchor="middle"
        fontWeight={600}
      >
        NOW
      </text>
    </g>
  );
}
