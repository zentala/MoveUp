/**
 * KpiTrend.tsx — Small multiples: 1 mini sparkline per KPI across 7 days.
 *
 * Shows standing%, position changes, longest session, daily score.
 */
import type { DailyKpi } from "@/test/analyst-fixtures";
import { ChartCard } from "./ChartCard";
import { chartColors, scaleLinear } from "./chart-utils";

export interface KpiTrendProps {
  data: DailyKpi[];
  width?: number;
}

interface KpiSpec {
  key: keyof Pick<DailyKpi, "standingPct" | "positionChanges" | "longestSessionSecs" | "score">;
  label: string;
  unit: string;
  format: (v: number) => string;
}

const KPIS: KpiSpec[] = [
  { key: "standingPct", label: "Standing %", unit: "%", format: (v) => `${v}%` },
  { key: "positionChanges", label: "Changes", unit: "", format: (v) => `${v}` },
  {
    key: "longestSessionSecs",
    label: "Longest session",
    unit: "min",
    format: (v) => `${Math.round(v / 60)}m`,
  },
  { key: "score", label: "Score", unit: "pts", format: (v) => `${v}` },
];

function Sparkline({
  values,
  width,
  height,
  color,
}: {
  values: number[];
  width: number;
  height: number;
  color: string;
}) {
  if (values.length === 0) {
    return <svg width={width} height={height} />;
  }
  const min = Math.min(...values);
  const max = Math.max(...values);
  const x = scaleLinear(0, values.length - 1, 2, width - 2);
  const y = scaleLinear(min, max || 1, height - 4, 4);
  const path = values
    .map((v, i) => `${i === 0 ? "M" : "L"}${x(i).toFixed(1)},${y(v).toFixed(1)}`)
    .join(" ");
  return (
    <svg width={width} height={height} role="img" aria-label="sparkline">
      <path d={path} fill="none" stroke={color} strokeWidth={1.4} />
      {values.map((v, i) => (
        <circle key={i} cx={x(i)} cy={y(v)} r={1.6} fill={color} />
      ))}
    </svg>
  );
}

export function KpiTrend({ data, width = 360 }: KpiTrendProps) {
  return (
    <ChartCard title="KPI trends (7 days)" subtitle="Daily totals — most recent on the right.">
      <div
        style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8 }}
        data-testid="kpi-trend-grid"
      >
        {KPIS.map((k) => {
          const values = data.map((d) => Number(d[k.key]));
          const latest = values[values.length - 1] ?? 0;
          return (
            <div
              key={k.key}
              style={{
                padding: 8,
                background: "#11111c",
                borderRadius: 6,
                border: "1px solid #222230",
              }}
            >
              <div
                style={{
                  fontSize: 10,
                  color: chartColors.subtext,
                  textTransform: "uppercase",
                  letterSpacing: 0.5,
                }}
              >
                {k.label}
              </div>
              <div style={{ fontSize: 18, fontWeight: 600, color: chartColors.text }}>
                {k.format(latest)}
              </div>
              <Sparkline
                values={values}
                width={(width - 32) / 2}
                height={32}
                color={chartColors.primary}
              />
            </div>
          );
        })}
      </div>
    </ChartCard>
  );
}
