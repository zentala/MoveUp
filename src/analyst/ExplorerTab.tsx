/**
 * ExplorerTab.tsx — Grid of 5 charts driven by snapshots/sessions/KPI rollups.
 */
import { useMemo } from "react";
import type {
  DailyKpi,
  SessionRow,
  SnapshotRow,
} from "@/test/analyst-fixtures";
import { DateRangePicker, type DateRange } from "./DateRangePicker";
import { DeskHeightTimeline } from "./charts/DeskHeightTimeline";
import { StateGantt } from "./charts/StateGantt";
import { DailyScoreTrajectory } from "./charts/DailyScoreTrajectory";
import { BreakCreditHistogram } from "./charts/BreakCreditHistogram";
import { KpiTrend } from "./charts/KpiTrend";

export interface ExplorerTabProps {
  snapshots: SnapshotRow[];
  sessions: SessionRow[];
  kpis: DailyKpi[];
  range: DateRange;
  onRangeChange: (range: DateRange) => void;
}

function inRange<T extends { ts?: string; dateLocal?: string; startedAt?: string }>(
  rows: T[],
  range: DateRange,
): T[] {
  const from = range.from;
  const to = range.to;
  return rows.filter((r) => {
    const day = (r.ts ?? r.startedAt ?? r.dateLocal ?? "").slice(0, 10);
    return day >= from && day <= to;
  });
}

export function ExplorerTab({
  snapshots,
  sessions,
  kpis,
  range,
  onRangeChange,
}: ExplorerTabProps) {
  const filteredSnaps = useMemo(() => inRange(snapshots, range), [snapshots, range]);
  const filteredSessions = useMemo(() => inRange(sessions, range), [sessions, range]);
  const filteredKpis = useMemo(
    () => kpis.filter((k) => k.dateLocal >= range.from && k.dateLocal <= range.to),
    [kpis, range],
  );

  return (
    <div data-testid="explorer-tab">
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: 16,
        }}
      >
        <DateRangePicker value={range} onChange={onRangeChange} />
        <div style={{ fontSize: 11, color: "#888" }}>
          {filteredSnaps.length} snapshots · {filteredSessions.length} sessions
        </div>
      </div>
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "2fr 1fr",
          gap: 12,
        }}
      >
        <DeskHeightTimeline data={filteredSnaps} />
        <BreakCreditHistogram data={filteredSessions} />
        <StateGantt data={filteredSnaps} />
        <KpiTrend data={filteredKpis} />
        <div style={{ gridColumn: "span 2" }}>
          <DailyScoreTrajectory data={filteredSnaps} />
        </div>
      </div>
    </div>
  );
}
