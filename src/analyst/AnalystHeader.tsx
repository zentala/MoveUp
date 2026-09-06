/**
 * AnalystHeader.tsx — the "what am I looking at" line at the top of the
 * Analyst Explorer tab.
 *
 * Shows a centred `◀ Tuesday · 17 May 2026 ▶` title. The arrows drive the
 * whole page (timeline strip, KPI panels, day navigator), which is why this
 * lives at the tab level instead of inside `TimelineDetail` — the nav is no
 * longer the timeline's own chrome.
 */
import { chartColors } from "./charts/chart-utils";
import { formatDate, weekdayName } from "@/utils/format";

export interface AnalystHeaderProps {
  /** Active day, `YYYY-MM-DD`. */
  selectedDay: string;
  onPrev: () => void;
  onNext: () => void;
}

const navBtn: React.CSSProperties = {
  width: 34,
  height: 34,
  background: "transparent",
  border: `1px solid ${chartColors.gridline}`,
  color: chartColors.subtext,
  borderRadius: 6,
  cursor: "pointer",
  fontFamily: "inherit",
  fontSize: 14,
  lineHeight: 1,
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
};

export function AnalystHeader({
  selectedDay,
  onPrev,
  onNext,
}: AnalystHeaderProps) {
  return (
    <header
      data-testid="analyst-header"
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        gap: 20,
        padding: "2px 0 16px",
      }}
    >
      <button
        type="button"
        onClick={onPrev}
        title="Previous day"
        aria-label="Previous day"
        data-testid="analyst-header-prev"
        style={navBtn}
      >
        ◀
      </button>
      <h2
        data-testid="analyst-header-title"
        style={{
          margin: 0,
          fontSize: 22,
          fontWeight: 500,
          color: chartColors.text,
          textAlign: "center",
          letterSpacing: "0.01em",
          whiteSpace: "nowrap",
        }}
      >
        <span data-testid="analyst-header-weekday" style={{ fontStyle: "italic" }}>
          {weekdayName(selectedDay)}
        </span>
        <span style={{ color: chartColors.subtext, margin: "0 10px" }}>·</span>
        <span
          data-testid="analyst-header-date"
          style={{ fontSize: 18, color: chartColors.subtext }}
        >
          {formatDate(selectedDay)}
        </span>
      </h2>
      <button
        type="button"
        onClick={onNext}
        title="Next day"
        aria-label="Next day"
        data-testid="analyst-header-next"
        style={navBtn}
      >
        ▶
      </button>
    </header>
  );
}
