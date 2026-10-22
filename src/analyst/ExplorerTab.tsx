import { useEffect, useMemo, useState } from "react";
import type { DailyKpi, SessionRow, SnapshotRow } from "@/test/analyst-fixtures";
import type { DateRange } from "./DateRangePicker";
import { AnalystHeader } from "./AnalystHeader";
import { BreakCreditHistogram } from "./charts/BreakCreditHistogram";

import { DailyScoreTrajectory } from "./charts/DailyScoreTrajectory";
import { DateNavigator } from "./charts/DateNavigator";
import { DateNavigatorHeader } from "./charts/DateNavigatorHeader";
import { TimelineDetail } from "./charts/TimelineDetail";
import { analystTimelineIntervals } from "./charts/timeline-utils";
import { useEventsRange } from "./hooks/useEventsRange";
import { useSessionsRange } from "./hooks/useSessionsRange";
import { useSnapshotsRange } from "./hooks/useSnapshotsRange";
import { useTimelineNav } from "./hooks/useTimelineNav";
import { formatRangeLabel, formatRefreshedAt } from "@/utils/format";
import { localCalendarDay } from "./open-day";

export interface ExplorerTabProps {
  range: DateRange;
  onRangeChange: (range: DateRange) => void;
  snapshots?: SnapshotRow[];
  sessions?: SessionRow[];
  kpis?: DailyKpi[];
  openDayRequest?: { day: string; requestId: string } | null;
  onOpenDayHandled?: (requestId: string) => void;
}

export function ExplorerTab({ range, onRangeChange, snapshots: propSnapshots, sessions: propSessions,
  openDayRequest = null, onOpenDayHandled }: ExplorerTabProps) {
  const isLive = propSnapshots === undefined;
  const snapsQuery = useSnapshotsRange(range.from, range.to, !isLive);
  const sessionsQuery = useSessionsRange(range.from, range.to, !isLive);
  const eventsQuery = useEventsRange(range.from, range.to, !isLive);
  const snapshots = isLive ? (snapsQuery.status === "ready" ? snapsQuery.data : []) : propSnapshots ?? [];
  const sessions = isLive ? (sessionsQuery.status === "ready" ? sessionsQuery.data : []) : propSessions ?? [];
  const intervals = useMemo(() => analystTimelineIntervals(sessions), [sessions]);
  const nav = useTimelineNav({ rangeFrom: range.from, rangeTo: range.to });
  const [nowMs, setNowMs] = useState(() => Date.now());

  useEffect(() => {
    const timer = window.setInterval(() => setNowMs(Date.now()), 60_000);
    return () => window.clearInterval(timer);
  }, []);

  useEffect(() => {
    if (!openDayRequest || openDayRequest.day < range.from || openDayRequest.day > range.to) return;
    nav.goTo(openDayRequest.day);
  }, [openDayRequest?.requestId, openDayRequest?.day, range.from, range.to, nav.goTo]);

  useEffect(() => {
    if (openDayRequest?.day === nav.selectedDay) onOpenDayHandled?.(openDayRequest.requestId);
  }, [openDayRequest?.requestId, openDayRequest?.day, nav.selectedDay, onOpenDayHandled]);

  const loading = isLive && (snapsQuery.status === "loading" || sessionsQuery.status === "loading");
  const error = isLive
    ? snapsQuery.status === "error" ? snapsQuery.error
      : sessionsQuery.status === "error" ? sessionsQuery.error
        : null
    : null;
  const eventsLoading = isLive && eventsQuery.status === "loading";
  const eventsError = isLive && eventsQuery.status === "error" ? eventsQuery.error : null;
  const rangeLabel = formatRangeLabel(range.from, range.to);
  const refreshedAt = isLive && snapsQuery.status === "ready" ? formatRefreshedAt(snapsQuery.lastRefreshed) : null;
  const selectedSnapshots = useMemo(() => snapshots.filter((row) => row.localDate === nav.selectedDay), [snapshots, nav.selectedDay]);
  const selectedSessions = useMemo(() => sessions.filter((row) => row.dateLocal === nav.selectedDay), [sessions, nav.selectedDay]);
  const handleRefresh = () => {
    if (!isLive) return;
    snapsQuery.refetch();
    sessionsQuery.refetch();
    eventsQuery.refetch();
  };

  return <div className="analyst-local-theme" data-testid="explorer-tab">
    <AnalystHeader selectedDay={nav.selectedDay} onPrev={nav.prev} onNext={nav.next} />
    <DateNavigatorHeader range={range} onRangeChange={onRangeChange} today={localCalendarDay(new Date(nowMs))} />
    <div className="analyst-range-status">
      <p className="analyst-range-status__text" aria-live="polite">
        {error ? `Failed: ${error}` : loading ? "Loading observations…"
          : `${snapshots.length} snapshots · ${sessions.length} sessions · ${rangeLabel}${refreshedAt ? ` · refreshed ${refreshedAt}` : ""}`}
      </p>
      {eventsLoading ? <p className="analyst-range-status__text" role="status">Loading supplemental events…</p> : null}
      {eventsError ? <p className="analyst-range-status__text" role="alert">
        Events unavailable: {eventsError} <button type="button" onClick={eventsQuery.refetch}>Retry events</button>
      </p> : null}
      {isLive ? <button type="button" className="analyst-refresh" onClick={handleRefresh} disabled={loading}
        aria-label={error ? "Retry loading Analyst data" : "Refresh"}>
        {error ? "Retry" : "Refresh"}
      </button> : null}
    </div>
    <section className="day-composition__panel" aria-label={`Selected day ${nav.selectedDay}`}>
      <div role="tabpanel" id="analyst-selected-day-panel" aria-labelledby={`analyst-day-${nav.selectedDay}`} aria-busy={loading} tabIndex={0}>
        <TimelineDetail range={{ from: nav.selectedDay, to: nav.selectedDay }} selectedDay={nav.selectedDay}
          snapshots={snapshots} intervals={intervals} onPrev={nav.prev} onNext={nav.next} nowMs={nowMs} showHeader={false} />
      </div>
      <DateNavigator range={range} selectedDay={nav.selectedDay} intervals={intervals}
        panelId="analyst-selected-day-panel" busy={loading} error={error} nowMs={nowMs} onSelectDay={nav.goTo} />
    </section>
    <details className="analyst-day-details">
      <summary>Score and break-credit details for {nav.selectedDay}</summary>
      <DailyScoreTrajectory data={selectedSnapshots} />
      <BreakCreditHistogram data={selectedSessions} />
    </details>
  </div>;
}
