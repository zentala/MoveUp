/**
 * playwright.config.ts
 *
 * Configuration for Playwright E2E tests with Tauri WebDriver.
 *
 * To run:
 *   tauri driver --compatibility-mode
 *   pnpm test:e2e
 */

import { defineConfig, devices } from "@playwright/test";

const baseUrl = "http://localhost:1443";

export default defineConfig({
  testDir: "./tests/e2e",
  testMatch: "*.test.ts",

  fullyParallel: false, // Run tests serially to avoid port conflicts
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1, // Single worker due to single app instance

  reporter: "html",

  use: {
    baseURL: baseUrl,
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },

  webServer: {
    command: "pnpm tauri dev",
    url: baseUrl,
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },

  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
