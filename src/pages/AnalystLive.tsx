/**
 * AnalystLive.tsx — Production route at `/#/analyst`.
 *
 * Renders the Analyst window with NO fixture overrides — `CatalogTab` and
 * `ExplorerTab` both fall through to their live Tauri-invoke hooks.
 */
import { AnalystWindow } from "@/analyst/AnalystWindow";

const DAY_MS = 86_400_000;

function defaultRange(): { from: string; to: string } {
  const now = new Date();
  const to = now.toISOString().slice(0, 10);
  const from = new Date(now.getTime() - 6 * DAY_MS).toISOString().slice(0, 10);
  return { from, to };
}

export default function AnalystLive() {
  return <AnalystWindow defaultRange={defaultRange()} />;
}
