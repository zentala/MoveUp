/**
 * TimelineDetailHeader.tsx — Caption strip above the TimelineDetail scroller.
 *
 * Since E018-T09 the page's date title lives in `AnalystHeader` at the top of
 * the Explorer tab, so this strip is demoted to a small in-context caption:
 * a subdued weekday/date label plus ‹ › buttons that nudge the strip without
 * making the reader travel back to the top of the page.
 */
import { chartColors } from "./chart-utils";
import { formatDate, weekdayName } from "@/utils/format";

export interface TimelineDetailHeaderProps {
  selectedDay: string;
  onPrev: () => void;
  onNext: () => void;
}

const navBtn: React.CSSProperties = {
  width: 22,
  height: 22,
  background: "transparent",
  border: `1px solid ${chartColors.gridline}`,
  color: chartColors.subtext,
  borderRadius: 4,
  cursor: "pointer",
  fontFamily: "inherit",
  fontSize: 12,
  lineHeight: 1,
};

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
      <div style={{ display: "flex", alignItems: "baseline", gap: 8 }}>
        <span
          data-testid="tl-weekday"
          style={{
            fontSize: 11,
            color: chartColors.subtext,
            letterSpacing: "0.12em",
            textTransform: "uppercase",
          }}
        >
          {weekdayName(selectedDay)}
        </span>
        <span
          style={{ fontSize: 11, color: chartColors.gridline, letterSpacing: "0.05em" }}
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
