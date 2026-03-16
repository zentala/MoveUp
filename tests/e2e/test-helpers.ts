/**
 * test-helpers.ts
 *
 * Helper utilities for E2E tests.
 */

import { Page } from "@playwright/test";

/**
 * Invoke a Tauri command from within a Playwright page context.
 *
 * This works by executing JavaScript within the page that calls the Tauri API.
 */
export async function invokeCommand(
  page: Page,
  command: string,
  args: Record<string, any> = {}
): Promise<any> {
  const result = await page.evaluate(
    ({ cmd, cmdArgs }) => {
      // @ts-ignore - Tauri API is injected by the app
      return window.__TAURI_INVOKE__(cmd, cmdArgs);
    },
    { cmd: command, cmdArgs: args }
  );

  return result;
}

/**
 * Wait for a Tauri event to be emitted.
 */
export async function waitForEvent(
  page: Page,
  eventName: string,
  timeoutMs: number = 5000
): Promise<any> {
  return page.evaluate(
    ({ event, timeout }) => {
      return new Promise((resolve, reject) => {
        let timeoutId: NodeJS.Timeout;

        const handler = (e: CustomEvent) => {
          clearTimeout(timeoutId);
          // @ts-ignore
          window.__TAURI_EVENTS__.off(event, handler);
          resolve(e.detail);
        };

        // @ts-ignore
        window.__TAURI_EVENTS__.once(event, handler);

        timeoutId = setTimeout(() => {
          // @ts-ignore
          window.__TAURI_EVENTS__.off(event, handler);
          reject(new Error(`Event "${event}" not received within ${timeout}ms`));
        }, timeout);
      });
    },
    { event: eventName, timeout: timeoutMs }
  );
}

/**
 * Get the current session state from the app.
 */
export async function getSessionState(page: Page): Promise<any> {
  return invokeCommand(page, "get_session_state", {});
}

/**
 * Send a series of identical distance readings (for debouncing).
 */
export async function sendReadings(
  page: Page,
  mm: number,
  count: number = 5,
  active: boolean = true,
  delayMs: number = 100
): Promise<void> {
  for (let i = 0; i < count; i++) {
    await invokeCommand(page, "inject_reading", { mm, active });
    await page.waitForTimeout(delayMs);
  }
}
