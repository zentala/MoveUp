/**
 * registry.ts — static widget registry and resolver.
 *
 * Maps widget IDs to their React components. No dynamic loading —
 * all widgets are imported statically and registered here.
 */
import type { WidgetRegistration, DeskWidget } from "@/types";
import { PlaceholderWidget } from "./PlaceholderWidget";

/** All available widgets, in display order. */
export const WIDGET_REGISTRY: WidgetRegistration[] = [
  { id: "placeholder", name: "Placeholder (dev)", component: PlaceholderWidget },
];

/** Default widget ID when none is configured or config value is invalid. */
export const DEFAULT_WIDGET_ID = "placeholder";

/**
 * Resolves a widget ID to its component. Falls back to default if not found.
 * Logs a warning when falling back.
 */
export function resolveWidget(widgetId: string): DeskWidget {
  const entry = WIDGET_REGISTRY.find((w) => w.id === widgetId);
  if (entry) return entry.component;

  console.warn(
    `Widget "${widgetId}" not found in registry, falling back to "${DEFAULT_WIDGET_ID}"`,
  );
  const fallback = WIDGET_REGISTRY.find((w) => w.id === DEFAULT_WIDGET_ID);
  if (fallback) return fallback.component;

  // Should never happen — registry always has at least one entry
  return PlaceholderWidget;
}
