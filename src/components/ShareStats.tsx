/**
 * ShareStats.tsx — "Share My Stats" modal for viral growth.
 *
 * Shows a preview card of today's desk stats with options to
 * copy formatted text to clipboard or share on social media.
 */
import { useState, useEffect, type FC } from "react";
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
    "Tracked by Smart Desk \u{2014} desk.zentala.io",
  ].join("\n");
}

const DESK_URL = "https://desk.zentala.io";

/** Builds a Twitter/X intent URL with pre-filled text. */
function twitterShareUrl(text: string): string {
  return `https://twitter.com/intent/tweet?text=${encodeURIComponent(text)}&url=${encodeURIComponent(DESK_URL)}`;
}

/** Builds a Reddit submit URL with pre-filled title. */
function redditShareUrl(text: string): string {
  const title = "My desk ergonomics stats today \u{2014} tracked by Smart Desk";
  return `https://reddit.com/submit?title=${encodeURIComponent(title)}&url=${encodeURIComponent(DESK_URL)}&selftext=true&text=${encodeURIComponent(text)}`;
}

/** Level-to-CSS-variable mapping for colored stat values. */
const LEVEL_COLORS: Record<string, string> = {
  green: "var(--signal-ok)",
  yellow: "var(--signal-warn)",
  red: "var(--signal-alert)",
};

/** A single stat row in the preview card. */
const StatRow: FC<{ emoji: string; label: string; value: string; level?: string }> = ({
  emoji,
  label,
  value,
  level = "green",
}) => (
  <div className="share-card__row">
    <span className="share-card__emoji">{emoji}</span>
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
}) => {
  const [copied, setCopied] = useState(false);
  const shareText = buildShareText(metrics, todayChanges, todayScore);

  const standingPct = metricDisplay(metrics, "standing_pct");
  const standingLevel = metricLevel(metrics, "standing_pct");
  const longestSession = metricDisplay(metrics, "longest_session");
  const longestLevel = metricLevel(metrics, "longest_session");
  const totalTime = formatDurationShort(todaySittingSecs + todayStandingSecs);
  const scoreSign = todayScore >= 0 ? "+" : "";

  /** Copies share text to clipboard using the Clipboard API. */
  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(shareText);
      setCopied(true);
    } catch {
      // Fallback: select a textarea and attempt execCommand
      try {
        const ta = document.createElement("textarea");
        ta.value = shareText;
        document.body.appendChild(ta);
        ta.select();
        const ok = document.execCommand("copy");
        document.body.removeChild(ta);
        if (ok) {
          setCopied(true);
        } else {
          console.warn("Copy failed — select and copy manually");
        }
      } catch {
        console.warn("Copy failed — select and copy manually");
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
      <div className="share-modal" onClick={(e) => e.stopPropagation()}>
        <div className="share-modal__header">
          <span className="share-modal__title">share my stats</span>
          <button className="share-modal__close" onClick={onClose} title="Close">
            {"\u00d7"}
          </button>
        </div>

        {/* Preview card */}
        <div className="share-card">
          <div className="share-card__header">My Desk Stats Today</div>
          <div className="share-card__time">{totalTime} tracked</div>
          <div className="share-card__stats">
            <StatRow emoji={"\u{1f9cd}"} label="Standing" value={standingPct} level={standingLevel} />
            <StatRow emoji={"\u{1f504}"} label="Position changes" value={String(todayChanges)} />
            <StatRow emoji={"\u{23f1}\u{fe0f}"} label="Longest session" value={longestSession} level={longestLevel} />
            <StatRow emoji={"\u{1f3c6}"} label="Score" value={`${scoreSign}${Math.round(todayScore)} pts`} />
          </div>
          <div className="share-card__watermark">
            Tracked by Smart Desk {"\u2014"} desk.zentala.io
          </div>
        </div>

        {/* Action buttons */}
        <div className="share-actions">
          <button className="share-actions__btn share-actions__btn--copy" onClick={handleCopy}>
            {copied ? "\u2713 Copied!" : "\u{1f4cb} Copy Text"}
          </button>
          <button
            className="share-actions__btn share-actions__btn--twitter"
            onClick={() => openUrl(twitterShareUrl(shareText))}
          >
            {"\u{1d54f}"} Post on X
          </button>
          <button
            className="share-actions__btn share-actions__btn--reddit"
            onClick={() => openUrl(redditShareUrl(shareText))}
          >
            Share on Reddit
          </button>
        </div>
      </div>
    </div>
  );
};
