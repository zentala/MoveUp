/**
 * ChartCard.tsx — Common wrapper for analyst chart panels.
 *
 * Provides title, subtitle, and consistent padding/background.
 */
import type { ReactNode } from "react";
import { chartColors } from "./chart-utils";

export interface ChartCardProps {
  title: string;
  subtitle?: string;
  children: ReactNode;
}

export function ChartCard({ title, subtitle, children }: ChartCardProps) {
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
        <h3 style={{ margin: 0, fontSize: 14, fontWeight: 600 }}>{title}</h3>
        {subtitle && (
          <p style={{ margin: "2px 0 0", fontSize: 11, color: chartColors.subtext }}>{subtitle}</p>
        )}
      </header>
      <div>{children}</div>
    </section>
  );
}
