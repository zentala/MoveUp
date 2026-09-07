/**
 * HealthWidget.tsx — compact "health today" badge for the OneBar popup.
 *
 * Was `StepsWidget` until E021-T04: the source is no longer Google Fit
 * specifically, so the badge names the metric, not the vendor. It renders
 * whatever `HealthView` the active source produced — steps always, heart
 * rate when a source supplies it — and labels the source by `source_id`.
 *
 * Visual language matches the KPI strip: borderless badge with small label
 * + bold value. Slotted INSIDE KpiStrip so it participates in the same
 * flex-wrap flow.
 *
 * Render states:
 *   - remote display     → same data as desktop, minus the refresh button
 *   - not configured     → muted "connect health source" hint
 *   - configured, no snapshot yet → "—"
 *   - snapshot fresh (<1h) → step count (+ HR badge) + refresh button
 *   - snapshot stale (>1h) → value dimmed, "(stale)" in the tooltip
 *   - auth_revoked       → "reconnect google fit" CTA (tooltip explains how)
 *   - transient error    → red dot indicator next to the value
 *
 * Transport, refresh cadence and concurrency all live in `useHealth`.
 */
import { useEffect, useState, type FC } from "react";
import { useHealth } from "@/hooks/useHealth";

const STALE_THRESHOLD_MS = 60 * 60 * 1000;
/** How often the staleness check re-evaluates against the current time. */
const STALENESS_TICK_MS = 60 * 1000;

const NUMBER_FORMATTER = new Intl.NumberFormat(
  typeof navigator !== "undefined" ? navigator.language : "en-US",
);

/** Source whose upstream API has an announced end-of-life. */
const SUNSETTING_SOURCE_ID = "google_fit";
const SUNSET_HINT = "Google Fit ends late 2026";

/** Human label for a source id; unknown ids show verbatim. */
const SOURCE_LABELS: Record<string, string> = {
  google_fit: "Google Fit",
  push: "phone",
};

function sourceLabel(sourceId: string): string {
  return SOURCE_LABELS[sourceId] ?? sourceId;
}

export const HealthWidget: FC = () => {
  const { view, refreshing, refresh, canRefresh } = useHealth();
  const [nowMs, setNowMs] = useState(() => Date.now());

  // Re-evaluate staleness periodically instead of reading Date.now() during
  // render (which would make the render impure).
  useEffect(() => {
    const id = setInterval(() => setNowMs(Date.now()), STALENESS_TICK_MS);
    return () => clearInterval(id);
  }, []);

  // ─── Render branches ────────────────────────────────────────────────

  if (view && !view.configured) {
    return (
      <span
        className="health-widget kpi-strip__badge health-widget--unconfigured"
        data-testid="health-widget"
        title="No health source set up. Set GOOGLE_REFRESH_TOKEN in apps/desk/.env, or push readings to POST /display/health."
      >
        <span className="kpi-strip__label">Health</span>
        <span className="kpi-strip__value">connect health source</span>
      </span>
    );
  }

  if (view && view.error_kind === "auth_revoked") {
    return (
      <span
        className="health-widget kpi-strip__badge health-widget--reconnect"
        data-testid="health-widget"
        title="Google Fit refresh token revoked. Run: node apps/desk/scripts/google-fit-auth.cjs — then paste new GOOGLE_REFRESH_TOKEN to .env and restart."
      >
        <span className="kpi-strip__label">Health</span>
        <span className="kpi-strip__value" data-testid="health-reconnect">
          reconnect google fit
        </span>
      </span>
    );
  }

  const snap = view?.snapshot ?? null;
  const count = snap ? NUMBER_FORMATTER.format(snap.steps_today) : "—";
  const isStale = snap ? nowMs - snap.fetched_at_ms > STALE_THRESHOLD_MS : false;
  const hasTransientError = view?.error_kind === "transient";
  // The snapshot carries exactly one source, so a Google Fit snapshot IS the
  // only source feeding this view — that is when the sunset hint applies.
  const isSunsettingSource = snap?.source_id === SUNSETTING_SOURCE_ID;

  const statusTooltip = hasTransientError
    ? `Last refresh failed: ${view?.error_message ?? "unknown error"}`
    : isStale
      ? "Last successful refresh > 1h ago (stale)"
      : snap
        ? `Steps today (${sourceLabel(snap.source_id)})`
        : "Steps today";
  // The sunset hint is a fact about the source, not about this reading, so
  // it rides along with whatever the current status happens to be.
  const tooltip = isSunsettingSource
    ? `${statusTooltip} — ${SUNSET_HINT}`
    : statusTooltip;

  return (
    <span
      className={`health-widget kpi-strip__badge${isStale ? " health-widget--stale" : ""}`}
      data-testid="health-widget"
      title={tooltip}
    >
      <span className="kpi-strip__label">Steps</span>
      <span className="kpi-strip__value" data-testid="health-steps">
        {count}
      </span>
      {snap?.heart_rate_bpm != null && (
        <span
          className="health-widget__hr"
          data-testid="health-hr"
          title="Heart rate (bpm)"
        >
          {"♥"} {Math.round(snap.heart_rate_bpm)}
        </span>
      )}
      {snap && (
        <span className="health-widget__source" data-testid="health-source">
          {sourceLabel(snap.source_id)}
        </span>
      )}
      {canRefresh && (
        <button
          className="health-widget__refresh"
          onClick={() => void refresh()}
          disabled={refreshing}
          aria-label="Refresh health"
          title="Refresh now"
        >
          {refreshing ? "…" : "↻"}
        </button>
      )}
      {hasTransientError && (
        <span
          className="health-widget__error-dot"
          data-testid="health-error"
          aria-hidden
        >
          •
        </span>
      )}
    </span>
  );
};

export default HealthWidget;
