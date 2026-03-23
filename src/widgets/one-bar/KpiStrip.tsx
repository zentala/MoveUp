/**
 * KpiStrip.tsx — renders a row of color-coded KPI metrics.
 * Receives Vec<MetricSnapshot> from useDesk and renders dynamically.
 */
import type { FC } from "react";
import "./kpi-strip.css";

interface MetricResult {
  value: number;
  display: string;
  level: "green" | "yellow" | "red";
  is_personal_best: boolean;
}

interface MetricSnapshot {
  id: string;
  label: string;
  result: MetricResult;
}

interface KpiStripProps {
  metrics: MetricSnapshot[];
}

export const KpiStrip: FC<KpiStripProps> = ({ metrics }) => {
  if (metrics.length === 0) return null;

  return (
    <div className="kpi-strip" data-testid="kpi-strip">
      {metrics.map((m) => (
        <div
          key={m.id}
          className="kpi-strip__item"
          title={`${m.label}: ${m.result.display}`}
        >
          <span className={`kpi-strip__dot kpi-strip__dot--${m.result.level}`} />
          <span className="kpi-strip__value">
            {m.result.display}
            {m.result.is_personal_best && <span className="kpi-strip__best">★</span>}
          </span>
        </div>
      ))}
    </div>
  );
};
