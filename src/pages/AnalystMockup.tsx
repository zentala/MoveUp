/**
 * AnalystMockup.tsx — Dev-only route rendering the Analyst window with fixtures.
 *
 * Accessible at /#/mockup/analyst in dev mode. No Tauri commands invoked —
 * the catalog fixture is adapted to the live `DataCatalog` wire shape and
 * passed explicitly so CatalogTab skips its `useDataCatalog()` fetch.
 */
import { AnalystWindow } from "@/analyst/AnalystWindow";
import type { DataCatalog } from "@/analyst/types/catalog";
import {
  analystCatalogFixture,
  analystSnapshotsFixture,
  analystSessionsFixture,
  analystDailyKpisFixture,
  analystDefaultRange,
} from "@/test/analyst-fixtures";

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

export default function AnalystMockup() {
  return (
    <AnalystWindow
      catalog={mockupCatalog}
      snapshots={analystSnapshotsFixture}
      sessions={analystSessionsFixture}
      kpis={analystDailyKpisFixture}
      defaultRange={analystDefaultRange}
    />
  );
}
