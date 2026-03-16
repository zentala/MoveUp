/**
 * setup.ts — integration test setup.
 *
 * Verifies that the Tauri app is running before running integration tests.
 * If the app is not running, provides a helpful error message.
 */

import { beforeAll } from "vitest";
import { invoke } from "@tauri-apps/api/core";

beforeAll(async () => {
  try {
    // Try to invoke a simple command to verify app is running
    await invoke("get_session_state");
  } catch (error) {
    const errorMessage = `
╭─────────────────────────────────────────────────────────────────────╮
│ INTEGRATION TEST SETUP ERROR                                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│ The Tauri app is not running!                                      │
│                                                                     │
│ To run integration tests, start the app in another terminal:       │
│                                                                     │
│   pnpm tauri:dev                                                   │
│                                                                     │
│ Then run the tests:                                                │
│                                                                     │
│   pnpm test:integration                                            │
│                                                                     │
│ Note: Integration tests require the app to be running because      │
│       they communicate via Tauri IPC to inject sensor readings.    │
│                                                                     │
╰─────────────────────────────────────────────────────────────────────╯
`;
    console.error(errorMessage);
    throw new Error(
      "Tauri app not running. Start with: pnpm tauri:dev"
    );
  }
}, {
  timeout: 10000, // Allow up to 10 seconds for app to be ready
});

export {};
