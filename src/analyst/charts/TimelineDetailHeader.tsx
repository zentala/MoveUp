/**
 * TimelineDetailHeader.tsx — Header strip above TimelineDetail.
 *
 * Shows the weekday + formatted date for the active day plus ‹ › nav
 * buttons. Layout split from TimelineDetail.tsx to respect the
 * 250-line-per-file cap.
 */
import { chartColors } from "./chart-utils";

export interface TimelineDetailHeaderProps {
  selectedDay: string;
  onPrev: () => void;
  onNext: () => void;
}

const navBtn: React.CSSProperties = {
  width: 26,
  height: 26,
  background: "transparent",
  border: `1px solid ${chartColors.gridline}`,
  color: chartColors.subtext,
  borderRadius: 4,
  cursor: "pointer",
  fontFamily: "inherit",
  fontSize: 15,
};

function formatDate(date: string): string {
  const d = new Date(`${date}T00:00:00`);
  return `${date.slice(8, 10)} ${d.toLocaleDateString("en-US", {
    month: "short",
  })} ${date.slice(0, 4)}`;
}

function weekdayName(date: string): string {
  const d = new Date(`${date}T00:00:00`);
  return d.toLocaleDateString("en-US", { weekday: "long" });
}

export function TimelineDetailHeader({
  selectedDay,
  onPrev,
  onNext,
}: TimelineDetailHeaderProps) {
  return (
    <div
      style={{
        display: "flex",
        justifyContent: "space-between",
        alignItems: "baseline",
        gap: 16,
        marginBottom: 10,
        flexWrap: "wrap",
      }}
    >
      <div style={{ display: "flex", alignItems: "baseline", gap: 12 }}>
        <span
          data-testid="tl-weekday"
          style={{ fontSize: 24, color: chartColors.text, fontStyle: "italic" }}
        >
          {weekdayName(selectedDay)}
        </span>
        <span
          style={{ fontSize: 12, color: chartColors.subtext, letterSpacing: "0.05em" }}
        >
          {formatDate(selectedDay)}
        </span>
      </div>
      <div style={{ display: "inline-flex", gap: 4 }}>
        <button
          type="button"
          onClick={onPrev}
          title="Previous day"
          style={navBtn}
          data-testid="tl-prev"
        >
          ‹
        </button>
        <button
          type="button"
          onClick={onNext}
          title="Next day"
          style={navBtn}
          data-testid="tl-next"
        >
          ›
        </button>
      </div>
    </div>
  );
}
