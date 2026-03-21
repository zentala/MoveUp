/**
 * useActiveWidget.ts — reads and persists the active widget ID from AppConfig.
 *
 * Uses Tauri IPC to get/save the active_widget setting. Falls back to
 * DEFAULT_WIDGET_ID when config is unavailable.
 */
import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { DEFAULT_WIDGET_ID } from "@/widgets/registry";

interface DeskSettings {
  active_widget?: string;
}

/**
 * Returns [activeWidgetId, setActiveWidgetId] tuple.
 * Reads from Tauri config on mount, persists on change.
 */
export function useActiveWidget(): [string, (id: string) => void] {
  const [widgetId, setWidgetId] = useState(DEFAULT_WIDGET_ID);

  useEffect(() => {
    invoke<DeskSettings>("get_settings")
      .then((settings) => {
        if (settings.active_widget) {
          console.info(`[widget] active_widget on startup: "${settings.active_widget}"`);
          setWidgetId(settings.active_widget);
        }
      })
      .catch(() => {
        // Config not available yet — use default
      });
  }, []);

  const setAndPersist = useCallback((id: string) => {
    console.info(`[widget] switching to: "${id}"`);
    setWidgetId(id);
    invoke("get_settings")
      .then((current) => {
        const updated = { ...(current as Record<string, unknown>), active_widget: id };
        return invoke("save_settings", { settings: updated });
      })
      .catch((err) => console.error("Failed to persist active_widget:", err));
  }, []);

  return [widgetId, setAndPersist];
}
