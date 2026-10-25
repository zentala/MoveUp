/**
 * AnalystLive.tsx — Production route at `/#/analyst`.
 *
 * Renders the Analyst window with NO fixture overrides — `CatalogTab` and
 * `ExplorerTab` both fall through to their live Tauri-invoke hooks.
 */
import { AnalystWindow } from "@/analyst/AnalystWindow";
import { localCalendarDay, shiftCalendarDay } from "@/analyst/open-day";
import { useLiveObservation } from "@/hooks/useLiveObservation";

function defaultRange(): { from: string; to: string } {
  const now = new Date();
  const to = localCalendarDay(now);
  const from = shiftCalendarDay(to, -6);
  return { from, to };
}

export default function AnalystLive() {
  const observation = useLiveObservation();
  const live = observation.liveObservation;

  return <div
    data-testid="analyst-live-observation"
    data-observation-quality={observation.observationQuality ?? undefined}
    data-observation-reason={live?.reason ?? undefined}
    data-span-state={live?.span?.state ?? undefined}
    data-span-started-at={live?.span?.started_at ?? undefined}
    data-observed-through={live?.span?.observed_through ?? undefined}
    data-last-observation-at={live?.last_observation_at ?? undefined}
    data-snapshot-at={live?.snapshot_at ?? undefined}
    data-refresh-loading={observation.refresh.loading}
    data-refresh-last-success-at={observation.refresh.lastSuccessAt ?? undefined}
    data-refresh-error={observation.refresh.error ?? undefined}
  >
    <AnalystWindow defaultRange={defaultRange()} />
  </div>;
}
