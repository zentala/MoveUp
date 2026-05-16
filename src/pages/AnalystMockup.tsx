/**
 * AnalystMockup.tsx — Dev-only route rendering the Analyst window with fixtures.
 *
 * Accessible at /#/mockup/analyst in dev mode. No Tauri commands invoked.
 */
import { AnalystWindow } from "@/analyst/AnalystWindow";
import {
  analystCatalogFixture,
  analystSnapshotsFixture,
  analystSessionsFixture,
  analystDailyKpisFixture,
  analystDefaultRange,
} from "@/test/analyst-fixtures";

export default function AnalystMockup() {
  return (
    <AnalystWindow
      sources={analystCatalogFixture}
      snapshots={analystSnapshotsFixture}
      sessions={analystSessionsFixture}
      kpis={analystDailyKpisFixture}
      defaultRange={analystDefaultRange}
    />
  );
}
