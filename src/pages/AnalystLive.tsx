/**
 * AnalystLive.tsx — Production route at `/#/analyst`.
 *
 * Renders the Analyst window with NO fixture overrides — `CatalogTab` and
 * `ExplorerTab` both fall through to their live Tauri-invoke hooks.
 */
import { AnalystWindow } from "@/analyst/AnalystWindow";
import { localCalendarDay, shiftCalendarDay } from "@/analyst/open-day";

function defaultRange(): { from: string; to: string } {
  const now = new Date();
  const to = localCalendarDay(now);
  const from = shiftCalendarDay(to, -6);
  return { from, to };
}

export default function AnalystLive() {
  return <AnalystWindow defaultRange={defaultRange()} />;
}
