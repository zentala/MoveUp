/**
 * PlaceholderWidget.tsx — dev verification widget that renders all WidgetProps.
 *
 * Displays every field from WidgetProps as raw data so developers can verify
 * the widget system is wiring data correctly. Will be replaced by real widgets
 * (OneBar, TimelineZen) in T032/T033.
 */
import type { FC } from "react";
import type { WidgetProps } from "@/types";
import { formatDuration } from "@/utils/format";

/** Renders all widget data as a compact debug view. */
export const PlaceholderWidget: FC<WidgetProps> = (props) => {
  const stateLabel = props.state ?? "waiting…";
  const heightLabel = props.deskHeightCm > 0
    ? `${props.deskHeightCm.toFixed(0)} cm`
    : "—";

  return (
    <div className="placeholder-widget" data-testid="placeholder-widget">
      <div className="placeholder-widget__header">
        <span className="placeholder-widget__title">
          Widget Debug View
        </span>
        <button
          className="btn btn--link"
          onClick={props.onOpenSettings}
          title="Open settings"
        >
          settings
        </button>
      </div>

      <div className="placeholder-widget__section">
        <h4>Connection</h4>
        <p>connected: {String(props.connected)} | port: {props.port ?? "none"}</p>
      </div>

      <div className="placeholder-widget__section">
        <h4>State</h4>
        <p>state: {stateLabel} | height: {heightLabel}</p>
        <p>session: {formatDuration(props.currentSessionSecs)}</p>
      </div>

      <div className="placeholder-widget__section">
        <h4>Limit</h4>
        <p>
          remaining: {props.limitRemaining}s |
          ratio: {(props.limitRatio * 100).toFixed(1)}% |
          limit: {props.limitSecs}s
        </p>
      </div>

      <div className="placeholder-widget__section">
        <h4>Break</h4>
        <p>
          break: {formatDuration(props.breakSecs)} |
          reset: {(props.breakResetProgress * 100).toFixed(0)}% of {props.breakResetThreshold}s
        </p>
      </div>

      <div className="placeholder-widget__section">
        <h4>Previous Session</h4>
        <p>
          {props.previousSession
            ? `${props.previousSession.state} ${props.previousSession.durationSecs}s (effective: ${String(props.previousSession.wasEffective)})`
            : "none"}
        </p>
      </div>

      <div className="placeholder-widget__section">
        <h4>Today</h4>
        <p>
          sit: {props.todaySittingSecs}s |
          stand: {props.todayStandingSecs}s |
          changes: {props.todayChanges} |
          score: {props.todayScore} |
          sessions: {props.todaySessions.length}
        </p>
      </div>

      {props.error && (
        <div className="error-banner">{props.error}</div>
      )}
    </div>
  );
};
