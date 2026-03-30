/**
 * TimelineSkinSection.tsx — dropdown to select the timeline color skin.
 *
 * Reads/writes the timeline_skin setting via the useTimelineSkin hook.
 */
import type { FC } from "react";
import { useTimelineSkin, TIMELINE_SKINS } from "@/hooks/useTimelineSkin";
import type { TimelineSkinId } from "@/hooks/useTimelineSkin";

/** Settings section for choosing the timeline color skin. */
const TimelineSkinSection: FC = () => {
  const [skinId, setSkinId] = useTimelineSkin();

  return (
    <div className="settings-panel__section">
      <h3 className="settings-panel__section-title">Timeline Theme</h3>
      <div className="settings-panel__field">
        <label htmlFor="timeline-skin" className="settings-panel__label">
          Color skin
        </label>
        <select
          id="timeline-skin"
          className="settings-panel__select"
          value={skinId}
          onChange={(e) => setSkinId(e.target.value as TimelineSkinId)}
          data-testid="timeline-skin-picker"
        >
          {TIMELINE_SKINS.map((s) => (
            <option key={s.id} value={s.id}>
              {s.name} — {s.description}
            </option>
          ))}
        </select>
      </div>
    </div>
  );
};

export default TimelineSkinSection;
