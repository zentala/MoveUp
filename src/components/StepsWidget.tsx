/**
 * StepsWidget.tsx — today's walking-step count from Google Fit.
 *
 * States:
 *   - not configured  → setup hint (no errors)
 *   - configured + no snapshot yet → "—" placeholder
 *   - snapshot present → step count + "updated Xm ago"
 *
 * Refreshes on mount and every 5 minutes; user can force-refresh via the
 * arrow button. All Tauri errors surface inline; never crashes the panel.
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

/** Format `<n>m ago` / `<h>h ago` / `just now` for the freshness line. */
function formatAgo(fetchedAtMs: number, nowMs: number): string {
  const deltaSec = Math.max(0, Math.floor((nowMs - fetchedAtMs) / 1000));
  if (deltaSec < 30) return "just now";
  if (deltaSec < 3600) return `${Math.floor(deltaSec / 60)}m ago`;
  return `${Math.floor(deltaSec / 3600)}h ago`;
}

export const StepsWidget: FC = () => {
  const [view, setView] = useState<StepsView | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [now, setNow] = useState(Date.now());

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
    const tick = setInterval(() => setNow(Date.now()), 30_000);
    const poll = setInterval(() => {
      if (view?.configured !== false) void refresh();
    }, POLL_INTERVAL_MS);
    return () => {
      clearInterval(tick);
      clearInterval(poll);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  if (view && !view.configured) {
    return (
      <div className="steps-widget steps-widget--unconfigured" data-testid="steps-widget">
        <div className="steps-widget__label">Steps</div>
        <div className="steps-widget__hint">Connect Google Fit in settings</div>
      </div>
    );
  }

  const snap = view?.snapshot ?? null;
  const count = snap ? snap.steps_today.toLocaleString("en-US") : "—";
  const fresh = snap ? formatAgo(snap.fetched_at_ms, now) : "loading…";

  return (
    <div className="steps-widget" data-testid="steps-widget">
      <div className="steps-widget__row">
        <div>
          <div className="steps-widget__label">Steps today</div>
          <div className="steps-widget__count" data-testid="steps-count">
            {count}
          </div>
        </div>
        <button
          className="steps-widget__refresh"
          onClick={() => void refresh()}
          disabled={refreshing}
          title="Refresh now"
          aria-label="Refresh steps"
        >
          {refreshing ? "…" : "↻"}
        </button>
      </div>
      <div className="steps-widget__fresh">{fresh}</div>
      {error && (
        <div className="steps-widget__error" data-testid="steps-error">
          {error}
        </div>
      )}
    </div>
  );
};

export default StepsWidget;
