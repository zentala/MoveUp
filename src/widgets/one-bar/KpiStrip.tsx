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

/** Single KPI badge. */
const KpiBadge: FC<{ metric: MetricSnapshot }> = ({ metric }) => (
  <span
    className="kpi-strip__badge"
    data-testid={`kpi-badge-${metric.id}`}
    style={{ borderColor: levelColor[metric.result.level] }}
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
