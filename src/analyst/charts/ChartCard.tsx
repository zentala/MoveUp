/**
 * ChartCard.tsx — Common wrapper for analyst chart panels.
 *
 * Provides title, subtitle, consistent padding/background, and an optional
 * `help` tooltip rendered as a `?` icon next to the title.
 */
import type { ReactNode } from "react";
import { chartColors } from "./chart-utils";

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
  width: 14,
  height: 14,
  borderRadius: "50%",
  border: `1px solid ${chartColors.gridline}`,
  color: chartColors.subtext,
  fontSize: 9,
  marginLeft: 6,
  cursor: "help",
  fontWeight: 400,
  verticalAlign: "middle",
};

export function ChartCard({ title, subtitle, help, children }: ChartCardProps) {
  return (
    <section
      style={{
        background: chartColors.card,
        border: "1px solid #2a2a3a",
        borderRadius: 8,
        padding: 12,
        color: chartColors.text,
      }}
    >
      <header style={{ marginBottom: 8 }}>
        <h3 style={{ margin: 0, fontSize: 14, fontWeight: 600 }}>
          {title}
          {help ? (
            <span
              style={helpIconStyle}
              role="img"
              aria-label={`Help: ${help}`}
              title={help}
            >
              ?
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
