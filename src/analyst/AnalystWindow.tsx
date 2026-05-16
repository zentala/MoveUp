/**
 * AnalystWindow.tsx — Tabbed shell for the Analyst dashboard.
 *
 * Renders header + tab triggers + active tab content. Pure mockup —
 * fixtures are injected via props; no Tauri invokes.
 */
import { useState } from "react";
import type {
  CatalogSource,
  DailyKpi,
  SessionRow,
  SnapshotRow,
} from "@/test/analyst-fixtures";
import { CatalogTab } from "./CatalogTab";
import { ExplorerTab } from "./ExplorerTab";
import type { DateRange } from "./DateRangePicker";
import { chartColors } from "./charts/chart-utils";

export interface AnalystWindowProps {
  sources: CatalogSource[];
  snapshots: SnapshotRow[];
  sessions: SessionRow[];
  kpis: DailyKpi[];
  defaultRange: DateRange;
}

type TabId = "catalog" | "explorer";

const tabButton = (active: boolean): React.CSSProperties => ({
  background: "transparent",
  border: "none",
  color: active ? chartColors.primary : chartColors.subtext,
  borderBottom: `2px solid ${active ? chartColors.primary : "transparent"}`,
  padding: "8px 16px",
  fontSize: 13,
  cursor: "pointer",
  fontFamily: "inherit",
});

export function AnalystWindow({
  sources,
  snapshots,
  sessions,
  kpis,
  defaultRange,
}: AnalystWindowProps) {
  const [tab, setTab] = useState<TabId>("explorer");
  const [range, setRange] = useState<DateRange>(defaultRange);

  return (
    <div
      style={{
        minWidth: 1280,
        minHeight: "100vh",
        background: chartColors.background,
        color: chartColors.text,
        fontFamily: "system-ui, -apple-system, Segoe UI, sans-serif",
        padding: 16,
        boxSizing: "border-box",
      }}
      data-testid="analyst-window"
    >
      <header style={{ marginBottom: 16 }}>
        <h1 style={{ margin: 0, fontSize: 20, fontWeight: 600 }}>Analyst</h1>
        <p style={{ margin: "2px 0 0", fontSize: 12, color: chartColors.subtext }}>
          Inspect every data source the desk app produces, then explore it.
        </p>
      </header>
      <nav
        role="tablist"
        style={{ borderBottom: `1px solid ${chartColors.gridline}`, marginBottom: 16 }}
      >
        <button
          role="tab"
          aria-selected={tab === "catalog"}
          style={tabButton(tab === "catalog")}
          onClick={() => setTab("catalog")}
        >
          Catalog
        </button>
        <button
          role="tab"
          aria-selected={tab === "explorer"}
          style={tabButton(tab === "explorer")}
          onClick={() => setTab("explorer")}
        >
          Explorer
        </button>
      </nav>
      {tab === "catalog" ? (
        <CatalogTab sources={sources} />
      ) : (
        <ExplorerTab
          snapshots={snapshots}
          sessions={sessions}
          kpis={kpis}
          range={range}
          onRangeChange={setRange}
        />
      )}
    </div>
  );
}
