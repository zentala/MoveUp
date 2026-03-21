/**
 * temperature.ts — computes CSS temperature class based on state and limit ratio.
 *
 * Temperature escalation drives the entire widget's visual theme:
 * background, border color, and glow intensity change together.
 */
import type { WidgetProps } from "@/types";

/** Visual temperature stages, from calm to burning. */
export type Temperature =
  | "calm"
  | "warm"
  | "hot"
  | "burning"
  | "standing"
  | "away"
  | "reset";

/**
 * Computes the temperature stage from widget props.
 *
 * - Away state always returns "away"
 * - Standing/Walking with full reset returns "reset", otherwise "standing"
 * - Sitting escalates: calm < 0.5, warm < 0.8, hot < 1.0, burning >= 1.0
 */
export function computeTemperature(props: WidgetProps): Temperature {
  if (props.state === "Away") return "away";

  if (props.state === "Standing" || props.state === "Walking") {
    return props.breakResetProgress >= 1.0 ? "reset" : "standing";
  }

  if (props.limitRatio >= 1.0) return "burning";
  if (props.limitRatio >= 0.8) return "hot";
  if (props.limitRatio >= 0.5) return "warm";
  return "calm";
}
