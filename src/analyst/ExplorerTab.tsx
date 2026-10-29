import { useCallback, useEffect, useMemo, useReducer, useState } from "react";
import type { DailyKpi, SessionRow, SnapshotRow } from "@/test/analyst-fixtures";
import type { DateRange } from "./DateRangePicker";
import { AnalystHeader } from "./AnalystHeader";
import { BreakCreditHistogram } from "./charts/BreakCreditHistogram";

import { DailyScoreTrajectory } from "./charts/DailyScoreTrajectory";
import { DateNavigator } from "./charts/DateNavigator";
import { DateNavigatorHeader } from "./charts/DateNavigatorHeader";
import { TimelineDetail } from "./charts/TimelineDetail";
import { useEventsRange } from "./hooks/useEventsRange";
import { useSessionsRange } from "./hooks/useSessionsRange";
import { useSnapshotsRange } from "./hooks/useSnapshotsRange";
import { useTimelineNav } from "./hooks/useTimelineNav";
import { formatRangeLabel, formatRefreshedAt } from "@/utils/format";
import { localCalendarDay, rangeForOpenDay } from "./open-day";
import type { LiveObservation } from "@/generated/LiveObservation";
import type { QueryStatus } from "@/hooks/useDeskTypes";
import { mergeTimelineIntervals, retainLiveObservationIntervals, sessionIntervals, TimelineDataError } from "@/timeline/intervals";

function readableTime(value: string | null | undefined): string | null {
  if (!value) return null;
  const ms = Date.parse(value);
  return Number.isFinite(ms) ? new Date(ms).toLocaleString([], { dateStyle: "short", timeStyle: "short" }) : null;
}

function liveStatus(observation: LiveObservation | null, mergeError?: string | null): string {
  if (mergeError) return `Current observation unavailable: ${mergeError}`;
  const span = observation?.span;
  const through = readableTime(span?.observed_through);
  if (span && through) {
    const prefix = observation?.quality === "known" ? "Observed" : "Last confirmed";
    const reason = observation?.reason === "stale" ? " · sensor observation stale"
      : observation?.reason === "pending_confirmation" ? " · next posture pending confirmation" : "";
    return `${prefix} ${span.state} through ${through}${reason}`;
  }
  const observedAt = readableTime(observation?.last_observation_at);
  if (observation?.quality === "known" && observedAt) return `Instant observation at ${observedAt}; duration unknown`;
  if (observation?.reason === "no_observation") return "No sensor observations yet";
  if (observation?.reason === "pending_confirmation") return "Next posture pending confirmation; elapsed state is Unknown";
  if (observation?.reason === "clock_rollback") return "Clock moved backward; current observation unavailable";
  return "Current observation unavailable";
}

function liveAnnouncement(observation: LiveObservation | null, mergeError: string | null, refresh?: QueryStatus): string {
  const span = observation?.span;
  const health = mergeError ? `Current observation unavailable: ${mergeError}`
    : observation?.quality === "known" && span ? `${span.state} observation available`
      : observation?.reason === "stale" ? "Sensor observation stale"
        : observation?.reason === "pending_confirmation" ? "Next posture pending confirmation"
          : observation?.reason === "clock_rollback" ? "Clock moved backward; observation unavailable"
            : observation?.reason === "no_observation" ? "No sensor observations yet"
              : "Current observation unavailable";
  return refresh?.error ? `${health} · View update failed: ${refresh.error}` : health;
}

function viewStatus(refresh?: QueryStatus): string {
  const lastResponse = readableTime(refresh?.lastSuccessAt);
  if (refresh?.error) {
    return `View update failed: ${refresh.error}${lastResponse ? ` · last view response received ${lastResponse}` : ""}`;
  }
  if (refresh?.loading) return "View updating";
  return lastResponse ? `View response received ${lastResponse}` : "No successful view response yet";
}

export interface ExplorerTabProps {
  range: DateRange;
  onRangeChange: (range: DateRange) => void;
  snapshots?: SnapshotRow[];
  sessions?: SessionRow[];
  kpis?: DailyKpi[];
  liveObservation?: LiveObservation | null;
  dashboardRefresh?: QueryStatus;
  openDayRequest?: { day: string; requestId: string } | null;
  onOpenDayHandled?: (requestId: string) => void;
}

export function ExplorerTab({ range, onRangeChange, snapshots: propSnapshots, sessions: propSessions,
  liveObservation = null, dashboardRefresh, openDayRequest = null, onOpenDayHandled }: ExplorerTabProps) {
  const isLive = propSnapshots === undefined;
  const snapsQuery = useSnapshotsRange(range.from, range.to, !isLive);
  const sessionsQuery = useSessionsRange(range.from, range.to, !isLive);
  const eventsQuery = useEventsRange(range.from, range.to, !isLive);
  const rangeKey = `${range.from}/${range.to}`;
  const [lastSuccessful, rememberRange] = useReducer(
    (previous: { key: string; snapshots: SnapshotRow[]; sessions: SessionRow[]; refreshed: number | null } | null,
      next: { key: string; snapshots: SnapshotRow[]; sessions: SessionRow[]; refreshed: number | null }) =>
      previous?.key === next.key && previous.snapshots === next.snapshots && previous.sessions === next.sessions &&
        previous.refreshed === next.refreshed ? previous : next,
    null,
  );
  useEffect(() => {
    if (!isLive || snapsQuery.status !== "ready" || sessionsQuery.status !== "ready") return;
    rememberRange({ key: rangeKey, snapshots: snapsQuery.data, sessions: sessionsQuery.data, refreshed: snapsQuery.lastRefreshed });
  }, [isLive, rangeKey, snapsQuery.status, snapsQuery.data, snapsQuery.lastRefreshed,
    sessionsQuery.status, sessionsQuery.data]);
  const snapshots = useMemo(() => isLive
    ? snapsQuery.status === "ready" ? snapsQuery.data : lastSuccessful?.key === rangeKey ? lastSuccessful.snapshots : []
    : propSnapshots ?? [], [isLive, snapsQuery.status, snapsQuery.data, lastSuccessful, rangeKey, propSnapshots]);
  const sessions = useMemo(() => isLive
    ? sessionsQuery.status === "ready" ? sessionsQuery.data : lastSuccessful?.key === rangeKey ? lastSuccessful.sessions : []
    : propSessions ?? [], [isLive, sessionsQuery.status, sessionsQuery.data, lastSuccessful, rangeKey, propSessions]);
  const [retainedObservations, rememberObservation] = useReducer(
    (retained: LiveObservation[], action: { observation: LiveObservation | null; sessions: typeof sessions }) => {
      try {
        const next = retainLiveObservationIntervals(retained, action.observation, action.sessions.map((row) => ({
          start: row.startedAt, end: row.endedAt, state: row.state ?? "", history_kind: row.historyKind,
        })));
        return next.length === retained.length && next.every((observation, index) => observation === retained[index])
          ? retained : next;
      } catch (cause) {
        if (cause instanceof TimelineDataError) return retained;
        throw cause;
      }
    },
    [],
  );
  const sessionEntries = useMemo(() => sessions.map((row) => ({
    start: row.startedAt,
    end: row.endedAt,
    state: row.state ?? "",
    history_kind: row.historyKind,
  })), [sessions]);
  useEffect(() => {
    rememberObservation({ observation: liveObservation, sessions });
  }, [liveObservation, sessions]);
  const [currentNowMs, setNowMs] = useReducer((_: number, now: number) => now, undefined, () => Date.now());
  const history = useMemo(() => {
    try {
      return { intervals: sessionIntervals(sessionEntries), error: null };
    } catch (cause) {
      if (!(cause instanceof TimelineDataError)) throw cause;
      return { intervals: [], error: `History unavailable: ${cause.message}` };
    }
  }, [sessionEntries]);
  const retained = history.error ? retainedObservations
    : retainLiveObservationIntervals(retainedObservations, liveObservation, sessionEntries);
  const timelineData = useMemo(() => {
    if (history.error) return { intervals: [], error: null };
    try {
      return { intervals: mergeTimelineIntervals(sessionEntries, liveObservation, undefined, currentNowMs, retained), error: null };
    } catch (error) {
      return {
        intervals: history.intervals,
        error: error instanceof Error ? error.message : "Invalid live observation",
      };
    }
  }, [sessionEntries, liveObservation, currentNowMs, retained, history]);
  const intervals = timelineData.intervals;
  const readCurrentNow = useCallback(() => currentNowMs, [currentNowMs]);
  const nav = useTimelineNav({ rangeFrom: range.from, rangeTo: range.to, nowMs: readCurrentNow });
  const [todayRequest, setTodayRequest] = useState<string | null>(null);

  useEffect(() => {
    const timer = window.setInterval(() => setNowMs(Date.now()), 60_000);
    return () => window.clearInterval(timer);
  }, []);
  useEffect(() => {
    setNowMs(Date.now());
  }, [dashboardRefresh?.loading, dashboardRefresh?.lastSuccessAt, dashboardRefresh?.error, liveObservation?.snapshot_at]);

  useEffect(() => {
    if (!openDayRequest) return;
    setTodayRequest(null);
    if (openDayRequest.day < range.from || openDayRequest.day > range.to) return;
    nav.goTo(openDayRequest.day);
  }, [openDayRequest?.requestId, openDayRequest?.day, range.from, range.to, nav.goTo]);

  useEffect(() => {
    if (!todayRequest || openDayRequest || todayRequest < range.from || todayRequest > range.to) return;
    nav.goTo(todayRequest);
    setTodayRequest(null);
  }, [todayRequest, openDayRequest?.requestId, range.from, range.to, nav.goTo]);

  useEffect(() => {
    if (openDayRequest?.day === nav.selectedDay) onOpenDayHandled?.(openDayRequest.requestId);
  }, [openDayRequest?.requestId, openDayRequest?.day, nav.selectedDay, onOpenDayHandled]);

  const loading = isLive && (snapsQuery.status === "loading" || sessionsQuery.status === "loading");
  const queryError = isLive
    ? snapsQuery.status === "error" ? snapsQuery.error
      : sessionsQuery.status === "error" ? sessionsQuery.error
        : null
    : null;
  const error = queryError ?? history.error;
  const eventsLoading = isLive && eventsQuery.status === "loading";
  const eventsError = isLive && eventsQuery.status === "error" ? eventsQuery.error : null;
  const rangeLabel = formatRangeLabel(range.from, range.to);
  const refreshedAt = isLive
    ? formatRefreshedAt(snapsQuery.status === "ready" ? snapsQuery.lastRefreshed : lastSuccessful?.key === rangeKey ? lastSuccessful.refreshed : null)
    : null;
  const selectedSnapshots = useMemo(() => snapshots.filter((row) => row.localDate === nav.selectedDay), [snapshots, nav.selectedDay]);
  const selectedSessions = useMemo(() => sessions.filter((row) => row.dateLocal === nav.selectedDay), [sessions, nav.selectedDay]);
  const handleRefresh = () => {
    if (!isLive) return;
    snapsQuery.refetch();
    sessionsQuery.refetch();
    eventsQuery.refetch();
  };
  const handleToday = () => {
    const now = Date.now();
    setNowMs(now);
    const today = localCalendarDay(new Date(now));
    if (today < range.from || today > range.to) {
      setTodayRequest(today);
      onRangeChange(rangeForOpenDay(range, today));
      return;
    }
    nav.goTo(today);
  };

  return <div className="analyst-local-theme" data-testid="explorer-tab">
    <AnalystHeader selectedDay={nav.selectedDay} onPrev={nav.prev} onNext={nav.next} />
    <DateNavigatorHeader range={range} onRangeChange={onRangeChange} onToday={handleToday}
      today={localCalendarDay(new Date(currentNowMs))} />
    <div className="analyst-range-status">
      <p className="analyst-range-status__text" aria-live="polite">
        {error ? `Failed: ${error}${snapshots.length || sessions.length ? ` · showing last successful history${refreshedAt ? ` from ${refreshedAt}` : ""}` : ""}`
          : loading ? `Loading observations…${snapshots.length || sessions.length ? ` · showing last successful history${refreshedAt ? ` from ${refreshedAt}` : ""}` : ""}`
          : `${snapshots.length} snapshots · ${sessions.length} sessions · ${rangeLabel}${refreshedAt ? ` · refreshed ${refreshedAt}` : ""}`}
      </p>
      <p className="analyst-range-status__text" aria-live="off" data-testid="live-observation-status">
        Device connection status unknown · {liveStatus(liveObservation, timelineData.error)} · <span aria-live="off" data-testid="view-response-details">{viewStatus(dashboardRefresh)}</span>
      </p>
      <span role="status" aria-live="polite" data-testid="live-observation-announcement"
        style={{ position: "absolute", width: 1, height: 1, padding: 0, margin: -1, overflow: "hidden", clip: "rect(0, 0, 0, 0)", whiteSpace: "nowrap", border: 0 }}>
        Device connection status unknown · {liveAnnouncement(liveObservation, timelineData.error, dashboardRefresh)}
      </span>
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
        {history.error || loading || (queryError && !snapshots.length && !sessions.length)
          ? <p className="day-composition__status" role={error ? "alert" : "status"}>
          {error ?? "Loading observations…"}
        </p> : <TimelineDetail range={{ from: nav.selectedDay, to: nav.selectedDay }} selectedDay={nav.selectedDay}
          snapshots={snapshots} intervals={intervals} onPrev={nav.prev} onNext={nav.next} nowMs={currentNowMs} showHeader={false} />}
      </div>
      <DateNavigator range={range} selectedDay={nav.selectedDay} intervals={intervals}
        panelId="analyst-selected-day-panel" busy={loading} error={error} nowMs={currentNowMs} onSelectDay={nav.goTo} />
    </section>
    {!error && !loading ? <details className="analyst-day-details">
      <summary>Score and break-credit details for {nav.selectedDay}</summary>
      <DailyScoreTrajectory data={selectedSnapshots} />
      <BreakCreditHistogram data={selectedSessions} />
    </details> : null}
  </div>;
}
