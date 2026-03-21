/**
 * WidgetPickerSection.tsx — dropdown to select the active widget layout.
 *
 * Reads/writes the active_widget setting via the useActiveWidget hook.
 * Lists all registered widgets from the static registry.
 */
import type { FC } from "react";
import { useActiveWidget } from "@/hooks/useActiveWidget";
import { WIDGET_REGISTRY } from "@/widgets/registry";

/** Settings section for choosing the active widget. */
const WidgetPickerSection: FC = () => {
  const [activeWidgetId, setActiveWidgetId] = useActiveWidget();

  return (
    <div className="settings-panel__section">
      <h3 className="settings-panel__section-title">Widget Layout</h3>
      <div className="settings-panel__field">
        <label htmlFor="widget-picker" className="settings-panel__label">
          Active widget
        </label>
        <select
          id="widget-picker"
          className="settings-panel__select"
          value={activeWidgetId}
          onChange={(e) => setActiveWidgetId(e.target.value)}
          data-testid="widget-picker"
        >
          {WIDGET_REGISTRY.map((w) => (
            <option key={w.id} value={w.id}>
              {w.name}
            </option>
          ))}
        </select>
      </div>
    </div>
  );
};

export default WidgetPickerSection;
