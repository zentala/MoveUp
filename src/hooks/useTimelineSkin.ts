/**
 * useTimelineSkin.ts — reads and persists the timeline skin preference.
 *
 * Uses Tauri IPC (same pattern as useActiveWidget) to get/save the
 * timeline_skin setting. Falls back to "semantic" when unavailable.
 */
import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export const TIMELINE_SKINS = [
  { id: "semantic", name: "Semantic", description: "Distinct hues per state" },
  { id: "amber", name: "Amber", description: "Warm instrument panel palette" },
  { id: "clinical", name: "Clinical", description: "Cool data-viz palette" },
] as const;

export type TimelineSkinId = (typeof TIMELINE_SKINS)[number]["id"];

export const DEFAULT_TIMELINE_SKIN: TimelineSkinId = "semantic";

/** CSS class name for the active skin. */
export function skinClassName(skin: TimelineSkinId): string {
  return `tl-skin-${skin}`;
}

interface DeskSettings {
  timeline_skin?: string;
}

/**
 * Returns [skinId, setSkinId] tuple.
 * Reads from Tauri config on mount, persists on change.
 */
export function useTimelineSkin(): [TimelineSkinId, (id: TimelineSkinId) => void] {
  const [skin, setSkin] = useState<TimelineSkinId>(DEFAULT_TIMELINE_SKIN);

  useEffect(() => {
    invoke<DeskSettings>("get_settings")
      .then((settings) => {
        if (settings.timeline_skin && isValidSkin(settings.timeline_skin)) {
          setSkin(settings.timeline_skin as TimelineSkinId);
        }
      })
      .catch(() => {
        // Config not available yet — use default
      });
  }, []);

  const setAndPersist = useCallback((id: TimelineSkinId) => {
    setSkin(id);
    invoke("get_settings")
      .then((current) => {
        const updated = { ...(current as Record<string, unknown>), timeline_skin: id };
        return invoke("save_settings", { settings: updated });
      })
      .catch((err) => console.error("Failed to persist timeline_skin:", err));
  }, []);

  return [skin, setAndPersist];
}

function isValidSkin(value: string): value is TimelineSkinId {
  return TIMELINE_SKINS.some((s) => s.id === value);
}
