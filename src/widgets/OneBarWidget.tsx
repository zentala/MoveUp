/**
 * OneBarWidget.tsx — horizontal popup widget with unified progress bar,
 * temperature escalation, timeline, and one-line coach.
 *
 * Visual hierarchy (top to bottom):
 * 1. Header — state dot + label + height + gear
 * 2. Timer — big number + progress bar
 * 3. KPI strip — today's metrics
 * 4. Timeline — session history at bottom
 */
import { useState, useRef, useEffect, useId, type FC, type RefObject } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { DeskState, LiveObservation, WidgetProps } from "@/types";
import StateIndicator from "@/components/StateIndicator";
import { computeTemperature } from "./one-bar/temperature";
import { useTimelineSkin, skinClassName } from "@/hooks/useTimelineSkin";
import { formatObservationAnnouncement, formatObservationStatus, formatObservationViewDetails, OneBarTimeline } from "./one-bar/OneBarTimeline";
import { OneBarTimer } from "./one-bar/OneBarTimer";
import { KpiStrip } from "./one-bar/KpiStrip";
import { HealthWidget } from "@/components/HealthWidget";
import { ShareStats } from "@/components/ShareStats";
import { CalendarDays, Info, Settings, Share2, X } from "lucide-react";
import { UiIcon } from "@/components/UiIcon";
import { formatIdleTime } from "@/utils/format";
import { createOpenDayRequest, localCalendarDay, OPEN_ANALYST_DAY_KEY } from "@/analyst/open-day";
import "./one-bar/one-bar.css";

/** Header props with share callback. */
interface HeaderProps extends WidgetProps {
  onOpenShare: () => void;
  onOpenToday: () => void;
  onCloseMain: () => void;
  shareButtonRef: RefObject<HTMLButtonElement | null>;
  observationDetailsRef: RefObject<HTMLDetailsElement | null>;
}

let todayRequestSequence = 0;

async function openTodayInAnalyst(): Promise<void> {
  const nowMs = Date.now();
  const request = createOpenDayRequest(
    localCalendarDay(new Date(nowMs)),
    nowMs,
    `${nowMs}-${++todayRequestSequence}`,
  );
  const serializedRequest = JSON.stringify(request);
  // Analyst reads this request on activation, so persist it before opening the window.
  localStorage.setItem(OPEN_ANALYST_DAY_KEY, serializedRequest);
  try {
    await invoke("open_analyst_window");
  } catch (error) {
    try {
      if (localStorage.getItem(OPEN_ANALYST_DAY_KEY) === serializedRequest) {
        localStorage.removeItem(OPEN_ANALYST_DAY_KEY);
      }
    } catch {
      // Best-effort cleanup when storage is restricted.
    }
    throw error;
  }
}

/** Header row: StateIndicator (dot + label + height + activity) + share + gear. */
function headerState(
  observation: LiveObservation | null | undefined,
  fallback: DeskState | null,
  refresh: WidgetProps["dashboardRefresh"],
) {
  const span = observation?.span;
  if (span && (observation?.reason === "stale"
    || observation?.reason === "pending_confirmation"
    || observation?.reason === "clock_rollback"
    || observation?.quality !== "known")) {
    return { state: span.state, label: `Last estimate ${span.state}`, showHeight: false };
  }

  const hasCurrentObservation = observation?.quality === "known"
    && (!!span || observation.last_observation_at !== null);
  const state = span?.state ?? (hasCurrentObservation ? fallback : null);
  if (!hasCurrentObservation || !state) return { state: null, label: "Unknown", showHeight: false };
  if (refresh?.error) return { state, label: `Last estimate ${state}`, showHeight: state !== "Away" };
  return { state, label: `Estimated ${state}`, showHeight: state !== "Away" };
}

function observationSummary(observation: LiveObservation | null | undefined, refresh: WidgetProps["dashboardRefresh"]): string {
  const status = formatObservationStatus(observation, refresh).replace(/^Device connection status unknown · /, "");
  const viewSeparator = status.lastIndexOf(" · View ");
  const observationStatus = viewSeparator === -1 ? status : status.slice(0, viewSeparator);
  return refresh?.error ? `View refresh failed · ${observationStatus}` : observationStatus;
}

function observationCue(observation: LiveObservation | null | undefined, refresh: WidgetProps["dashboardRefresh"]): string | null {
  const cues: string[] = [];
  if (observation?.reason === "stale") cues.push("Stale");
  else if (observation?.reason === "clock_rollback") cues.push("Clock changed");
  else if (observation?.reason === "pending_confirmation") cues.push("Pending");
  else if (observation?.quality !== "known") cues.push("Unknown");
  if (refresh?.error) cues.push("View error");
  return cues.length > 0 ? cues.join(" · ") : null;
}

const OneBarHeader: FC<HeaderProps> = ({ shareButtonRef, observationDetailsRef, onOpenToday, onCloseMain, ...props }) => {
  const headingId = useId();
  const state = headerState(props.liveObservation, props.state, props.dashboardRefresh);
  const statusSummary = observationSummary(props.liveObservation, props.dashboardRefresh);
  const statusCue = observationCue(props.liveObservation, props.dashboardRefresh);

  return (
    <div className="one-bar__header">
      <StateIndicator
        state={state.state}
        stateLabel={state.label}
        showHeight={state.showHeight && props.deskHeightCm > 0}
        deskHeightCm={props.deskHeightCm}
        idleSecs={props.idleSecs}
        showActivity={false}
      />
      <details className="one-bar__observation-details" data-testid="observation-details" ref={observationDetailsRef}>
        <summary
          className="one-bar__observation-status"
          data-testid="timeline-observation-status"
          aria-label={`Observation status: ${statusSummary}. Activate for details.`}
          title={`Observation status: ${statusSummary}`}
        >
          <UiIcon icon={Info} />
          {statusCue && <span>{statusCue}</span>}
        </summary>
        <section className="one-bar__observation-panel" role="region" aria-labelledby={headingId}>
          <header className="one-bar__observation-panel-header">
            <h2 id={headingId}>How to read this status</h2>
            <button
              className="one-bar__observation-close"
              type="button"
              aria-label="Close status details"
              onClick={() => {
                observationDetailsRef.current?.removeAttribute("open");
                observationDetailsRef.current?.querySelector("summary")?.focus();
              }}
            >
              Close
            </button>
          </header>
          <div
            className="one-bar__observation-body"
            role="region"
            aria-label="Status explanation"
            tabIndex={0}
            onKeyDown={event => {
              if (event.altKey || event.ctrlKey || event.metaKey) return;
              const body = event.currentTarget;
              const maxScroll = Math.max(0, body.scrollHeight - body.clientHeight);
              let nextScroll: number | null = null;
              switch (event.key) {
                case "ArrowDown": nextScroll = body.scrollTop + 40; break;
                case "ArrowUp": nextScroll = body.scrollTop - 40; break;
                case "PageDown": nextScroll = body.scrollTop + body.clientHeight; break;
                case "PageUp": nextScroll = body.scrollTop - body.clientHeight; break;
                case "Home": nextScroll = 0; break;
                case "End": nextScroll = maxScroll; break;
              }
              if (nextScroll !== null) {
                body.scrollTop = Math.min(maxScroll, Math.max(0, nextScroll));
                event.preventDefault();
              }
            }}
          >
            <p>Desk-state estimates use desk-height observations and recent Windows keyboard/mouse input; they do not measure physical posture or attention.</p>
            <p>No recent input can also mean reading, listening, thinking, or AI work. Low input alone does not prove that you left the desk.</p>
            {props.showActivityStatus !== false && <p><strong>Input activity:</strong> {props.idleSecs >= 30 ? `No recent input · ${formatIdleTime(props.idleSecs)}` : "Recent input"}</p>}
            <p><strong>Observation:</strong> {statusSummary}</p>
            <p><strong>View response:</strong> <span aria-live="off" data-testid="view-response-details">{formatObservationViewDetails(props.dashboardRefresh)}</span></p>
            <p>Timeline colors identify recorded desk state and time.</p>
            <p>Break credit reduces the current Sitting balance, but never removes recorded intervals.</p>
            <p>A sitting-colored interval alone does not mean the limit is exceeded.</p>
          </div>
        </section>
      </details>
      <span role="status" aria-live="polite" data-testid="timeline-observation-announcement"
        style={{ position: "absolute", width: 1, height: 1, padding: 0, margin: -1, overflow: "hidden", clip: "rect(0, 0, 0, 0)", whiteSpace: "nowrap", border: 0 }}>
        {formatObservationAnnouncement(props.liveObservation, props.dashboardRefresh)}
      </span>
      <div className="one-bar__header-actions">
        <button
          ref={shareButtonRef}
          className="one-bar__share-btn"
          type="button"
          onClick={props.onOpenShare}
          title="Share my stats"
          aria-label="Share my stats"
        >
          <UiIcon icon={Share2} />
        </button>
        <button
          className="one-bar__today-btn"
          type="button"
          onClick={onOpenToday}
          title="Open Analyst for today's local day"
          aria-label="Open Analyst for today's local day"
        >
          <UiIcon icon={CalendarDays} />
          <span>Today</span>
        </button>
        <button
          className="one-bar__settings-btn"
          type="button"
          onClick={props.onOpenSettings}
          title="Settings"
          aria-label="Settings"
        >
          <UiIcon icon={Settings} />
        </button>
        {props.onShareChange && (
          <button
            className="one-bar__close-btn"
            type="button"
            onClick={onCloseMain}
            title="Close popup to tray"
            aria-label="Close popup to tray"
          >
            <UiIcon icon={X} />
          </button>
        )}
      </div>
    </div>
  );
};

/** One Bar widget — horizontal layout with header, timer, KPIs, timeline. */
export const OneBarWidget: FC<WidgetProps> = (props) => {
  const [showShare, setShowShare] = useState(false);
  const shareButtonRef = useRef<HTMLButtonElement>(null);
  const observationDetailsRef = useRef<HTMLDetailsElement>(null);
  const wasSharing = useRef(false);
  const [shareError, setShareError] = useState<string | null>(null);
  const shareChanging = useRef(false);
  const [todayError, setTodayError] = useState<string | null>(null);
  const [closeMainError, setCloseMainError] = useState<string | null>(null);
  const todayChanging = useRef(false);
  const closeMainChanging = useRef(false);
  async function closeMain() {
    if (closeMainChanging.current) return;
    closeMainChanging.current = true;
    setCloseMainError(null);
    try {
      await invoke("hide_main_window");
    } catch {
      setCloseMainError("Could not close the popup. Please retry.");
    } finally {
      closeMainChanging.current = false;
    }
  }
  async function openToday() {
    if (todayChanging.current) return;
    todayChanging.current = true;
    setTodayError(null);
    try {
      await openTodayInAnalyst();
    } catch {
      setTodayError("Could not open Analyst for today. Please retry.");
    } finally {
      todayChanging.current = false;
    }
  }
  async function changeShare(open: boolean) {
    if (!props.onShareChange) { setShowShare(open); setShareError(null); return; }
    if (shareChanging.current) return;
    shareChanging.current = true;
    try {
      await props.onShareChange(open);
      setShowShare(open); setShareError(null);
    } catch { setShareError("Could not change the Share view. Please retry."); }
    finally { shareChanging.current = false; }
  }
  useEffect(() => {
    if (wasSharing.current && !showShare) shareButtonRef.current?.focus();
    wasSharing.current = showShare;
  }, [showShare]);
  const temperature = computeTemperature(props);
  const [skin] = useTimelineSkin();

  return (
    <div
      className={`one-bar one-bar--${temperature} ${skinClassName(skin)}`}
      data-testid="one-bar-widget"
    >
      <OneBarHeader {...props} shareButtonRef={shareButtonRef} observationDetailsRef={observationDetailsRef} onOpenShare={() => { void changeShare(true); }} onOpenToday={() => { void openToday(); }} onCloseMain={() => { void closeMain(); }} />
      {closeMainError && <div className="one-bar__action-error" role="alert"><span>{closeMainError}</span><button type="button" onClick={() => { void closeMain(); }}>Retry close</button></div>}
      {todayError && <div className="one-bar__action-error" role="alert"><span>{todayError}</span><button type="button" onClick={() => { void openToday(); }}>Retry opening Analyst</button></div>}
      {shareError && !showShare && <p role="alert">{shareError}</p>}
      <OneBarTimer {...props} />
      <KpiStrip metrics={props.metrics}>
        <HealthWidget />
      </KpiStrip>
      <OneBarTimeline {...props} />

      {showShare && (
        <ShareStats
          metrics={props.metrics}
          todayChanges={props.todayChanges}
          todayStandingSecs={props.todayStandingSecs}
          todaySittingSecs={props.todaySittingSecs}
          todayScore={props.todayScore}
          onClose={() => { void changeShare(false); }}
          closeError={shareError}
          onRetryClose={() => { void changeShare(false); }}
        />
      )}
    </div>
  );
};
