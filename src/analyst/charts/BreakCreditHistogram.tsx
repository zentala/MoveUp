/**
 * BreakCreditHistogram.tsx — Bar chart of break credit counts (none/partial/full).
 *
 * Counts session rows by break credit type and renders three labeled bars.
 */
import type { SessionRow } from "@/test/analyst-fixtures";
import type { BreakCredit } from "@/types";
import { ChartCard } from "./ChartCard";
import { DEFAULT_MARGIN, chartColors, scaleLinear } from "./chart-utils";

export interface BreakCreditHistogramProps {
  data: SessionRow[];
  width?: number;
  height?: number;
}

const BUCKETS: { key: BreakCredit; label: string; color: string }[] = [
  { key: "none", label: "None (<60s)", color: chartColors.alert },
  { key: "partial", label: "Partial (60–120s)", color: chartColors.warn },
  { key: "full", label: "Full (≥120s)", color: chartColors.standing },
];

export function BreakCreditHistogram({
  data,
  width = 360,
  height = 220,
}: BreakCreditHistogramProps) {
  const m = DEFAULT_MARGIN;
  const innerW = width - m.left - m.right;
  const innerH = height - m.top - m.bottom;

  const counts = BUCKETS.map((b) => ({
    ...b,
    count: data.filter((d) => d.breakCredit === b.key).length,
  }));
  const max = Math.max(1, ...counts.map((c) => c.count));
  const y = scaleLinear(0, max, innerH, 0);
  const barW = innerW / counts.length - 16;

  return (
    <ChartCard
      title="Break credit histogram"
      subtitle="Standing-break duration buckets across all sessions."
      help="Counts sessions by how much sit-time their following break cancelled. None: <60s break (no credit). Partial: 60–120s. Full: ≥120s (full proportional credit per ADR 008). Synthesised from durations until break_credit is persisted (see backlog)."
    >
      <svg width={width} height={height} role="img" aria-label="Break credit histogram">
        <g transform={`translate(${m.left},${m.top})`}>
          {/* Grid */}
          {[0, 0.5, 1].map((p) => {
            const yy = innerH * (1 - p);
            return (
              <g key={p} transform={`translate(0,${yy})`}>
                <line x1={0} x2={innerW} stroke={chartColors.gridline} />
                <text x={-6} y={4} fontSize={10} textAnchor="end" fill={chartColors.axisText}>
                  {Math.round(p * max)}
                </text>
              </g>
            );
          })}
          {counts.map((c, i) => {
            const x = i * (innerW / counts.length) + 8;
            const top = y(c.count);
            return (
              <g key={c.key}>
                <rect x={x} y={top} width={barW} height={innerH - top} fill={c.color} />
                <text
                  x={x + barW / 2}
                  y={top - 4}
                  fontSize={11}
                  textAnchor="middle"
                  fill={chartColors.text}
                >
                  {c.count}
                </text>
                <text
                  x={x + barW / 2}
                  y={innerH + 14}
                  fontSize={10}
                  textAnchor="middle"
                  fill={chartColors.axisText}
                >
                  {c.label}
                </text>
              </g>
            );
          })}
        </g>
      </svg>
    </ChartCard>
  );
}
