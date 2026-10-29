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
    await expect(page.getByLabel('From')).toHaveValue('2026-05-16');
    await expect(page.getByTestId('range-to')).toHaveValue('2026-05-17');
    await expect(page.getByRole('button', { name: '7d', exact: true })).toHaveAttribute('aria-pressed', 'false');
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
    const currentDay = page.getByRole('tab', { name: /2026-05-17/ });
    const previousDay = page.getByRole('tab', { name: /2026-05-16/ });
    await currentDay.focus();
    await page.keyboard.press('ArrowLeft');
    await expect(previousDay).toBeFocused();
    await expect(currentDay).toHaveAttribute('aria-selected', 'true');
    await page.keyboard.press('Space');
    await expect(previousDay).toHaveAttribute('aria-selected', 'true');
    await page.keyboard.press('ArrowRight');
    await expect(currentDay).toBeFocused();
    await expect(previousDay).toHaveAttribute('aria-selected', 'true');
    await page.keyboard.press('Enter');
    await expect(currentDay).toHaveAttribute('aria-selected', 'true');
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
    const today = await page.evaluate(() => {
      const now = new Date();
      return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
    });
    await page.getByRole('button', { name: 'Today', exact: true }).click();
    await expect(page.getByTestId('range-to')).toHaveValue(today);
    await expect(page.getByRole('tab', { name: new RegExp(today) })).toHaveAttribute('aria-selected', 'true');
    await page.getByRole('button', { name: '7d', exact: true }).click();
    const expectedFrom = await page.evaluate(() => {
      const date = new Date();
      date.setDate(date.getDate() - 6);
      return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
    });
    await expect(page.getByTestId('range-from')).toHaveValue(expectedFrom);
    await expect(page.getByTestId('range-to')).toHaveValue(today);
    await expect(page.getByRole('button', { name: '7d', exact: true })).toHaveAttribute('aria-pressed', 'true');

    if (width === 760) {
      const scaledSelectors = [
        'label[for="analyst-range-from"]', '#analyst-range-from',
        'label[for="analyst-range-to"]', '#analyst-range-to',
        '[data-testid="range-today"]', '[data-testid="preset-7d"]',
        '[data-testid="preset-14d"]', '[data-testid="preset-30d"]',
      ];
      const baseline = await page.evaluate(selectors => Object.fromEntries(selectors.map(selector => {
        const element = document.querySelector(selector);
        if (!element) throw new Error(`Missing Analyst date control: ${selector}`);
        return [selector, parseFloat(getComputedStyle(element).fontSize)];
      })), scaledSelectors);
      await page.addStyleTag({ content: scaledSelectors.map(selector =>
        `${selector} { font-size: ${baseline[selector] * 2}px !important; }`).join('\n') });
      const enlarged = await page.evaluate(selectors => Object.fromEntries(selectors.map(selector => [
        selector, parseFloat(getComputedStyle(document.querySelector(selector)!).fontSize),
      ])), scaledSelectors);
      for (const selector of scaledSelectors) expect(enlarged[selector]).toBe(baseline[selector] * 2);

      const dateFields = [
        { id: 'analyst-range-from' },
        { id: 'analyst-range-to' },
      ];
      for (const field of dateFields) {
        const input = page.locator(`#${field.id}`);
        await input.focus();
        await expect(input).toBeFocused();
      }
      const dateGeometry = await page.evaluate(() => {
        const viewport = { width: innerWidth, height: innerHeight };
        return ['from', 'to'].map(name => {
          const label = document.querySelector<HTMLLabelElement>(`label[for="analyst-range-${name}"]`)!;
          const input = document.querySelector<HTMLInputElement>(`#analyst-range-${name}`)!;
          const labelRect = label.getBoundingClientRect();
          const inputRect = input.getBoundingClientRect();
          const range = document.createRange();
          range.selectNodeContents(label);
          const textRects = [...range.getClientRects()];
          return {
            label: { left: labelRect.left, top: labelRect.top, right: labelRect.right, bottom: labelRect.bottom },
            input: { left: inputRect.left, top: inputRect.top, right: inputRect.right, bottom: inputRect.bottom },
            text: textRects.map(rect => ({ left: rect.left, top: rect.top, right: rect.right, bottom: rect.bottom })),
            inputScrollWidth: input.scrollWidth, inputClientWidth: input.clientWidth,
            viewport,
          };
        });
      });
      for (const field of dateGeometry) {
        expect(field.text.length).toBeGreaterThan(0);
        for (const rect of [field.label, field.input, ...field.text]) {
          expect(rect.left).toBeGreaterThanOrEqual(0);
          expect(rect.top).toBeGreaterThanOrEqual(0);
          expect(rect.right).toBeLessThanOrEqual(field.viewport.width);
          expect(rect.bottom).toBeLessThanOrEqual(field.viewport.height);
        }
        expect(field.inputScrollWidth).toBeLessThanOrEqual(field.inputClientWidth);
      }
      for (const control of await page.locator('[data-testid="range-today"], [data-testid^="preset-"]').all()) {
        await expect(control).toBeVisible();
        const box = await control.boundingBox();
        expect(box).not.toBeNull();
        expect(box!.x).toBeGreaterThanOrEqual(0);
        expect(box!.y).toBeGreaterThanOrEqual(0);
        expect(box!.x + box!.width).toBeLessThanOrEqual(width);
        expect(box!.y + box!.height).toBeLessThanOrEqual(800);
      }
      await page.screenshot({ path: testInfo.outputPath('analyst-date-preset-200pct-760.png'), fullPage: true });
    }
    expect(await page.evaluate(() => window.analystContainmentTest.boot)).toBe(boot);
    expect(await page.evaluate(() => window.analystContainmentTest.blocked)).toEqual([]);
    expect(await page.evaluate(() => window.__MOVEUP_UI_DIAGNOSTICS__?.boot)).toBe('mounted');
    expect(pageErrors).toEqual([]);
  });
}
