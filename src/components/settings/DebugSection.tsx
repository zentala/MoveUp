/**
 * DebugSection.tsx — raw data dump for debugging the session state machine.
 *
 * Shows all values from useDesk() in readable groups so the developer
 * can verify what the backend sends vs what the UI displays.
 */
import { useState, useEffect } from "react";
import type { FC } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useDesk } from "@/hooks/useDesk";
import { formatDurationShort } from "@/utils/format";
import "./debug.css";

/** Formats seconds as "Xm Ys" or just "Xs". */
function fmtSecs(s: number): string {
  return `${formatDurationShort(Math.abs(s))}${s < 0 ? " (neg)" : ""}`;
}

/** Single key-value row in the debug panel. */
const Row: FC<{ label: string; value: string | number; warn?: boolean }> = ({
  label,
  value,
  warn,
}) => (
  <div className={`debug-row${warn ? " debug-row--warn" : ""}`}>
    <span className="debug-row__label">{label}</span>
    <span className="debug-row__value">{String(value)}</span>
  </div>
);

/** Group header in the debug panel. */
const Group: FC<{ title: string; children: React.ReactNode }> = ({
  title,
  children,
}) => (
  <div className="debug-group">
    <div className="debug-group__title">{title}</div>
    {children}
  </div>
);

/** Debug data panel — shows raw session state for troubleshooting. */
const DebugSection: FC = () => {
  const d = useDesk();
  const [config, setConfig] = useState<Record<string, unknown> | null>(null);

  useEffect(() => {
    invoke("get_settings")
      .then((s) => setConfig(s as Record<string, unknown>))
      .catch(() => {});
  }, []);

  const limitPct = d.sessionLimitSecs > 0
    ? ((d.limitUsedSecs / d.sessionLimitSecs) * 100).toFixed(0)
    : "n/a";

  return (
    <div className="debug-section">
      <Group title="Current State">
        <Row label="state" value={d.state} />
        <Row label="connected" value={d.connected ? "yes" : "no"} warn={!d.connected} />
        <Row label="port" value={d.port ?? "—"} warn={!d.port} />
        <Row label="desk height" value={`${Math.round(d.deskHeightCm)} cm`} />
        <Row label="error" value={d.error ?? "—"} warn={!!d.error} />
      </Group>

      <Group title="Sitting Session">
        <Row label="this session (sitting)" value={fmtSecs(d.sittingSeconds)} />
        <Row label="limit used" value={`${fmtSecs(d.limitUsedSecs)} (${limitPct}%)`} />
        <Row label="limit remaining" value={fmtSecs(d.limitRemaining)} warn={d.limitRemaining < 0} />
        <Row label="limit ratio" value={d.limitRatio.toFixed(2)} warn={d.limitRatio > 1} />
        <Row label="limit total" value={fmtSecs(d.sessionLimitSecs)} />
      </Group>

      <Group title="Break / Standing">
        <Row label="breakSeconds" value={fmtSecs(d.breakSeconds)} />
        <Row label="standingSeconds (daily)" value={fmtSecs(d.standingSeconds)} />
        <Row label="breakResetProgress" value={`${(d.breakResetProgress * 100).toFixed(0)}%`} />
        <Row label="breakResetThreshold" value={fmtSecs(d.breakResetThreshold)} />
      </Group>

      <Group title="Today Totals">
        <Row label="todaySittingSecs" value={fmtSecs(d.todaySittingSecs)} />
        <Row label="todayStandingSecs" value={fmtSecs(d.todayStandingSecs)} />
        <Row label="positionChanges" value={d.positionChanges} />
        <Row label="dailyScore" value={d.dailyScore.toFixed(1)} />
      </Group>

      <Group title="Transition">
        {d.transition ? (
          <>
            <Row label="transitionTo" value={d.transition.transitionTo} />
            <Row label="lastBreakSecs" value={fmtSecs(d.transition.lastBreakSecs)} />
            <Row label="lastSittingSecs" value={fmtSecs(d.transition.lastSittingSecs)} />
            <Row label="breakCredit" value={d.transition.breakCredit} />
          </>
        ) : (
          <Row label="(no recent transition)" value="—" />
        )}
      </Group>

      <Group title="Previous Session">
        {d.previousSession ? (
          <>
            <Row label="state" value={d.previousSession.state} />
            <Row label="durationSecs" value={fmtSecs(d.previousSession.durationSecs)} />
            <Row label="wasEffective" value={d.previousSession.wasEffective ? "yes" : "no"} />
          </>
        ) : (
          <Row label="(none yet)" value="—" />
        )}
      </Group>

      <Group title="Notification Criteria">
        <Row
          label="posture balance"
          value={d.todaySittingSecs > d.todayStandingSecs * 2 ? "TRIGGERED" : "ok"}
          warn={d.todaySittingSecs > d.todayStandingSecs * 2}
        />
        <Row label="  sitting/standing ratio" value={
          d.todayStandingSecs > 0
            ? `${(d.todaySittingSecs / d.todayStandingSecs).toFixed(1)}x (fires at >2x)`
            : "∞ (no standing yet)"
        } />
      </Group>

      {config && (
        <Group title="Config">
          <Row label="sit_limit_mins" value={String(config.sit_limit_mins)} />
          <Row label="stand_limit_mins" value={String(config.stand_limit_mins)} />
          <Row label="standing_target_mins" value={String(config.standing_target_mins)} />
          <Row label="notify_inactivity" value={String(config.notify_inactivity)} />
          <Row label="notify_posture_balance" value={String(config.notify_daily_posture_balance)} />
          <Row label="notify_praise_halfway" value={String(config.notify_praise_halfway)} />
        </Group>
      )}

      <Group title="Sessions Today ({d.todaySessions.length})">
        {d.todaySessions.length === 0 && <Row label="(empty)" value="—" />}
        {d.todaySessions.slice(-5).map((s, i) => (
          <Row
            key={i}
            label={`${s.state} ${fmtSecs(s.duration_secs)}`}
            value={new Date(s.start).toLocaleTimeString()}
          />
        ))}
        {d.todaySessions.length > 5 && (
          <Row label="..." value={`${d.todaySessions.length - 5} more`} />
        )}
      </Group>
    </div>
  );
};

export default DebugSection;
