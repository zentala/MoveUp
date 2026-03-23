/**
 * scenarios.ts
 *
 * Predefined scenarios for simulating desk sensor readings in tests.
 * Constants and helpers for generating test scenarios.
 */

import { invoke } from "@tauri-apps/api/core";

/**
 * Default sitting desk height (floor distance 750mm = desk height 72cm at 3cm thickness).
 */
export const SITTING_DISTANCE_MM = 750;

/**
 * Default standing desk height (floor distance 1080mm = desk height 105cm at 3cm thickness).
 */
export const STANDING_DISTANCE_MM = 1080;

/**
 * Midpoint between sitting and standing (for ambiguous readings).
 */
export const MID_DISTANCE_MM = (SITTING_DISTANCE_MM + STANDING_DISTANCE_MM) / 2;

/**
 * Debounce count — required number of consistent readings for state change.
 * Defined in session.rs as DEBOUNCE_COUNT = 5.
 */
export const DEBOUNCE_COUNT = 5;

/**
 * Send multiple readings of the same distance to trigger debounce.
 * Default is 10 readings to ensure debounce + extra confirmations.
 */
export async function sendRepeated(
  mm: number,
  count: number = DEBOUNCE_COUNT + 5,
  active: boolean = true,
  delayMs: number = 100
): Promise<void> {
  for (let i = 0; i < count; i++) {
    await invoke("inject_reading", { mm, active });
    if (delayMs > 0) {
      await new Promise((resolve) => setTimeout(resolve, delayMs));
    }
  }
}

/**
 * Simulate a transition from sitting to standing and back.
 */
export async function simulatePositionChange(
  from: "Sitting" | "Standing",
  duration_ms: number = 1000,
  active: boolean = true
): Promise<void> {
  const fromMm = from === "Sitting" ? SITTING_DISTANCE_MM : STANDING_DISTANCE_MM;
  const toMm = from === "Sitting" ? STANDING_DISTANCE_MM : SITTING_DISTANCE_MM;

  // Transition to new position
  await sendRepeated(toMm, DEBOUNCE_COUNT + 2, active, 50);

  // Hold position
  await new Promise((resolve) => setTimeout(resolve, duration_ms));

  // Transition back
  await sendRepeated(fromMm, DEBOUNCE_COUNT + 2, active, 50);
}

/**
 * Simulate a full sitting session with break.
 *
 * @param config - Scenario configuration
 */
export interface SessionScenarioConfig {
  /** Duration of initial sitting in milliseconds */
  sittingDuration_ms?: number;
  /** Duration of standing/break in milliseconds */
  breakDuration_ms?: number;
  /** Whether to return to sitting after break (default: false) */
  returnToSitting?: boolean;
  /** Delay between readings in milliseconds */
  readingDelayMs?: number;
}

export async function simulateSession(config: SessionScenarioConfig = {}): Promise<void> {
  const {
    sittingDuration_ms = 5000,
    breakDuration_ms = 3000,
    returnToSitting = false,
    readingDelayMs = 100,
  } = config;

  // Initial sitting
  await sendRepeated(SITTING_DISTANCE_MM, DEBOUNCE_COUNT + 2, true, readingDelayMs);
  await new Promise((resolve) => setTimeout(resolve, sittingDuration_ms));

  // Transition to standing/break
  await sendRepeated(STANDING_DISTANCE_MM, DEBOUNCE_COUNT + 2, true, readingDelayMs);
  await new Promise((resolve) => setTimeout(resolve, breakDuration_ms));

  // Return to sitting if requested
  if (returnToSitting) {
    await sendRepeated(SITTING_DISTANCE_MM, DEBOUNCE_COUNT + 2, true, readingDelayMs);
  }
}

/**
 * Simulate standing without keyboard/mouse activity (Walking state).
 */
export async function simulateWalking(duration_ms: number = 2000): Promise<void> {
  await sendRepeated(STANDING_DISTANCE_MM, DEBOUNCE_COUNT + 2, false, 50); // active = false
  await new Promise((resolve) => setTimeout(resolve, duration_ms));
}

export default {
  SITTING_DISTANCE_MM,
  STANDING_DISTANCE_MM,
  MID_DISTANCE_MM,
  DEBOUNCE_COUNT,
  sendRepeated,
  simulatePositionChange,
  simulateSession,
  simulateWalking,
};
