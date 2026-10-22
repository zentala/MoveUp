/**
 * ChartCard.tsx — Common wrapper for analyst chart panels.
 *
 * Provides title, subtitle, consistent padding/background, and an optional
 * `help` tooltip rendered as a `?` icon next to the title.
 */
import { useState, type KeyboardEvent, type ReactNode } from "react";
import { chartColors } from "./chart-utils";
import { CircleHelp } from "lucide-react";
import { UiIcon } from "@/components/UiIcon";

export interface ChartCardProps {
  title: string;
  subtitle?: string;
  /** Optional longer-form explanation. Shows on hover of a `?` icon next to the title. */
  help?: string;
  children: ReactNode;
}

const helpIconStyle: React.CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  width: 28,
  height: 28,
  borderRadius: 4,
  border: "1px solid transparent",
  background: "transparent",
  color: chartColors.subtext,
  marginLeft: 6,
  cursor: "help",
  fontWeight: 400,
  verticalAlign: "middle",
};

export function ChartCard({ title, subtitle, help, children }: ChartCardProps) {
  const [helpOpen, setHelpOpen] = useState(false);
  const closeOnEscape = (event: KeyboardEvent<HTMLButtonElement>) => {
    if (event.key === "Escape") { event.stopPropagation(); setHelpOpen(false); }
  };
  return (
    <section
      style={{
        background: chartColors.card,
        border: `1px solid ${chartColors.gridline}`,
        borderRadius: 8,
        padding: 12,
        color: chartColors.text,
      }}
    >
      <header style={{ marginBottom: 8 }}>
        <h3 style={{ margin: 0, fontSize: 14, fontWeight: 600, display: "flex", alignItems: "center" }}>
          {title}
          {help ? (
            <span style={{ position: "relative", display: "inline-flex" }} onMouseEnter={() => setHelpOpen(true)} onMouseLeave={() => setHelpOpen(false)}>
              <button
                type="button"
                style={helpIconStyle}
                aria-label={`Help: ${title}`}
                aria-describedby={helpOpen ? `chart-help-${title.replace(/[^a-z0-9]+/gi, "-").toLowerCase()}` : undefined}
                aria-expanded={helpOpen}
                onFocus={() => setHelpOpen(true)}
                onBlur={() => setHelpOpen(false)}
                onKeyDown={closeOnEscape}
                onClick={() => setHelpOpen(true)}
              >
                <UiIcon icon={CircleHelp} />
              </button>
              {helpOpen ? <span id={`chart-help-${title.replace(/[^a-z0-9]+/gi, "-").toLowerCase()}`} role="tooltip" style={{ position: "absolute", zIndex: 5, top: "100%", left: 0, width: 260, padding: 8, background: chartColors.card, border: `1px solid ${chartColors.gridline}`, borderRadius: 4, color: chartColors.text, fontSize: 11, fontWeight: 400 }}>
                {help}
              </span> : null}
            </span>
          ) : null}
        </h3>
        {subtitle && (
          <p style={{ margin: "2px 0 0", fontSize: 11, color: chartColors.subtext }}>{subtitle}</p>
        )}
      </header>
      <div>{children}</div>
    </section>
  );
}
