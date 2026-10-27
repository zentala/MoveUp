import { test, expect } from '@playwright/test';

for (const width of [1280, 760]) {
  test(`Analyst ${width}: conflicting history stays local and clean range recovers without reload`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width, height: 800 });
    const pageErrors: string[] = [];
    page.on('pageerror', error => pageErrors.push(error.message));
    await page.route('**/*', async route => {
      const request = route.request();
      const url = new URL(request.url());
      if (request.method() !== 'GET' || url.origin !== 'http://127.0.0.1:15431' || url.pathname.startsWith('/display')) {
        return route.abort();
      }
      if (url.pathname === '/') {
        const response = await route.fetch();
        return route.fulfill({ response, body: (await response.text()).replace('/src/main.tsx', '/tests/e2e/analyst-containment.fixture.ts') });
      }
      return route.continue();
    });
    await page.goto('/?analyst-containment-fixture=conflict#/analyst');
    const panel = page.locator('#analyst-selected-day-panel');
    await expect(panel).toContainText('History unavailable: Conflicting overlapping intervals: Sitting / Away');
    await expect(page.getByText('View unavailable.', { exact: true })).toHaveCount(0);
    await expect(page.getByTestId('timeline-detail')).toHaveCount(0);
    await expect(page.locator('.analyst-day-details')).toHaveCount(0);
    await expect(page.getByTestId('range-from')).toHaveValue('2026-05-16');
    await expect(page.getByTestId('range-to')).toHaveValue('2026-05-17');
    for (const day of ['2026-05-16', '2026-05-17']) {
      const ring = page.getByTestId(`day-ring-${day}`);
      await expect(ring).toHaveAccessibleName(/composition unavailable/);
      await expect(ring).not.toHaveAccessibleName(/No observations recorded|100.00 percent|0.000 seconds/);
      await expect(ring.locator('circle')).toHaveCount(0);
    }
    const boot = await page.evaluate(() => window.analystContainmentTest.boot);
    const calls = await page.evaluate(() => window.analystContainmentTest.calls.filter(c => c === 'get_sessions_range').length);
    await page.getByRole('button', { name: 'Retry loading Analyst data' }).click();
    await expect.poll(() => page.evaluate(() => window.analystContainmentTest.calls.filter(c => c === 'get_sessions_range').length)).toBeGreaterThan(calls);
    await expect(panel).toContainText('History unavailable: Conflicting overlapping intervals');
    await page.getByRole('tab', { name: /2026-05-16/ }).click();
    await expect(page.getByRole('tab', { name: /2026-05-16/ })).toHaveAttribute('aria-selected', 'true');
    await page.getByRole('button', { name: 'Next day' }).click();
    await expect(page.getByRole('tab', { name: /2026-05-17/ })).toHaveAttribute('aria-selected', 'true');
    await page.screenshot({ path: testInfo.outputPath(`analyst-conflict-${width}.png`), fullPage: true });

    // The real date input changes the real range hook, not a fixture switch/reload.
    await page.getByTestId('range-from').fill('2026-05-17');
    await expect(page.getByTestId('timeline-detail')).toBeVisible();
    await expect(panel).not.toContainText('History unavailable');
    await expect(page.getByTestId('timeline-segment')).toHaveCount(1);
    await expect(page.getByTestId('timeline-segment')).toHaveAttribute('data-state', 'Standing');
    await expect(page.getByTestId('day-ring-2026-05-17')).toHaveAccessibleName(/Standing 1.39 percent, 1200.000 seconds/);
    await expect(page.getByRole('button', { name: 'Refresh', exact: true })).toBeEnabled();
    await page.screenshot({ path: testInfo.outputPath(`analyst-clean-${width}.png`), fullPage: true });
    expect(await page.evaluate(() => window.analystContainmentTest.boot)).toBe(boot);
    expect(await page.evaluate(() => window.analystContainmentTest.blocked)).toEqual([]);
    expect(await page.evaluate(() => window.__MOVEUP_UI_DIAGNOSTICS__?.boot)).toBe('mounted');
    expect(pageErrors).toEqual([]);
  });
}
