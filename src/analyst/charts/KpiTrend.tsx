/**
 * KpiTrend.tsx — Small multiples: 1 mini sparkline per KPI across 7 days.
 *
 * Shows standing%, position changes, longest session, daily score.
 */
import type { DailyKpi } from "@/test/analyst-fixtures";
import { formatDurationShort } from "@/utils/format";
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
  format: (v: number | null) => string;
}

const KPIS: KpiSpec[] = [
  { key: "standingPct", label: "Standing %", unit: "%", format: (v) => v === null ? "Unknown" : `${v}%` },
  { key: "positionChanges", label: "Changes", unit: "", format: (v) => v === null ? "Unknown" : `${v}` },
  {
    key: "longestSessionSecs",
    label: "Longest session",
    unit: "",
    format: (v) => v === null ? "Unknown" : formatDurationShort(v),
  },
  { key: "score", label: "Score", unit: "pts", format: (v) => v === null ? "Unknown" : `${v}` },
];

function Sparkline({
  values,
  width,
  height,
  color,
}: {
  values: Array<number | null>;
  width: number;
  height: number;
  color: string;
}) {
  const present = values.filter((value): value is number => value !== null && Number.isFinite(value));
  if (present.length === 0) {
    return <svg width={width} height={height} role="img" aria-label="sparkline; all values unknown" />;
  }
  const min = Math.min(...present);
  const max = Math.max(...present);
  const x = scaleLinear(0, values.length - 1, 2, width - 2);
  const y = scaleLinear(min, max || 1, height - 4, 4);
  const path = values.reduce<{ path: string; inRun: boolean }>((run, value, i) => {
    if (value === null || !Number.isFinite(value)) return { ...run, inRun: false };
    const command = run.inRun ? "L" : "M";
    return {
      path: `${run.path}${run.path ? " " : ""}${command}${x(i).toFixed(1)},${y(value).toFixed(1)}`,
      inRun: true,
    };
  }, { path: "", inRun: false }).path;
  return (
    <svg width={width} height={height} role="img" aria-label="sparkline; gaps mark unknown values">
      <path d={path} fill="none" stroke={color} strokeWidth={1.4} />
      {values.map((value, i) => value === null || !Number.isFinite(value) ? null : (
        <circle key={i} data-index={i} cx={x(i)} cy={y(value)} r={1.6} fill={color} />
      ))}
    </svg>
  );
}

export function KpiTrend({ data, width = 360 }: KpiTrendProps) {
  return (
    <ChartCard
      title="KPI trends (7 days)"
      subtitle="Daily totals — most recent on the right."
      help="Big number = today (last day in range). Sparkline = one point per day. Standing% = standing seconds / (sitting + standing). Changes = sit↔stand transitions today. Longest session = max contiguous standing run. Score = posture score per active ergonomic profile."
    >
      <div
        style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 8 }}
        data-testid="kpi-trend-grid"
      >
        {KPIS.map((k) => {
          const values = data.map((d) => {
            const value = d[k.key];
            return typeof value === "number" && Number.isFinite(value) ? value : null;
          });
          const latest = values[values.length - 1] ?? null;
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
