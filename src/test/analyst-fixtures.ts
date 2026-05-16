/**
 * analyst-fixtures.ts — Fake data for Analyst Window mockup (E012-T03).
 *
 * Realistic 7-day fake data covering snapshots, events, sessions, and
 * the data source catalog. Used only by the /mockup/analyst route and
 * the analyst chart unit tests.
 *
 * Implementation is split across:
 *   - analyst-fixtures-types.ts      (types + helpers)
 *   - analyst-fixtures-catalog.ts    (static catalog)
 *   - analyst-fixtures-builders.ts   (synthetic data builders)
 */
import {
  buildDailyKpis,
  buildEvents,
  buildSessions,
  buildSnapshots,
} from "./analyst-fixtures-builders";
import { dateLocal } from "./analyst-fixtures-types";

export type {
  CatalogSource,
  DailyKpi,
  EventRow,
  SessionRow,
  SnapshotRow,
} from "./analyst-fixtures-types";
export { analystCatalogFixture } from "./analyst-fixtures-catalog";
export {
  buildDailyKpis,
  buildEvents,
  buildSessions,
  buildSnapshots,
} from "./analyst-fixtures-builders";

const TODAY = new Date("2026-05-16T12:00:00Z");

export const analystSnapshotsFixture = buildSnapshots(TODAY);
export const analystEventsFixture = buildEvents(TODAY);
export const analystSessionsFixture = buildSessions(TODAY);
export const analystDailyKpisFixture = buildDailyKpis(TODAY);

/** Default mockup date range — last 7 days. */
export const analystDefaultRange = {
  from: dateLocal(TODAY, 6),
  to: dateLocal(TODAY, 0),
};
