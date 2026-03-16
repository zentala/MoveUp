/**
 * vitest.config.ts
 *
 * Configuration for integration tests that need the node environment
 * (Tauri IPC access) instead of jsdom.
 *
 * Usage:
 *   vitest run --config vitest.config.ts tests/integration
 */

import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "node",
    globals: true,
    include: ["tests/integration/**/*.test.ts"],
    // Integration tests require the Tauri app to be running
    // Start with: pnpm tauri dev
  },
});
