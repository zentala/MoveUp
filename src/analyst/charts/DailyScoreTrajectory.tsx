/**
 * DailyScoreTrajectory.tsx — Line per day showing score over hour-of-day.
 *
 * Computes max score per hour bucket per day, then renders one polyline
 * per day plus a thicker average line.
 */
import type { SnapshotRow } from "@/test/analyst-fixtures";
import { ChartCard } from "./ChartCard";
import { DEFAULT_MARGIN, chartColors, scaleLinear } from "./chart-utils";

export interface DailyScoreTrajectoryProps {
  data: SnapshotRow[];
  width?: number;
  height?: number;
}

function hourlyScores(rows: SnapshotRow[]): Map<string, number[]> {
  // day -> hour buckets [0..23], max score in that hour
  const map = new Map<string, number[]>();
  for (const r of rows) {
    const day = r.ts.slice(0, 10);
    const hour = new Date(r.ts).getUTCHours();
    let arr = map.get(day);
    if (!arr) {
      arr = Array(24).fill(0);
      map.set(day, arr);
    }
    if (r.score > arr[hour]) arr[hour] = r.score;
  }
  return map;
}

export function DailyScoreTrajectory({
  data,
  width = 720,
  height = 220,
}: DailyScoreTrajectoryProps) {
  const m = DEFAULT_MARGIN;
  const innerW = width - m.left - m.right;
  const innerH = height - m.top - m.bottom;

  const days = Array.from(hourlyScores(data).entries()).sort(([a], [b]) => a.localeCompare(b));
  const maxScore = Math.max(1, ...days.flatMap(([, v]) => v));

  const x = scaleLinear(0, 23, 0, innerW);
  const y = scaleLinear(0, maxScore, innerH, 0);

  return (
    <ChartCard
      title="Daily score trajectory"
      subtitle="Posture score by hour of day. One line per day."
    >
      <svg width={width} height={height} role="img" aria-label="Daily score trajectory">
        <g transform={`translate(${m.left},${m.top})`}>
          {/* Y grid */}
          {[0, 0.25, 0.5, 0.75, 1].map((p) => {
            const yy = innerH * (1 - p);
            return (
              <g key={p} transform={`translate(0,${yy})`}>
                <line x1={0} x2={innerW} stroke={chartColors.gridline} />
                <text x={-6} y={4} fontSize={10} textAnchor="end" fill={chartColors.axisText}>
                  {Math.round(p * maxScore)}
                </text>
              </g>
            );
          })}
          {/* Hour ticks */}
          {[0, 6, 12, 18, 23].map((h) => (
            <g key={h} transform={`translate(${x(h)},${innerH})`}>
              <line y2={4} stroke={chartColors.axis} />
              <text y={16} fontSize={10} fill={chartColors.axisText} textAnchor="middle">
                {h}h
              </text>
            </g>
          ))}
          {days.map(([day, hours], i) => {
            const path = hours
              .map((s, h) => `${h === 0 ? "M" : "L"}${x(h).toFixed(1)},${y(s).toFixed(1)}`)
              .join(" ");
            const opacity = 0.35 + (i / Math.max(days.length - 1, 1)) * 0.55;
            return (
              <path
                key={day}
                d={path}
                fill="none"
                stroke={chartColors.score}
                strokeOpacity={opacity}
                strokeWidth={1.4}
              />
            );
          })}
          <text x={-m.left + 4} y={-2} fontSize={10} fill={chartColors.axisText}>
            score
          </text>
        </g>
      </svg>
    </ChartCard>
  );
}
