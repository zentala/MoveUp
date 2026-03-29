/**
 * setup.ts — global test setup loaded before each test file.
 * Extends Vitest's `expect` with jest-dom matchers and mocks Tauri API.
 */
import "@testing-library/jest-dom";
import { vi } from "vitest";

/**
 * Mock @tauri-apps/api/core to prevent "window is not defined" errors.
 * Tests that call Tauri commands (invoke, listen, etc.) will use these mocks.
 */
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (cmd: string, _args?: any) => {
    // Default mock responses per command
    const responses: Record<string, any> = {
      get_session_state: {
        state: "sitting",
        sitting_seconds: 0,
        standing_seconds: 0,
        break_seconds: 0,
        session_limit_secs: 2700,
        stand_limit_secs: 0,
        desk_height_cm: 75.0,
        position_changes: 0,
        limit_used_secs: 0,
        daily_score: 0,
        standing_session_secs: 0,
      },
      get_settings: {
        sitting_mm: 750,
        standing_mm: 1050,
        desk_thickness_mm: 30,
        sit_limit_mins: 40,
        stand_limit_mins: 0,
        notify_inactivity: true,
        notify_daily_posture_balance: true,
        notify_praise_halfway: true,
      },
      get_today_summary: {
        sitting_secs: 0,
        standing_secs: 0,
        yesterday_sitting_secs: 0,
        yesterday_standing_secs: 0,
        position_changes: 0,
        sessions: [],
      },
      list_ports: [],
    };

    return responses[cmd] ?? null;
  }),
  listen: vi.fn(async (_event: string, _handler: any) => {
    // Return an unlisten function
    return vi.fn();
  }),
  unlisten: vi.fn(),
}));

/**
 * Mock @tauri-apps/api/event for event listening in components.
 */
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (_event: string, _handler: any) => {
    return vi.fn();
  }),
  unlisten: vi.fn(),
}));
