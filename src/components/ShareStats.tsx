/**
 * ShareStats.tsx — "Share My Stats" modal for viral growth.
 *
 * Shows a preview card of today's desk stats with options to
 * copy formatted text to clipboard or share on social media.
 */
import { useState, useEffect, useRef, useId, type FC } from "react";
import { Check, Copy, ExternalLink, X } from "lucide-react";
import { UiIcon } from "./UiIcon";
import type { MetricSnapshot } from "@/types";
import { formatDurationShort } from "@/utils/format";
import "./share-stats.css";

/** Props for the ShareStats modal. */
interface ShareStatsProps {
  /** KPI metrics from the dashboard. */
  metrics: MetricSnapshot[];
  /** Today's position changes count. */
  todayChanges: number;
  /** Today's standing seconds. */
  todayStandingSecs: number;
  /** Today's sitting seconds. */
  todaySittingSecs: number;
  /** Daily score. */
  todayScore: number;
  /** Callback to close the modal. */
  onClose: () => void;
  /** Protected-flow release failure, kept inside the modal while it remains open. */
  closeError?: string | null;
  /** Retries releasing the protected Share flow. */
  onRetryClose?: () => void;
}

/** Extracts a metric display value by ID, or returns a fallback. */
function metricDisplay(metrics: MetricSnapshot[], id: string): string {
  const m = metrics.find((x) => x.id === id);
  return m?.result.display ?? "\u2014";
}

/** Extracts a metric level by ID. */
function metricLevel(metrics: MetricSnapshot[], id: string): string {
  const m = metrics.find((x) => x.id === id);
  return m?.result.level ?? "green";
}

/** Builds the plain-text version for clipboard/social sharing. */
function buildShareText(
  metrics: MetricSnapshot[],
  todayChanges: number,
  todayScore: number,
): string {
  const standing = metricDisplay(metrics, "standing_pct");
  const longest = metricDisplay(metrics, "longest_session");
  const sign = todayScore >= 0 ? "+" : "";

  return [
    "My desk stats today:",
    `\u{1f9cd} Standing: ${standing}`,
    `\u{1f504} Position changes: ${todayChanges}`,
    `\u{23f1}\u{fe0f} Longest session: ${longest}`,
    `\u{1f3c6} Score: ${sign}${Math.round(todayScore)} points`,
    "",
    "Tracked by MoveUp \u{2014} desk.zentala.io",
  ].join("\n");
}

const DESK_URL = "https://desk.zentala.io";

/** Builds a Twitter/X intent URL with pre-filled text. */
function twitterShareUrl(text: string): string {
  return `https://twitter.com/intent/tweet?text=${encodeURIComponent(text)}&url=${encodeURIComponent(DESK_URL)}`;
}

/** Builds a Reddit submit URL with pre-filled title. */
function redditShareUrl(text: string): string {
  const title = "My desk ergonomics stats today \u{2014} tracked by MoveUp";
  return `https://reddit.com/submit?title=${encodeURIComponent(title)}&url=${encodeURIComponent(DESK_URL)}&selftext=true&text=${encodeURIComponent(text)}`;
}

/** Level-to-CSS-variable mapping for colored stat values. */
const LEVEL_COLORS: Record<string, string> = {
  green: "var(--signal-ok)",
  yellow: "var(--signal-warn)",
  red: "var(--signal-alert)",
};

/** A single stat row in the preview card. */
const StatRow: FC<{ label: string; value: string; level?: string }> = ({
  label,
  value,
  level = "green",
}) => (
  <div className="share-card__row">
    <span className="share-card__label">{label}</span>
    <span
      className="share-card__value"
      style={{ color: LEVEL_COLORS[level] ?? LEVEL_COLORS.green }}
    >
      {value}
    </span>
  </div>
);

/** Share My Stats modal overlay. */
export const ShareStats: FC<ShareStatsProps> = ({
  metrics,
  todayChanges,
  todayStandingSecs,
  todaySittingSecs,
  todayScore,
  onClose,
  closeError,
  onRetryClose,
}) => {
  const [copied, setCopied] = useState(false);
  const [copyError, setCopyError] = useState(false);
  const dialogRef = useRef<HTMLDialogElement>(null);
  const titleId = useId();
  useEffect(() => {
    const dialog = dialogRef.current!;
    dialog.showModal();
    return () => dialog.close();
  }, []);
  const shareText = buildShareText(metrics, todayChanges, todayScore);

  const standingPct = metricDisplay(metrics, "standing_pct");
  const standingLevel = metricLevel(metrics, "standing_pct");
  const longestSession = metricDisplay(metrics, "longest_session");
  const longestLevel = metricLevel(metrics, "longest_session");
  const totalTime = formatDurationShort(todaySittingSecs + todayStandingSecs);
  const scoreSign = todayScore >= 0 ? "+" : "";

  /** Copies share text to clipboard using the Clipboard API. */
  async function handleCopy() {
    setCopied(false);
    setCopyError(false);
    try {
      await navigator.clipboard.writeText(shareText);
      setCopied(true);
    } catch {
      // Fallback: select a textarea and attempt execCommand
      try {
        const ta = document.createElement("textarea");
        ta.value = shareText;
        const previousFocus = document.activeElement as HTMLElement | null;
        dialogRef.current!.appendChild(ta);
        let ok = false;
        try { ta.select(); ok = document.execCommand("copy"); }
        finally { ta.remove(); previousFocus?.focus(); }
        if (ok) setCopied(true);
        else setCopyError(true);
      } catch {
        setCopyError(true);
      }
    }
  }

  /** Opens a share URL in the default browser via Tauri shell. */
  function openUrl(url: string) {
    // In Tauri, open() from @tauri-apps/plugin-shell isn't always available.
    // Use window.open as fallback — Tauri intercepts external URLs.
    window.open(url, "_blank");
  }

  // Reset "Copied!" after 2 seconds
  useEffect(() => {
    if (!copied) return;
    const t = setTimeout(() => setCopied(false), 2000);
    return () => clearTimeout(t);
  }, [copied]);

  return (
    <div className="share-overlay" onClick={onClose}>
      <dialog
        ref={dialogRef}
        className="share-modal"
        aria-labelledby={titleId}
        onCancel={(event) => { event.preventDefault(); onClose(); }}
        onKeyDown={(event) => {
          if (event.key !== "Tab") return;
          const stops = event.currentTarget.querySelectorAll<HTMLElement>("button, [tabindex='0']");
          const first = stops[0], last = stops[stops.length - 1];
          if (event.shiftKey && document.activeElement === first) {
            event.preventDefault(); last.focus();
          } else if (!event.shiftKey && document.activeElement === last) {
            event.preventDefault(); first.focus();
          }
        }}
        onClick={(event) => {
          event.stopPropagation();
          if (event.target !== event.currentTarget) return;
          const box = event.currentTarget.getBoundingClientRect();
          if (event.clientX < box.left || event.clientX > box.right ||
              event.clientY < box.top || event.clientY > box.bottom) onClose();
        }}
      >
        <div className="share-modal__header">
          <h2 id={titleId} className="share-modal__title">share my stats</h2>
          <button autoFocus type="button" className="share-modal__close" onClick={onClose} title="Close" aria-label="Close">
            <UiIcon icon={X} />
          </button>
        </div>

        {closeError && (
          <div className="share-modal__close-error">
            <p role="alert" data-testid="share-close-error">{closeError}</p>
            {onRetryClose && <button type="button" className="share-actions__btn share-modal__retry-close" onClick={onRetryClose}>Retry close</button>}
          </div>
        )}

        {copyError && (
          <p className="share-modal__copy-error" role="alert" data-testid="copy-error">
            Copy failed.
          </p>
        )}

        {/* Preview card */}
        <section className="share-card" aria-label="Today's share statistics" tabIndex={0}>
          <div className="share-card__time">{totalTime} tracked</div>
          <div className="share-card__stats">
            <StatRow label="Standing" value={standingPct} level={standingLevel} />
            <StatRow label="Position changes" value={String(todayChanges)} />
            <StatRow label="Longest session" value={longestSession} level={longestLevel} />
            <StatRow label="Score" value={`${scoreSign}${Math.round(todayScore)} pts`} />
          </div>
          <div className="share-card__watermark">
            Tracked by MoveUp {"\u2014"} desk.zentala.io
          </div>
          {copyError && (
            <div className="share-card__copy-error">
              <p className="share-card__copy-guidance">Automatic copy failed. Retry or select the text below and copy it.</p>
              <button type="button" className="share-actions__btn share-card__retry" onClick={() => void handleCopy()}>
                Retry copy
              </button>
              <label className="share-card__manual-label" htmlFor="share-manual-copy">
                Share text
              </label>
              <textarea
                id="share-manual-copy"
                className="share-card__manual-copy"
                aria-label="Share text to copy manually"
                readOnly
                rows={3}
                value={shareText}
                onFocus={event => event.currentTarget.select()}
              />
            </div>
          )}
        </section>

        {/* Action buttons */}
        <div className="share-actions">
          <button type="button" className="share-actions__btn share-actions__btn--copy" onClick={handleCopy}>
            <UiIcon icon={copied ? Check : Copy} />
            <span aria-live="polite">{copied ? "Copied!" : "Copy Text"}</span>
          </button>
          <button
            className="share-actions__btn share-actions__btn--twitter"
            type="button"
            onClick={() => openUrl(twitterShareUrl(shareText))}
          >
            <UiIcon icon={ExternalLink} /><span>Post on X</span>
          </button>
          <button
            className="share-actions__btn share-actions__btn--reddit"
            type="button"
            onClick={() => openUrl(redditShareUrl(shareText))}
          >
            <UiIcon icon={ExternalLink} /><span>Reddit</span>
          </button>
        </div>
      </dialog>
    </div>
  );
};
