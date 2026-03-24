/**
 * KpiStrip.tsx — horizontal strip of KPI metric badges.
 *
 * Renders each MetricSnapshot as a compact badge with label, value, and
 * color-coded level indicator. Shows nothing when metrics array is empty.
 */
import type { FC } from "react";
import type { MetricSnapshot, MetricLevel } from "@/types";

interface KpiStripProps {
  metrics: MetricSnapshot[];
}

const levelColor: Record<MetricLevel, string> = {
  green: "var(--signal-ok)",
  yellow: "var(--signal-warn, #c8a62c)",
  red: "var(--signal-alert)",
};

/** Hover tooltips explaining what each KPI measures. */
const KPI_TOOLTIPS: Record<string, string> = {
  standing_pct:
    "Standing % — how much of your desk time was spent standing. Green: ≥15%, Yellow: 10-15%, Red: <10%",
  position_rate:
    "Position changes per hour — how often you switch between sitting and standing. Green: ≥1/h, Yellow: 0.5-1/h, Red: <0.5/h",
  hourly_breaks:
    "Hourly breaks — hours with at least 5 min away from screen. Green: all hours covered, Yellow: 1-2 missed, Red: ≥3 missed",
  longest_session:
    "Screen time — longest continuous time at computer without 5+ min break. Green: <45m, Yellow: 45-75m, Red: >75m. Resets after 5 min away.",
};

/** Single KPI badge. */
const KpiBadge: FC<{ metric: MetricSnapshot }> = ({ metric }) => (
  <span
    className="kpi-strip__badge"
    data-testid={`kpi-badge-${metric.id}`}
    style={{ borderColor: levelColor[metric.result.level] }}
    title={KPI_TOOLTIPS[metric.id] ?? metric.label}
  >
    <span className="kpi-strip__label">{metric.label}</span>
    <span
      className="kpi-strip__value"
      style={{ color: levelColor[metric.result.level] }}
    >
      {metric.result.display}
    </span>
    {metric.result.is_personal_best && (
      <span className="kpi-strip__pb" title="Personal best">PB</span>
    )}
  </span>
);

/** Horizontal strip of KPI metrics displayed above the timeline. */
export const KpiStrip: FC<KpiStripProps> = ({ metrics }) => {
  if (metrics.length === 0) return null;

  return (
    <div className="kpi-strip" data-testid="kpi-strip">
      {metrics.map((m) => (
        <KpiBadge key={m.id} metric={m} />
      ))}
    </div>
  );
};
