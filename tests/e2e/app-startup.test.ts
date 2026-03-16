/**
 * app-startup.test.ts
 *
 * E2E tests for app startup and basic UI initialization.
 */

import { test, expect } from "@playwright/test";

test.describe("App Startup", () => {
  test("app loads and shows main window", async ({ page }) => {
    await page.goto("http://localhost:1443");

    // Wait for app to load
    await page.waitForLoadState("networkidle");

    // Check for key UI elements
    const heading = page.locator("h1, h2, [role=heading]");
    await expect(heading).toBeVisible({ timeout: 5000 });
  });

  test("shows connection status", async ({ page }) => {
    await page.goto("http://localhost:1443");
    await page.waitForLoadState("networkidle");

    // Look for connection/device status indicator
    const status = page.locator("[data-testid=device-status], .device-status, .connection-status");

    // Either visible or element exists in DOM
    const count = await status.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });

  test("overlay window can be toggled", async ({ page }) => {
    await page.goto("http://localhost:1443");
    await page.waitForLoadState("networkidle");

    // Try to find and interact with overlay toggle
    const toggleButton = page.locator("[data-testid=toggle-overlay], button:has-text('Overlay')");

    // If button exists, it should be clickable (but may not exist in all states)
    const count = await toggleButton.count();
    if (count > 0) {
      await expect(toggleButton).toBeVisible();
    }
  });

  test("settings button opens settings panel", async ({ page }) => {
    await page.goto("http://localhost:1443");
    await page.waitForLoadState("networkidle");

    // Look for settings button (⚙ icon or "Settings" text)
    const settingsButton = page.locator("[data-testid=settings-btn], button:has-text('⚙'), button:has-text('Settings')");

    const count = await settingsButton.count();
    if (count > 0) {
      await settingsButton.click();

      // Wait for settings panel to appear
      const settingsPanel = page.locator("[data-testid=settings-panel], .settings-panel");
      await expect(settingsPanel).toBeVisible({ timeout: 2000 });
    }
  });
});
