import type { TimelineLabel } from "./intervals";

/** Colors resolve through MoveUp's accepted DESIGN.md tokens. */
export const timelineColors: Record<TimelineLabel, string> = {
  Sitting: "var(--state-sitting, #fa4d56)",
  Standing: "var(--state-standing, #42be65)",
  Walking: "var(--state-walking, #78a9ff)",
  Away: "var(--state-away, #8d8d8d)",
  Unknown: "var(--state-unknown, #525252)",
  Future: "var(--state-future, #393939)",
};
