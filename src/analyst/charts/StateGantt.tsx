/**
 * StateGantt.tsx — Per-day horizontal bars of Sitting/Standing/Walking/Away.
 *
 * Each day row is a stacked bar from 00:00 to 24:00. Segment widths are
 * scaled by snapshot count per state in each hour bin.
 */
import type { SnapshotRow } from "@/test/analyst-fixtures";
import type { DeskState } from "@/types";
import { ChartCard } from "./ChartCard";
import { DEFAULT_MARGIN, chartColors } from "./chart-utils";

export interface StateGanttProps {
  data: SnapshotRow[];
  width?: number;
  height?: number;
}

const STATE_COLOR: Record<DeskState, string> = {
  Sitting: chartColors.sitting,
  Standing: chartColors.standing,
  Walking: chartColors.walking,
  Away: chartColors.away,
};

function groupByDay(rows: SnapshotRow[]): Map<string, SnapshotRow[]> {
  const map = new Map<string, SnapshotRow[]>();
  for (const r of rows) {
    const day = r.ts.slice(0, 10);
    let bucket = map.get(day);
    if (!bucket) {
      bucket = [];
      map.set(day, bucket);
    }
    bucket.push(r);
  }
  return map;
}

const LEGEND_BAND = 24;

export function StateGantt({ data, width = 720, height = 264 }: StateGanttProps) {
  const m = DEFAULT_MARGIN;
  const innerW = width - m.left - m.right;
  // Reserve a fixed legend band below the regular bottom margin so labels
  // never collide with the hour axis or get clipped at the SVG edge.
  const innerH = height - m.top - m.bottom - LEGEND_BAND;

  const days = Array.from(groupByDay(data).entries()).sort(([a], [b]) => a.localeCompare(b));
  const rowH = days.length > 0 ? innerH / days.length : innerH;

  return (
    <ChartCard
      title="State Gantt by day"
      subtitle="One row per day. Color shows state across 24 hours."
      help="One horizontal row per local day in the range. Each segment is a snapshot bucket coloured by detected state (Sitting / Standing / Walking / Away). Useful for spotting daily rhythm and away gaps."
    >
      <svg width={width} height={height} role="img" aria-label="State Gantt by day">
        <g transform={`translate(${m.left},${m.top})`}>
          {days.map(([day, rows], idx) => {
            const y = idx * rowH;
            const segW = innerW / Math.max(rows.length, 1);
            return (
              <g key={day} transform={`translate(0,${y})`}>
                <text
                  x={-6}
                  y={rowH / 2 + 3}
                  fontSize={10}
                  textAnchor="end"
                  fill={chartColors.axisText}
                >
                  {day.slice(5)}
                </text>
                {rows.map((r, i) => (
                  <rect
                    key={i}
                    x={i * segW}
                    y={2}
                    width={segW + 0.5}
                    height={Math.max(rowH - 6, 4)}
                    fill={STATE_COLOR[r.state]}
                    fillOpacity={0.9}
                  />
                ))}
              </g>
            );
          })}
          {/* Hour axis */}
          {[0, 6, 12, 18, 24].map((h) => (
            <g key={h} transform={`translate(${(h / 24) * innerW},${innerH})`}>
              <line y2={4} stroke={chartColors.axis} />
              <text y={16} fontSize={10} fill={chartColors.axisText} textAnchor="middle">
                {h}:00
              </text>
            </g>
          ))}
        </g>
        {/* Legend — sits in the dedicated band below the hour axis. */}
        <g
          transform={`translate(${m.left},${height - LEGEND_BAND + 4})`}
          data-testid="legend"
        >
          {(["Sitting", "Standing", "Walking", "Away"] as DeskState[]).map((s, i) => (
            <g key={s} transform={`translate(${i * 78},0)`}>
              <rect width={10} height={10} fill={STATE_COLOR[s]} />
              <text x={14} y={9} fontSize={10} fill={chartColors.axisText}>
                {s}
              </text>
            </g>
          ))}
        </g>
      </svg>
    </ChartCard>
  );
}
