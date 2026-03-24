/**
 * scenarios.ts — Predefined app states for mockups and tests.
 *
 * Each scenario represents a realistic moment in the user's day.
 * Used by: /mockup dev route, unit tests, integration tests.
 *
 * Scenario data lives in scenarios-sitting.ts and scenarios-other.ts.
 * This file re-exports everything + defines collection arrays.
 */
import type { WidgetProps } from "@/types";

/** A named scenario with description and full widget state. */
export interface Scenario {
  id: string;
  name: string;
  description: string;
  /** When this typically happens in the user's day. */
  context: string;
  props: WidgetProps;
}

// Re-export all individual scenarios
export {
  S01_FRESH_START,
  S02_SITTING_GREEN,
  S03_SITTING_YELLOW,
  S04_SITTING_OVERTIME,
} from "./scenarios-sitting";

export {
  S05_STANDING_MID,
  S06_AWAY,
  S07_BACK_FROM_AWAY,
  S08_GOOD_DAY,
  S09_DISCONNECTED,
} from "./scenarios-other";

// Import for array construction
import {
  S01_FRESH_START, S02_SITTING_GREEN,
  S03_SITTING_YELLOW, S04_SITTING_OVERTIME,
} from "./scenarios-sitting";

import {
  S05_STANDING_MID, S06_AWAY, S07_BACK_FROM_AWAY,
  S08_GOOD_DAY, S09_DISCONNECTED,
} from "./scenarios-other";

/** Primary scenarios — must look right before any release. */
export const PRIMARY_SCENARIOS: Scenario[] = [
  S01_FRESH_START,
  S02_SITTING_GREEN,
  S03_SITTING_YELLOW,
  S04_SITTING_OVERTIME,
  S05_STANDING_MID,
  S06_AWAY,
  S07_BACK_FROM_AWAY,
];

/** Secondary scenarios — edge cases. */
export const SECONDARY_SCENARIOS: Scenario[] = [
  S08_GOOD_DAY,
  S09_DISCONNECTED,
];

/** All scenarios. */
export const ALL_SCENARIOS: Scenario[] = [
  ...PRIMARY_SCENARIOS,
  ...SECONDARY_SCENARIOS,
];
