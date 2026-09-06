/**
 * KpiDonutPanel.tsx — The three donut KPI cards for the selected day.
 *
 * Replaces the old `KpiTrend` sparklines + `BreakCreditHistogram` bars on the
 * Explorer tab. Range-wide trends stay in `DailyScoreTrajectory`; this panel is
 * the "one day at a glance" snapshot.
 */
import { formatDate, formatDurationShort } from "@/utils/format";
import type { DayKpis } from "../explorer-day-kpis";
import { POSITION_CHANGE_GOAL, SCORE_MAX } from "../explorer-day-kpis";
import { ChartCard } from "./ChartCard";
import { KpiDonut } from "./KpiDonut";
import { chartColors } from "./chart-utils";

export interface KpiDonutPanelProps {
  kpis: DayKpis;
  /** Disable arc animation (tests, reduced motion). */
  animate?: boolean;
}

export function KpiDonutPanel({ kpis, animate = true }: KpiDonutPanelProps) {
  const standingPctLabel = `${Math.round(kpis.standingPct * 100)}%`;

  return (
    <ChartCard
      title="Day at a glance"
      subtitle={formatDate(kpis.dateLocal)}
      help="Standing % = standing seconds / (sitting + standing) for the selected day. Posture changes = sit↔stand transitions that day, against a daily goal. Score = the daily posture score from the active ergonomic profile, out of 100."
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
            valuePct={kpis.standingPct}
            centerPrimary={standingPctLabel}
            centerSecondary={formatDurationShort(kpis.standingSecs)}
            color={chartColors.standing}
            animate={animate}
          />
          <KpiDonut
            label="Posture changes"
            valuePct={kpis.positionChanges / POSITION_CHANGE_GOAL}
            centerPrimary={`${kpis.positionChanges}`}
            centerSecondary={`of ${POSITION_CHANGE_GOAL} goal`}
            color={chartColors.walking}
            animate={animate}
          />
          <KpiDonut
            label="Daily score"
            valuePct={kpis.score / SCORE_MAX}
            centerPrimary={`${kpis.score}`}
            centerSecondary={`of ${SCORE_MAX}`}
            color={chartColors.score}
            animate={animate}
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
