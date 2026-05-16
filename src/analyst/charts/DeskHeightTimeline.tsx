/**
 * DeskHeightTimeline.tsx — Area chart of desk height over time.
 *
 * Renders an SVG area between min height and the per-snapshot height_cm,
 * shaded by state color band underneath.
 */
import type { SnapshotRow } from "@/test/analyst-fixtures";
import { ChartCard } from "./ChartCard";
import { DEFAULT_MARGIN, chartColors, scaleLinear } from "./chart-utils";

export interface DeskHeightTimelineProps {
  data: SnapshotRow[];
  width?: number;
  height?: number;
}

const HEIGHT_MIN = 70;
const HEIGHT_MAX = 120;

export function DeskHeightTimeline({ data, width = 720, height = 220 }: DeskHeightTimelineProps) {
  const m = DEFAULT_MARGIN;
  const innerW = width - m.left - m.right;
  const innerH = height - m.top - m.bottom;

  if (data.length === 0) {
    return (
      <ChartCard title="Desk height timeline" subtitle="Height over the selected range">
        <svg width={width} height={height} role="img" aria-label="empty">
          <text x={width / 2} y={height / 2} fill={chartColors.subtext} textAnchor="middle">
            No data
          </text>
        </svg>
      </ChartCard>
    );
  }

  const t0 = new Date(data[0].ts).getTime();
  const t1 = new Date(data[data.length - 1].ts).getTime();
  const x = scaleLinear(t0, t1, 0, innerW);
  const y = scaleLinear(HEIGHT_MIN, HEIGHT_MAX, innerH, 0);

  const path = data
    .map((d, i) => {
      const px = x(new Date(d.ts).getTime());
      const py = y(d.deskHeightCm);
      return `${i === 0 ? "M" : "L"}${px.toFixed(1)},${py.toFixed(1)}`;
    })
    .join(" ");
  const area = `${path} L${innerW},${innerH} L0,${innerH} Z`;

  const yTicks = [70, 80, 90, 100, 110, 120];

  return (
    <ChartCard
      title="Desk height timeline"
      subtitle="Sensor reading every 5 minutes (cm). Higher = standing."
    >
      <svg width={width} height={height} role="img" aria-label="Desk height timeline">
        <g transform={`translate(${m.left},${m.top})`}>
          {yTicks.map((t) => (
            <g key={t} transform={`translate(0,${y(t)})`}>
              <line x1={0} x2={innerW} stroke={chartColors.gridline} strokeWidth={1} />
              <text x={-6} y={4} textAnchor="end" fontSize={10} fill={chartColors.axisText}>
                {t}
              </text>
            </g>
          ))}
          <path d={area} fill={chartColors.primary} fillOpacity={0.18} />
          <path d={path} fill="none" stroke={chartColors.primary} strokeWidth={1.5} />
          <text
            x={-m.left + 4}
            y={-2}
            fontSize={10}
            fill={chartColors.axisText}
            data-testid="y-axis-label"
          >
            cm
          </text>
        </g>
      </svg>
    </ChartCard>
  );
}
