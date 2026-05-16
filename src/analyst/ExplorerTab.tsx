/**
 * ExplorerTab.tsx — Grid of 5 charts driven by snapshots/sessions/events.
 *
 * Live mode (default): fetches data via `useSnapshotsRange` / `useEventsRange` /
 * `useSessionsRange` based on the current date range. Mockup mode: pass
 * explicit `snapshots`, `sessions`, `kpis` props to skip the hooks entirely.
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
import { useSnapshotsRange } from "./hooks/useSnapshotsRange";
import { useSessionsRange } from "./hooks/useSessionsRange";
import { useEventsRange } from "./hooks/useEventsRange";
import { chartColors } from "./charts/chart-utils";
import { downsampleSnapshots, deriveDailyKpis } from "./explorer-derivations";

export interface ExplorerTabProps {
  range: DateRange;
  onRangeChange: (range: DateRange) => void;
  /** Optional fixture override — skips live hooks (mockup mode). */
  snapshots?: SnapshotRow[];
  /** Optional fixture override — skips live hooks (mockup mode). */
  sessions?: SessionRow[];
  /** Optional fixture override — skips live hooks (mockup mode). */
  kpis?: DailyKpi[];
}

const DOWNSAMPLE_THRESHOLD = 1000;

export function ExplorerTab({
  range,
  onRangeChange,
  snapshots: propSnapshots,
  sessions: propSessions,
  kpis: propKpis,
}: ExplorerTabProps) {
  const isLive = propSnapshots === undefined;
  const snapsQuery = useSnapshotsRange(range.from, range.to, !isLive);
  const sessionsQuery = useSessionsRange(range.from, range.to, !isLive);
  const eventsQuery = useEventsRange(range.from, range.to, !isLive);

  const liveSnaps = snapsQuery.status === "ready" ? snapsQuery.data : [];
  const liveSessions = sessionsQuery.status === "ready" ? sessionsQuery.data : [];

  const snapshots = isLive ? liveSnaps : propSnapshots ?? [];
  const sessions = isLive ? liveSessions : propSessions ?? [];

  const heightSnaps = useMemo(
    () => downsampleSnapshots(snapshots, DOWNSAMPLE_THRESHOLD),
    // key on length so a stable identity isn't required
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [range.from, range.to, snapshots.length],
  );
  const kpis = useMemo<DailyKpi[]>(
    () => propKpis ?? deriveDailyKpis(snapshots),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [propKpis, range.from, range.to, snapshots.length],
  );

  const loading =
    isLive &&
    (snapsQuery.status === "loading" ||
      sessionsQuery.status === "loading" ||
      eventsQuery.status === "loading");
  const error =
    isLive &&
    (snapsQuery.status === "error"
      ? snapsQuery.error
      : sessionsQuery.status === "error"
        ? sessionsQuery.error
        : eventsQuery.status === "error"
          ? eventsQuery.error
          : null);

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
        <div style={{ fontSize: 11, color: chartColors.subtext }}>
          {error
            ? `Failed: ${error}`
            : loading
              ? "Loading…"
              : `${snapshots.length} snapshots · ${sessions.length} sessions`}
        </div>
      </div>
      <div style={{ display: "grid", gridTemplateColumns: "2fr 1fr", gap: 12 }}>
        <DeskHeightTimeline data={heightSnaps} />
        <BreakCreditHistogram data={sessions} />
        <StateGantt data={snapshots} />
        <KpiTrend data={kpis} />
        <div style={{ gridColumn: "span 2" }}>
          <DailyScoreTrajectory data={snapshots} />
        </div>
      </div>
    </div>
  );
}
