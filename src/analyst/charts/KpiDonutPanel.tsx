/**
 * KpiDonutPanel.tsx — The three donut KPI cards for the selected day.
 *
 * Replaces the old `KpiTrend` sparklines + `BreakCreditHistogram` bars on the
 * Explorer tab. Range-wide trends stay in `DailyScoreTrajectory`; this panel is
 * the "one day at a glance" snapshot.
 */
import { formatDate, formatDurationShort } from "@/utils/format";
import type { DayKpis } from "../explorer-day-kpis";
import { ChartCard } from "./ChartCard";
import { KpiDonut } from "./KpiDonut";
import { chartColors } from "./chart-utils";

export interface KpiDonutPanelProps {
  kpis: DayKpis;
  /** Disable arc animation (tests, reduced motion). */
  animate?: boolean;
}

export function KpiDonutPanel({ kpis, animate = true }: KpiDonutPanelProps) {
  const standingPctLabel = kpis.standingPct === null
    ? "Unknown"
    : `${Math.round(kpis.standingPct * 100)}%`;

  return (
    <ChartCard
      title="Day at a glance"
      subtitle={formatDate(kpis.dateLocal)}
      help="Standing share uses raw sitting and standing counters. Posture changes show recorded counter values. Score shows signed recorded points from the active ergonomic profile."
    >
      {kpis.hasData ? (
        <div
          data-testid="kpi-donut-panel"
          style={{
            display: "flex",
            flexWrap: "wrap",
            gap: 12,
            justifyContent: "space-around",
          }}
        >
          <KpiDonut
            label="Standing %"
            valuePct={kpis.standingPct ?? 0}
            centerPrimary={standingPctLabel}
            centerSecondary={kpis.standingSecs === null ? "Unknown" : formatDurationShort(kpis.standingSecs)}
            color={chartColors.standing}
            animate={animate}
            pulseKey={kpis.dateLocal}
          />
          <KpiDonut
            label="Posture changes"
            valuePct={0}
            centerPrimary={kpis.positionChanges === null ? "Unknown" : `${kpis.positionChanges}`}
            centerSecondary={kpis.positionChanges === null ? undefined : "recorded"}
            color={chartColors.walking}
            animate={animate}
            pulseKey={kpis.dateLocal}
          />
          <KpiDonut
            label="Daily score"
            valuePct={0}
            centerPrimary={kpis.score === null ? "Unknown" : `${kpis.score}`}
            centerSecondary={kpis.score === null ? undefined : "signed points"}
            color={chartColors.score}
            animate={animate}
            pulseKey={kpis.dateLocal}
          />
        </div>
      ) : (
        <p
          data-testid="kpi-donut-panel-empty"
          style={{ margin: 0, fontSize: 11, color: chartColors.subtext }}
        >
          No data recorded for this day.
        </p>
      )}
    </ChartCard>
  );
}
