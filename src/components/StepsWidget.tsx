/**
 * StepsWidget.tsx — compact "steps today" badge for the OneBar popup.
 *
 * Visual language matches the KPI strip: borderless badge with small label
 * + bold value. Slotted INSIDE KpiStrip so it participates in the same
 * flex-wrap flow.
 *
 * Render states:
 *   - not configured  → muted "connect google fit" hint
 *   - configured, no snapshot yet → "—"
 *   - snapshot fresh (<1h) → step count + refresh button
 *   - snapshot stale (>1h) → step count dimmed, "(stale)" in tooltip
 *   - auth_revoked → "reconnect google fit" CTA (tooltip explains how)
 *   - transient error → red dot indicator next to the value
 *
 * Refresh cadence:
 *   - First refresh fires ~500ms after mount (cache warm-up).
 *   - On success: next refresh in 5 minutes.
 *   - On transient error: exponential backoff 1m → 2m → 5m → 10m → 30m cap.
 *   - On auth_revoked: stop auto-refresh (no point until user reconnects).
 *
 * Concurrency: an in-flight refresh ref guards against double-call from
 * rapid button clicks. Backend also dedupes, this is just a UI nicety.
 */
import { useCallback, useEffect, useRef, useState, type FC } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useExponentialPoll } from "@/hooks/useExponentialPoll";

const INITIAL_KICK_DELAY_MS = 500;
const SUCCESS_INTERVAL_MS = 5 * 60 * 1000;
const STALE_THRESHOLD_MS = 60 * 60 * 1000;
const BACKOFF_LADDER_MS = [
  60 * 1000,
  2 * 60 * 1000,
  5 * 60 * 1000,
  10 * 60 * 1000,
  30 * 60 * 1000,
];
const NUMBER_FORMATTER = new Intl.NumberFormat(
  typeof navigator !== "undefined" ? navigator.language : "en-US",
);

type ErrorKind = "transient" | "auth_revoked";

interface StepsSnapshot {
  steps_today: number;
  fetched_at_ms: number;
}

interface StepsView {
  configured: boolean;
  snapshot: StepsSnapshot | null;
  error_kind?: ErrorKind;
  error_message?: string;
}

export const StepsWidget: FC = () => {
  const [view, setView] = useState<StepsView | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const inflightRef = useRef(false);

  const loadCached = useCallback(async () => {
    try {
      const v = await invoke<StepsView>("get_steps_today");
      setView(v);
    } catch (e) {
      // Treat IPC failure (unlikely in production) as transient.
      setView({
        configured: true,
        snapshot: null,
        error_kind: "transient",
        error_message: String(e),
      });
    }
  }, []);

  const refresh = useCallback(async (): Promise<StepsView | null> => {
    if (inflightRef.current) return null;
    inflightRef.current = true;
    setRefreshing(true);
    try {
      const v = await invoke<StepsView>("refresh_steps_now");
      setView(v);
      return v;
    } catch (e) {
      const v: StepsView = {
        configured: true,
        snapshot: null,
        error_kind: "transient",
        error_message: String(e),
      };
      setView(v);
      return v;
    } finally {
      inflightRef.current = false;
      setRefreshing(false);
    }
  }, []);

  // Warm the cached view once on mount; the poll hook drives all
  // subsequent refreshes.
  useEffect(() => {
    void loadCached();
  }, [loadCached]);

  // Exponential-backoff polling delegated to the shared hook.
  useExponentialPoll<StepsView | null>(refresh, {
    initialDelayMs: INITIAL_KICK_DELAY_MS,
    successIntervalMs: SUCCESS_INTERVAL_MS,
    backoffLadderMs: BACKOFF_LADDER_MS,
    isFailure: (v) => v?.error_kind === "transient",
    // Halt on auth_revoked — retrying without re-consent is pointless.
    isTerminal: (v) => v?.error_kind === "auth_revoked",
  });

  // ─── Render branches ────────────────────────────────────────────────

  if (view && !view.configured) {
    return (
      <span
        className="kpi-strip__badge steps-widget steps-widget--unconfigured"
        data-testid="steps-widget"
        title="Set GOOGLE_REFRESH_TOKEN in apps/desk/.env (see CLAUDE.md)"
      >
        <span className="kpi-strip__label">Steps</span>
        <span className="kpi-strip__value">connect google fit</span>
      </span>
    );
  }

  if (view && view.error_kind === "auth_revoked") {
    return (
      <span
        className="kpi-strip__badge steps-widget steps-widget--reconnect"
        data-testid="steps-widget"
        title="Google Fit refresh token revoked. Run: node apps/desk/scripts/google-fit-auth.cjs — then paste new GOOGLE_REFRESH_TOKEN to .env and restart."
      >
        <span className="kpi-strip__label">Steps</span>
        <span className="kpi-strip__value" data-testid="steps-reconnect">
          reconnect google fit
        </span>
      </span>
    );
  }

  const snap = view?.snapshot ?? null;
  const count = snap ? NUMBER_FORMATTER.format(snap.steps_today) : "—";
  const isStale = snap
    ? Date.now() - snap.fetched_at_ms > STALE_THRESHOLD_MS
    : false;
  const hasTransientError = view?.error_kind === "transient";
  const tooltip = hasTransientError
    ? `Last refresh failed: ${view?.error_message ?? "unknown error"}`
    : isStale
      ? "Last successful refresh > 1h ago (stale)"
      : "Steps today (Google Fit)";

  return (
    <span
      className={`kpi-strip__badge steps-widget${isStale ? " steps-widget--stale" : ""}`}
      data-testid="steps-widget"
      title={tooltip}
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
      {hasTransientError && (
        <span
          className="steps-widget__error-dot"
          data-testid="steps-error"
          aria-hidden
        >
          •
        </span>
      )}
    </span>
  );
};

export default StepsWidget;
