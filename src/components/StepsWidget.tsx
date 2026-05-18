/**
 * StepsWidget.tsx — compact "steps today" badge for the OneBar popup.
 *
 * Visual language matches the KPI strip: borderless badge with small label
 * + bold value. Sits next to the other KPIs.
 *
 * Render states:
 *   - not configured  → muted "Steps: connect" hint
 *   - configured + no snapshot yet → "Steps: —"
 *   - snapshot present → "Steps: 7,421" with a tiny refresh button
 *
 * Polls every 5 minutes; manual refresh via button. Errors surface inline
 * (as a tooltip on the badge) so the popup never crashes.
 */
import { useCallback, useEffect, useState, type FC } from "react";
import { invoke } from "@tauri-apps/api/core";

const POLL_INTERVAL_MS = 5 * 60 * 1000;

interface StepsSnapshot {
  steps_today: number;
  fetched_at_ms: number;
}

interface StepsView {
  configured: boolean;
  snapshot: StepsSnapshot | null;
}

export const StepsWidget: FC = () => {
  const [view, setView] = useState<StepsView | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);

  const loadCached = useCallback(async () => {
    try {
      const v = await invoke<StepsView>("get_steps_today");
      setView(v);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const refresh = useCallback(async () => {
    setRefreshing(true);
    try {
      const snap = await invoke<StepsSnapshot>("refresh_steps_now");
      setView({ configured: true, snapshot: snap });
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setRefreshing(false);
    }
  }, []);

  useEffect(() => {
    void loadCached();
    // First refresh shortly after mount (only if configured) — cache is
    // empty on cold start.
    const kick = setTimeout(() => void refresh(), 500);
    const poll = setInterval(() => void refresh(), POLL_INTERVAL_MS);
    return () => {
      clearTimeout(kick);
      clearInterval(poll);
    };
  }, [loadCached, refresh]);

  if (view && !view.configured) {
    return (
      <span
        className="kpi-strip__badge steps-widget steps-widget--unconfigured"
        data-testid="steps-widget"
        title="Set GOOGLE_REFRESH_TOKEN in .env"
      >
        <span className="kpi-strip__label">Steps</span>
        <span className="kpi-strip__value">connect google fit</span>
      </span>
    );
  }

  const snap = view?.snapshot ?? null;
  const count = snap ? snap.steps_today.toLocaleString("en-US") : "—";

  return (
    <span
      className="kpi-strip__badge steps-widget"
      data-testid="steps-widget"
      title={error ? `Error: ${error}` : "Steps today (Google Fit)"}
    >
      <span className="kpi-strip__label">Steps</span>
      <span className="kpi-strip__value" data-testid="steps-count">
        {count}
      </span>
      <button
        className="steps-widget__refresh"
        onClick={() => void refresh()}
        disabled={refreshing}
        aria-label="Refresh steps"
        title="Refresh now"
      >
        {refreshing ? "…" : "↻"}
      </button>
      {error && (
        <span className="steps-widget__error-dot" data-testid="steps-error" aria-hidden>
          •
        </span>
      )}
    </span>
  );
};

export default StepsWidget;
