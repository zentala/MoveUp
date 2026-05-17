/**
 * AnalystMockup.tsx — Dev-only route rendering the Analyst window with fixtures.
 *
 * Accessible at /#/mockup/analyst in dev mode. No Tauri commands invoked —
 * the catalog fixture is adapted to the live `DataCatalog` wire shape and
 * passed explicitly so CatalogTab skips its `useDataCatalog()` fetch.
 */
import { AnalystWindow } from "@/analyst/AnalystWindow";
import { defaultRange } from "@/analyst/DateRangePicker";
import type { DataCatalog } from "@/analyst/types/catalog";
import {
  analystCatalogFixture,
  buildSnapshots,
  buildSessions,
  buildDailyKpis,
} from "@/test/analyst-fixtures";

// Recompute fixtures relative to TODAY at module init so the mockup stays
// useful indefinitely (the test exports stay anchored to a fixed date).
const NOW = new Date();
const mockupSnapshots = buildSnapshots(NOW);
const mockupSessions = buildSessions(NOW);
const mockupKpis = buildDailyKpis(NOW);
const mockupRange = defaultRange(NOW);

/**
 * Adapt the fixture (`CatalogSource[]`) to the live `DataCatalog` shape so
 * the mockup keeps rendering without a Tauri runtime. The fixture fields map
 * 1:1 except `description` / `sample_row` which the fixture lacks.
 */
const mockupCatalog: DataCatalog = {
  generated_at: new Date().toISOString(),
  sources: analystCatalogFixture.map((s) => ({
    id: s.id,
    name: s.name,
    kind: s.kind,
    location: s.location,
    retention: s.retention,
    fields: s.fields.map((f) => ({
      name: f.name,
      type: f.type,
      description: "",
    })),
    sample_row: null,
    description: "",
  })),
};

const bannerStyle: React.CSSProperties = {
  position: "sticky",
  top: 0,
  zIndex: 10,
  background: "#3b2a1a",
  color: "#ffb74d",
  borderBottom: "1px solid #5a3a1a",
  padding: "6px 16px",
  fontSize: 11,
  fontFamily: "system-ui, -apple-system, Segoe UI, sans-serif",
  letterSpacing: 0.4,
  textTransform: "uppercase",
  fontWeight: 600,
};

export default function AnalystMockup() {
  return (
    <>
      <div style={bannerStyle} data-testid="mockup-banner">
        Demo data · /#/mockup/analyst · No Tauri runtime, fixtures only
      </div>
      <AnalystWindow
        catalog={mockupCatalog}
        snapshots={mockupSnapshots}
        sessions={mockupSessions}
        kpis={mockupKpis}
        defaultRange={mockupRange}
      />
    </>
  );
}
