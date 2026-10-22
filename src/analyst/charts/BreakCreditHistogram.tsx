/** BreakCreditHistogram.tsx — counts recorded outcomes on completed break rows. */
import type { SessionRow } from "@/test/analyst-fixtures";
import { ChartCard } from "./ChartCard";
import { DEFAULT_MARGIN, chartColors, scaleLinear } from "./chart-utils";
import { creditOutcomes, type CreditOutcome } from "./credit-derivations";

export interface BreakCreditHistogramProps {
  data: SessionRow[];
  width?: number;
  height?: number;
}

const BUCKETS: { key: CreditOutcome; label: string; color: string }[] = [
  { key: "none", label: "None", color: chartColors.alert },
  { key: "partial", label: "Partial", color: chartColors.warn },
  { key: "full", label: "Full", color: chartColors.standing },
  { key: "unknown", label: "Unknown", color: chartColors.away },
];

export function BreakCreditHistogram({
  data,
  width = 360,
  height = 220,
}: BreakCreditHistogramProps) {
  const m = DEFAULT_MARGIN;
  const innerW = width - m.left - m.right;
  const innerH = height - m.top - m.bottom;
  const summary = creditOutcomes(data);
  const counts = BUCKETS.map((bucket) => ({
    ...bucket,
    count: summary.counts[bucket.key],
  }));
  const max = Math.max(1, ...counts.map((bucket) => bucket.count));
  const y = scaleLinear(0, max, innerH, 0);
  const slotW = innerW / counts.length;
  const barW = Math.max(1, slotW - 16);

  return (
    <ChartCard
      title="Break credit outcomes"
      subtitle="Stored classifications on completed break rows."
      help="These are the engine's recorded load-effect categories, governed by the active ergonomic profile. None means no sitting-load credit was recorded, Partial means sitting load remained after credit, and Full means the credited sitting counter reached zero. They are not universal duration bands or medical-effect claims. Unknown marks a source-qualified completed break whose classification was not recorded."
    >
      <p className="credit-coverage" aria-live="polite">
        {summary.eligible === 0
          ? "No completed, recorded break outcomes"
          : `Coverage: ${summary.known} of ${summary.eligible} completed break returns classified · ${summary.counts.unknown} unknown`}
      </p>
      <svg width={width} height={height} role="img" aria-label="Recorded break credit outcomes">
        <g transform={`translate(${m.left},${m.top})`}>
          {[0, 0.5, 1].map((fraction) => {
            const yy = innerH * (1 - fraction);
            return (
              <g key={fraction} transform={`translate(0,${yy})`}>
                <line x1={0} x2={innerW} stroke={chartColors.gridline} />
                <text x={-6} y={4} fontSize={10} textAnchor="end" fill={chartColors.axisText}>
                  {Math.round(fraction * max)}
                </text>
              </g>
            );
          })}
          {counts.map((bucket, index) => {
            const x = index * slotW + (slotW - barW) / 2;
            const top = y(bucket.count);
            return (
              <g key={bucket.key}>
                <rect
                  data-credit={bucket.key}
                  aria-label={`${bucket.label}: ${bucket.count} recorded outcomes`}
                  x={x}
                  y={top}
                  width={barW}
                  height={innerH - top}
                  fill={bucket.color}
                />
                <text
                  x={x + barW / 2}
                  y={top - 4}
                  fontSize={11}
                  textAnchor="middle"
                  fill={chartColors.text}
                >
                  {bucket.count}
                </text>
                <text
                  x={x + barW / 2}
                  y={innerH + 14}
                  fontSize={10}
                  textAnchor="middle"
                  fill={chartColors.axisText}
                >
                  {bucket.label}
                </text>
              </g>
            );
          })}
        </g>
      </svg>
    </ChartCard>
  );
}
