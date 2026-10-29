import { expect, test, type Locator, type Page } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

function configuredWelcomeEntry() {
  const config = JSON.parse(readFileSync(resolve('src-tauri/tauri.conf.json'), 'utf8')) as {
    app?: { windows?: Array<{ label?: string; url?: string }> };
  };
  const welcome = config.app?.windows?.find(window => window.label === 'welcome');
  return welcome?.url ?? 'index.html';
}

async function openApp(page: Page, scenario = 'normal') {
  page.on('pageerror', error => console.log('Settings fixture page error:', error.name, error.message, error.stack, JSON.stringify(error)));
  page.on('console', message => { if (message.type() === 'error') console.log(`Settings fixture console error: ${message.text()}`); });
  await page.setViewportSize({ width: 420, height: 240 });
  await page.route('**/*', async route => {
    const url = new URL(route.request().url());
    if (url.origin !== 'http://127.0.0.1:15431' || url.pathname.startsWith('/display')) return route.abort();
    if (url.pathname === '/') {
      const response = await route.fetch();
      return route.fulfill({ response, body: (await response.text()).replace('/src/main.tsx', '/tests/e2e/ux-flows.fixture.ts') });
    }
    return route.continue();
  });
  await page.goto(`/?settings-fixture=${scenario}`);
  await expect(page.getByTestId('one-bar-widget')).toBeVisible();
  if (scenario === 'load-retry') await page.evaluate(() => window.uxSettingsTest.rejectNextSettingsLoad());
}

async function expectContainedInScrollBody(marker: Locator, scrollBody: Locator, tolerance = 0) {
  await expect(marker).toBeVisible();
  const markerBox = await marker.boundingBox();
  const bodyBox = await scrollBody.boundingBox();
  expect(markerBox).not.toBeNull();
  expect(bodyBox).not.toBeNull();
  expect(markerBox!.x).toBeGreaterThanOrEqual(bodyBox!.x - tolerance);
  expect(markerBox!.y).toBeGreaterThanOrEqual(bodyBox!.y - tolerance);
  expect(markerBox!.x + markerBox!.width).toBeLessThanOrEqual(bodyBox!.x + bodyBox!.width + 1);
  expect(markerBox!.y + markerBox!.height).toBeLessThanOrEqual(bodyBox!.y + bodyBox!.height + 1);
}

async function keyboardBringIntoScrollBody(page: Page, marker: Locator, scrollBody: Locator) {
  for (let attempt = 0; attempt < 12; attempt++) {
    const markerBox = await marker.boundingBox();
    const bodyBox = await scrollBody.boundingBox();
    expect(markerBox).not.toBeNull();
    expect(bodyBox).not.toBeNull();
    if (markerBox!.y >= bodyBox!.y && markerBox!.y + markerBox!.height <= bodyBox!.y + bodyBox!.height) return;
    await page.keyboard.press(markerBox!.y < bodyBox!.y ? 'ArrowUp' : 'ArrowDown');
  }
  await expectContainedInScrollBody(marker, scrollBody);
}

async function expectFullStateCaption(page: Page, expected: string, expectedActivity: string) {
  const caption = page.locator('.state-indicator__label');
  const header = page.locator('.one-bar__header');
  const actions = page.locator('.one-bar__header-actions');
  const today = actions.getByRole('button', { name: "Open Analyst for today's local day" });
  await expect(caption).toHaveText(expected);
  const activity = page.locator('.state-indicator__activity');
  await expect(activity).toHaveText(expectedActivity);
  await expect(today).toBeVisible();
  await expect(today).toBeInViewport();
  await expect(actions.getByRole('button')).toHaveCount(3);
  for (const button of await actions.getByRole('button').all()) await expect(button).toBeInViewport();
  const dimensions = await caption.evaluate(element => {
    const header = element.closest('.one-bar__header')!;
    const status = element.closest('.state-indicator')!;
    const activity = status.querySelector<HTMLElement>('.state-indicator__activity')!;
    const fragments = (node: HTMLElement) => {
      const range = document.createRange();
      range.selectNodeContents(node);
      return Array.from(range.getClientRects()).map(rect => ({
        left: rect.left, top: rect.top, right: rect.right, bottom: rect.bottom,
      }));
    };
    return {
      label: {
        clientWidth: element.clientWidth,
        scrollWidth: element.scrollWidth,
        clientHeight: element.clientHeight,
        scrollHeight: element.scrollHeight,
        box: element.getBoundingClientRect().toJSON(),
        fragments: fragments(element),
      },
      activity: {
        clientWidth: activity.clientWidth,
        scrollWidth: activity.scrollWidth,
        clientHeight: activity.clientHeight,
        scrollHeight: activity.scrollHeight,
        box: activity.getBoundingClientRect().toJSON(),
        fragments: fragments(activity),
      },
      header: header.getBoundingClientRect().toJSON(),
      status: status.getBoundingClientRect().toJSON(),
      actions: header.querySelector<HTMLElement>('.one-bar__header-actions')!.getBoundingClientRect().toJSON(),
      details: header.querySelector<HTMLElement>('.one-bar__observation-details')!.getBoundingClientRect().toJSON(),
      height: status.querySelector<HTMLElement>('.state-indicator__height')?.getBoundingClientRect().toJSON() ?? null,
    };
  });
  for (const text of [dimensions.label, dimensions.activity]) {
    expect(text.clientWidth).toBeGreaterThan(0);
    expect(text.scrollWidth).toBeLessThanOrEqual(text.clientWidth + 1);
    expect(text.scrollHeight).toBeLessThanOrEqual(text.clientHeight + 1);
    expect(text.fragments.length).toBeGreaterThan(0);
    for (const fragment of text.fragments) {
      expect(fragment.left).toBeGreaterThanOrEqual(dimensions.header.x - 1);
      expect(fragment.top).toBeGreaterThanOrEqual(dimensions.header.y - 1);
      expect(fragment.right).toBeLessThanOrEqual(dimensions.header.right + 1);
      expect(fragment.bottom).toBeLessThanOrEqual(dimensions.header.bottom + 1);
    }
  }
  expect(dimensions.status.bottom).toBeLessThanOrEqual(dimensions.header.bottom + 1);
  if (dimensions.height) expect(dimensions.height.right).toBeLessThanOrEqual(dimensions.header.right + 1);
  expect(dimensions.actions.top).toBeGreaterThanOrEqual(dimensions.status.bottom - 1);
  expect(dimensions.actions.bottom).toBeLessThanOrEqual(dimensions.details.top + 1);
  expect(dimensions.details.right).toBeLessThanOrEqual(dimensions.header.right + 1);
}

async function expectSummaryTextFits(summary: Locator) {
  const dimensions = await summary.evaluate(element => ({
    clientWidth: element.clientWidth,
    scrollWidth: element.scrollWidth,
  }));
  expect(dimensions.clientWidth).toBeGreaterThan(0);
  expect(dimensions.scrollWidth).toBeLessThanOrEqual(dimensions.clientWidth + 1);
}

async function openSettings(page: Page, scenario = 'normal') {
  await openApp(page, scenario);
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await expect(page.getByText('Settings', { exact: true })).toBeVisible();
}

async function openWelcome(page: Page, scenario: 'welcome-queued' | 'welcome-error' | 'welcome-no-reading' | 'welcome-open-error' | 'welcome-preference-error') {
  await page.setViewportSize({ width: 480, height: 420 });
  await page.route(/\/welcome\.html(?:\?.*)?$/, async route => {
    const response = await route.fetch();
    const html = await response.text();
    const welcomeEntry = 'src="/src/welcome.tsx"';
    if (!html.includes(welcomeEntry)) throw new Error('Welcome HTML entry changed unexpectedly');
    await route.fulfill({ response, body: html.replace(welcomeEntry, 'src="/tests/e2e/ux-flows.fixture.ts"') });
  });
  await page.goto(`/${configuredWelcomeEntry()}?settings-fixture=${scenario}`);
  await expect(page.getByTestId('welcome-shell')).toBeVisible();
}

test('480x420 Welcome keeps truthful first-reading help and all footer controls visible and keyboard reachable', async ({ page }, info) => {
  await openWelcome(page, 'welcome-no-reading');
  await expect(page.getByRole('heading', { name: 'Your first reading' })).toBeVisible();
  await expect(page.getByText(/USB controller detection alone does not confirm that the assembled sensor sends readings/)).toBeVisible();
  await expect(page.getByText(/without a reading, the state remains unknown/i)).toBeVisible();
  await expect(page.getByText(/Sitting balance includes break credit; history shows recorded elapsed time/)).toBeVisible();

  const content = page.getByTestId('welcome-content');
  const footer = page.getByTestId('welcome-footer');
  const shell = page.getByTestId('welcome-shell');
  await expect(content).toHaveCSS('overflow-y', 'auto');
  await expect(shell).toHaveCSS('overflow', 'hidden');
  const footerControls = [
    page.getByRole('checkbox', { name: "Don't show this at next launch" }),
    page.getByRole('button', { name: 'Test reminder' }),
    page.getByRole('button', { name: 'Open MoveUp' }),
  ];
  const footerBox = await footer.boundingBox();
  expect(footerBox).not.toBeNull();
  expect(footerBox!.x).toBeGreaterThanOrEqual(0);
  expect(footerBox!.y).toBeGreaterThanOrEqual(0);
  expect(footerBox!.x + footerBox!.width).toBeLessThanOrEqual(480);
  expect(footerBox!.y + footerBox!.height).toBeLessThanOrEqual(420);
  for (const control of footerControls) {
    const box = await control.boundingBox();
    expect(box).not.toBeNull();
    expect(box!.x).toBeGreaterThanOrEqual(footerBox!.x);
    expect(box!.y).toBeGreaterThanOrEqual(footerBox!.y);
    expect(box!.x + box!.width).toBeLessThanOrEqual(footerBox!.x + footerBox!.width + 1);
    expect(box!.y + box!.height).toBeLessThanOrEqual(footerBox!.y + footerBox!.height + 1);
    expect(box!.x).toBeGreaterThanOrEqual(0);
    expect(box!.y).toBeGreaterThanOrEqual(0);
    expect(box!.x + box!.width).toBeLessThanOrEqual(480);
    expect(box!.y + box!.height).toBeLessThanOrEqual(420);
    await expect(control).toBeInViewport();
  }

  await footerControls[0].focus();
  await expect(footerControls[0]).toBeFocused();
  await page.keyboard.press('Tab');
  await expect(footerControls[1]).toBeFocused();
  await page.keyboard.press('Tab');
  await expect(footerControls[2]).toBeFocused();
  await page.screenshot({ path: info.outputPath('welcome-first-reading-480x420.png') });
});

test('480x420 Welcome keeps its footer reachable with measured 2x text growth', async ({ page }, info) => {
  await openWelcome(page, 'welcome-no-reading');
  const selectors = [
    '.welcome-screen__eyebrow', '.welcome-screen__header h1', '.welcome-screen__lead',
    '.welcome-screen__step-number', '.welcome-screen__steps h2', '.welcome-screen__steps p',
    '.welcome-screen__help-note', '.welcome-screen__feedback', '.welcome-screen__checkbox',
    '.welcome-screen__actions button',
  ];
  const baseline = await page.evaluate(items => Object.fromEntries(items.map(selector => [
    selector, parseFloat(getComputedStyle(document.querySelector(selector)!).fontSize),
  ])), selectors);
  await page.addStyleTag({ content: selectors.map(selector =>
    `${selector} { font-size: ${baseline[selector] * 2}px !important; }`).join('\n') });
  const enlarged = await page.evaluate(items => Object.fromEntries(items.map(selector => [
    selector, parseFloat(getComputedStyle(document.querySelector(selector)!).fontSize),
  ])), selectors);
  for (const selector of selectors) expect(enlarged[selector]).toBe(baseline[selector] * 2);

  const content = page.getByTestId('welcome-content');
  const footer = page.getByTestId('welcome-footer');
  const shell = page.getByTestId('welcome-shell');
  expect(await content.evaluate(element => element.scrollHeight > element.clientHeight)).toBe(true);
  await expect(content).toHaveCSS('overflow-y', 'auto');
  await expect(footer).toHaveCSS('overflow-y', 'visible');
  expect(await page.evaluate(() => document.documentElement.scrollHeight <= window.innerHeight)).toBe(true);
  expect(await shell.evaluate(element => element.scrollHeight <= element.clientHeight)).toBe(true);
  await content.evaluate(element => { element.scrollTop = element.scrollHeight; });

  const controls = [
    page.locator('.welcome-screen__checkbox input'),
    page.locator('.welcome-screen__test'),
    page.locator('.welcome-screen__primary'),
  ];
  const footerBox = await footer.boundingBox();
  expect(footerBox).not.toBeNull();
  expect(footerBox!.x).toBeGreaterThanOrEqual(0);
  expect(footerBox!.y).toBeGreaterThanOrEqual(0);
  expect(footerBox!.x + footerBox!.width).toBeLessThanOrEqual(480);
  expect(footerBox!.y + footerBox!.height).toBeLessThanOrEqual(420);
  for (const control of controls) {
    const box = await control.boundingBox();
    expect(box).not.toBeNull();
    expect(box!.x).toBeGreaterThanOrEqual(footerBox!.x);
    expect(box!.y).toBeGreaterThanOrEqual(footerBox!.y);
    expect(box!.x + box!.width).toBeLessThanOrEqual(footerBox!.x + footerBox!.width + 1);
    expect(box!.y + box!.height).toBeLessThanOrEqual(footerBox!.y + footerBox!.height + 1);
    expect(box!.x).toBeGreaterThanOrEqual(0);
    expect(box!.y).toBeGreaterThanOrEqual(0);
    expect(box!.x + box!.width).toBeLessThanOrEqual(480);
    expect(box!.y + box!.height).toBeLessThanOrEqual(420);
    await expect(control).toBeInViewport();
  }
  await controls[0].focus();
  await page.keyboard.press('Tab');
  await expect(controls[1]).toBeFocused();
  await page.keyboard.press('Tab');
  await expect(controls[2]).toBeFocused();
  await page.screenshot({ path: info.outputPath('welcome-first-reading-480x420-200pct.png') });
});

test('real App reports no sensor reading and exposes truthful calibration help', async ({ page }, info) => {
  await openApp(page, 'no-reading');
  await expect(page.getByTestId('timeline-observation-announcement')).not.toContainText(/device connection/i);
  await expect(page.getByTestId('timeline-observation-status')).toContainText('No sensor observations yet');
  await expect(page.getByText('Unknown', { exact: true })).toBeVisible();
  await expect(page.getByText('Sitting', { exact: true })).toHaveCount(0);
  await expect(page.getByText('72 cm', { exact: true })).toHaveCount(0);
  await expect(page.getByText('Recent input', { exact: true })).toBeVisible();
  await expect(page.getByTestId('one-bar-big-number')).toContainText('10:00 / 40:00');
  await expectFullStateCaption(page, 'Unknown', 'Recent input');

  const disclosure = page.getByTestId('observation-details');
  const summary = page.getByTestId('timeline-observation-status');
  const panel = page.getByRole('region', { name: 'How to read this status' });
  const scrollBody = page.getByRole('region', { name: 'Status explanation' });
  const close = page.getByRole('button', { name: 'Close status details' });
  await expect(summary).toHaveCSS('display', 'list-item');
  const summaryTarget = await summary.boundingBox();
  expect(summaryTarget).not.toBeNull();
  expect(summaryTarget!.height).toBeGreaterThanOrEqual(24);
  await summary.focus();
  await page.keyboard.press('Enter');
  await expect(disclosure).toHaveAttribute('open', '');
  await expect(panel).toBeVisible();
  await expect(scrollBody).toHaveAttribute('tabindex', '0');
  await expect(scrollBody).toHaveCSS('overflow-y', 'auto');
  await expect(close).toBeInViewport();
  await expect(panel).toContainText('No recent input can also mean reading, listening, thinking, or AI work');
  await expect(panel).toContainText('Timeline colors identify recorded desk state and time');
  await expect(panel).toContainText('Break credit reduces the current Sitting balance, but never removes recorded intervals');
  await expect(panel).toContainText('A sitting-colored interval alone does not mean the limit is exceeded');

  const panelBox = await panel.boundingBox();
  const closeBox = await close.boundingBox();
  expect(panelBox).not.toBeNull();
  expect(closeBox).not.toBeNull();
  expect(panelBox!.x).toBeGreaterThanOrEqual(0);
  expect(panelBox!.y).toBeGreaterThanOrEqual(0);
  expect(panelBox!.x + panelBox!.width).toBeLessThanOrEqual(420);
  expect(panelBox!.y + panelBox!.height).toBeLessThanOrEqual(240);
  expect(closeBox!.x).toBeGreaterThanOrEqual(panelBox!.x);
  expect(closeBox!.y).toBeGreaterThanOrEqual(panelBox!.y);
  expect(closeBox!.x + closeBox!.width).toBeLessThanOrEqual(panelBox!.x + panelBox!.width);
  expect(closeBox!.y + closeBox!.height).toBeLessThanOrEqual(panelBox!.y + panelBox!.height);
  expect(await scrollBody.evaluate(element => element.scrollHeight > element.clientHeight)).toBe(true);
  await scrollBody.focus();
  await page.keyboard.press('Home');
  const firstExplanation = scrollBody.locator('p').first();
  await expectContainedInScrollBody(firstExplanation, scrollBody, 2);
  await page.screenshot({ path: info.outputPath('app-observation-details-start-420x240.png') });
  const homeScroll = await scrollBody.evaluate(element => element.scrollTop);
  await page.keyboard.press('End');
  expect(await scrollBody.evaluate(element => element.scrollTop)).toBeGreaterThan(homeScroll);
  const finalExplanation = scrollBody.locator('p').last();
  await expectContainedInScrollBody(finalExplanation, scrollBody);
  await page.screenshot({ path: info.outputPath('app-observation-details-end-420x240.png') });

  const enlargedSelectors = [
    '.state-indicator__label', '.state-indicator__activity', '.one-bar__observation-status',
    '.one-bar__observation-panel-header h2', '.one-bar__observation-close', '.one-bar__observation-body',
  ];
  const baseline = await page.evaluate(selectors => Object.fromEntries(selectors.map(selector => {
    const element = document.querySelector(selector);
    if (!element) throw new Error(`Missing enlarged-text target: ${selector}`);
    return [selector, parseFloat(getComputedStyle(element).fontSize)];
  })), enlargedSelectors);
  await page.addStyleTag({ content: enlargedSelectors.map(selector =>
    `${selector} { font-size: ${baseline[selector] * 2}px !important; }`).join('\n') });
  const enlarged = await page.evaluate(selectors => Object.fromEntries(selectors.map(selector => [
    selector, parseFloat(getComputedStyle(document.querySelector(selector)!).fontSize),
  ])), enlargedSelectors);
  for (const selector of enlargedSelectors) expect(enlarged[selector]).toBe(baseline[selector] * 2);
  await expect(close).toBeInViewport();
  const enlargedPanelBox = await panel.boundingBox();
  expect(enlargedPanelBox).not.toBeNull();
  expect(enlargedPanelBox!.x).toBeGreaterThanOrEqual(0);
  expect(enlargedPanelBox!.y).toBeGreaterThanOrEqual(0);
  expect(enlargedPanelBox!.x + enlargedPanelBox!.width).toBeLessThanOrEqual(420);
  expect(enlargedPanelBox!.y + enlargedPanelBox!.height).toBeLessThanOrEqual(240);
  expect(await scrollBody.evaluate(element => element.scrollHeight > element.clientHeight)).toBe(true);
  await scrollBody.focus();
  await page.keyboard.press('Home');
  await expectContainedInScrollBody(firstExplanation, scrollBody);
  await page.screenshot({ path: info.outputPath('app-observation-details-2x-start-420x240.png') });
  await page.keyboard.press('End');
  await expectContainedInScrollBody(finalExplanation, scrollBody);
  await page.screenshot({ path: info.outputPath('app-observation-details-2x-420x240.png') });

  await close.click();
  await expect(disclosure).not.toHaveAttribute('open');
  await expect(summary).toBeFocused();
  await summary.click();
  await expect(disclosure).toHaveAttribute('open', '');
  await close.click();
  await expect(summary).toBeFocused();
  await expect(page.getByTestId('one-bar-big-number')).toContainText('10:00 / 40:00');
  await page.screenshot({ path: info.outputPath('app-no-sensor-reading-420x240.png') });

  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await page.getByRole('tab', { name: 'Calibr.' }).focus();
  await page.keyboard.press('Enter');
  const help = page.locator('.setup-help__summary');
  await help.focus();
  await page.keyboard.press('Enter');
  await expect(page.locator('.setup-help')).toHaveAttribute('open', '');
  await expect(page.getByText(/Board detection alone is not a sensor reading/)).toBeVisible();
  await expect(page.getByText(/Missing readings stay unknown/)).toBeVisible();
  await expect(page.getByText(/Sitting balance includes break credit; history remains recorded elapsed time/)).toBeVisible();
  await page.screenshot({ path: info.outputPath('settings-calibration-first-reading-help-420x240.png') });
  expect(await page.evaluate(() => window.uxSettingsTest.blocked)).toEqual([]);
});

test('real App qualifies observed Sitting, inferred Away, and stale posture separately', async ({ page }, info) => {
  await openApp(page, 'known-sitting');
  await expect(page.getByText('Estimated Sitting', { exact: true })).toBeVisible();
  await expectFullStateCaption(page, 'Estimated Sitting', 'Recent input');
  await expect(page.getByText('72 cm', { exact: true })).toBeVisible();
  await page.screenshot({ path: info.outputPath('app-known-sitting-420x240.png') });

  await page.goto('/?settings-fixture=inferred-away');
  await expect(page.getByTestId('one-bar-widget')).toBeVisible();
  await expect(page.getByText('Unknown', { exact: true })).toBeVisible();
  await expectFullStateCaption(page, 'Unknown', 'No recent input · 10m 0s');
  await expect(page.getByText('Estimated Away', { exact: true })).toHaveCount(0);
  await expect(page.locator('.state-indicator__activity')).toContainText('No recent input');

  await page.goto('/?settings-fixture=stale-observation');
  await expect(page.getByTestId('one-bar-widget')).toBeVisible();
  await expect(page.getByText('Last estimate Sitting', { exact: true })).toBeVisible();
  await expectFullStateCaption(page, 'Last estimate Sitting', 'Recent input');
  await expect(page.getByText('72 cm', { exact: true })).toHaveCount(0);
  await expect(page.getByTestId('observation-details')).toContainText('sensor observation stale');
  await page.screenshot({ path: info.outputPath('app-stale-sitting-420x240.png') });
});

test('real App keeps a captured estimate distinct from a failed dashboard refresh', async ({ page }, info) => {
  await openApp(page, 'view-refresh-error');
  const summary = page.getByTestId('timeline-observation-status');
  await expect(page.getByText('Estimated Sitting', { exact: true })).toBeVisible();
  await expectFullStateCaption(page, 'Estimated Sitting', 'Recent input');
  await expect(page.getByText('72 cm', { exact: true })).toBeVisible();
  await expect(summary).not.toContainText('View refresh failed');
  await page.screenshot({ path: info.outputPath('app-view-refresh-current-420x240.png') });

  await page.evaluate(() => window.uxSettingsTest.failNextDashboardRefresh());
  await expect.poll(() => page.evaluate(() => window.uxSettingsTest.dashboardRefreshPending())).toBe(true);
  await expect(page.getByText('Estimated Sitting', { exact: true })).toBeVisible();
  await expectFullStateCaption(page, 'Estimated Sitting', 'Recent input');
  await expect(page.getByText('72 cm', { exact: true })).toBeVisible();
  await expect(summary).not.toContainText('View refresh failed');

  await page.evaluate(() => window.uxSettingsTest.rejectDashboardRefresh());
  await expect(page.getByText('Last estimate Sitting', { exact: true })).toBeVisible();
  await expectFullStateCaption(page, 'Last estimate Sitting', 'Recent input');
  await expect(page.getByText('72 cm', { exact: true })).toBeVisible();
  await expect(summary).toContainText('View refresh failed');
  await expect(summary).toContainText('Sitting observed through');
  await expect(summary).not.toContainText('sensor observation stale');
  await expectSummaryTextFits(summary);
  await page.screenshot({ path: info.outputPath('app-view-refresh-failed-420x240.png') });

  await summary.click();
  const details = page.getByRole('region', { name: 'How to read this status' });
  await expect(details).toContainText('View update failed: synthetic dashboard refresh failure');
  await expect(details).toContainText('last response received');
  await expect(details).toContainText('Sitting observed through');
  await expect(page.getByText('Last estimate Sitting', { exact: true })).toBeVisible();
  await expect(page.getByText('72 cm', { exact: true })).toBeVisible();
  expect(await page.evaluate(() => window.uxSettingsTest.calls.filter(call => call.command === 'get_dashboard_state').length)).toBeGreaterThan(1);
  expect(await page.evaluate(() => window.uxSettingsTest.blocked)).toEqual([]);
  await page.screenshot({ path: info.outputPath('app-dashboard-refresh-failure-details-420x240.png') });
});

test('real App names sitting balance, shows overtime separately, and exposes captured break credit', async ({ page }) => {
  await openApp(page, 'break-credit');
  await expect(page.locator('.one-bar__balance-label')).toHaveText('Sitting balance');
  await expect(page.getByTestId('one-bar-big-number')).toHaveText('52:00 / 40:00');
  await expect(page.getByTestId('one-bar-overtime')).toHaveText('Over limit 12m');
  const limitInk = await page.locator('.one-bar__balance-label').evaluate(element => getComputedStyle(element).color);
  await expect(page.locator('.one-bar__limit-total')).toHaveCSS('color', limitInk);
  const disclosure = page.getByTestId('break-credit-details');
  await expect(disclosure).toBeVisible();
  await page.screenshot({ path: resolve('node_modules/.cache/E031-T01-after/break-credit-balance-420x240-r18-after.png') });
  await page.getByText('Break credit details').click();
  await expect(disclosure).toHaveAttribute('open', '');
  const panel = page.getByRole('region', { name: 'Captured break credit' });
  const scrollBody = page.getByRole('region', { name: 'Break credit entries' });
  const close = page.getByRole('button', { name: 'Close break credit details' });
  await expect(panel).toBeVisible();
  await expect(scrollBody).toHaveAttribute('tabindex', '0');
  await expect(scrollBody).toHaveCSS('overflow-y', 'auto');
  await expect(close).toBeInViewport();
  await expect(scrollBody.locator('dt').first()).toHaveCSS('color', limitInk);
  const panelBox = await panel.boundingBox();
  const closeBox = await close.boundingBox();
  expect(panelBox).not.toBeNull();
  expect(closeBox).not.toBeNull();
  expect(panelBox!.x).toBeGreaterThanOrEqual(0);
  expect(panelBox!.y).toBeGreaterThanOrEqual(0);
  expect(panelBox!.x + panelBox!.width).toBeLessThanOrEqual(420);
  expect(panelBox!.y + panelBox!.height).toBeLessThanOrEqual(240);
  expect(closeBox!.y).toBeGreaterThanOrEqual(panelBox!.y);
  expect(closeBox!.y + closeBox!.height).toBeLessThanOrEqual(panelBox!.y + panelBox!.height);
  expect(await scrollBody.evaluate(element => element.scrollHeight > element.clientHeight)).toBe(true);

  const entries = scrollBody.locator('ol > li');
  await expect(entries).toHaveCount(3);
  await expect(entries.nth(0)).toContainText('×2');
  await expect(entries.nth(1)).toContainText('×1');
  await expect(entries.nth(2)).toContainText('already met this rule');
  const firstApplication = entries.nth(0).locator('dd').filter({ hasText: '×2' }).first();
  const nextApplication = entries.nth(1).locator('dd').filter({ hasText: '×1' }).first();
  const lastReason = entries.nth(2).getByText(/already met this rule/);

  await scrollBody.focus();
  await page.keyboard.press('Home');
  const homeScroll = await scrollBody.evaluate(element => element.scrollTop);
  await expectContainedInScrollBody(firstApplication, scrollBody);
  await page.keyboard.press('PageDown');
  expect(await scrollBody.evaluate(element => element.scrollTop)).toBeGreaterThan(homeScroll);
  await keyboardBringIntoScrollBody(page, nextApplication, scrollBody);
  await expectContainedInScrollBody(nextApplication, scrollBody);
  const nextScroll = await scrollBody.evaluate(element => element.scrollTop);
  await page.keyboard.press('End');
  expect(await scrollBody.evaluate(element => element.scrollTop)).toBeGreaterThanOrEqual(nextScroll);
  await expectContainedInScrollBody(lastReason, scrollBody);
  await page.screenshot({ path: resolve('node_modules/.cache/E031-T01-after/break-credit-details-420x240-r18-after.png') });

  await close.click();
  await expect(disclosure).not.toHaveAttribute('open');
  await expect(page.getByTestId('one-bar-big-number')).toHaveText('52:00 / 40:00');
  await expect(page.getByTestId('one-bar-overtime')).toHaveText('Over limit 12m');
  await expect(disclosure.locator('summary')).toBeFocused();

  await page.getByText('Break credit details').click();
  const enlargedSelectors = [
    '.one-bar__credit-details summary', '.one-bar__credit-panel-header h2', '.one-bar__credit-close',
    '.one-bar__credit-scroll', '.one-bar__credit-list dt', '.one-bar__credit-list dd',
    '.one-bar__credit-reason', '.one-bar__balance-label', '.one-bar__limit-total', '.one-bar__overtime',
  ];
  const baseline = await page.evaluate(selectors => Object.fromEntries(selectors.map(selector => {
    const element = document.querySelector(selector);
    if (!element) throw new Error(`Missing enlarged-text target: ${selector}`);
    return [selector, parseFloat(getComputedStyle(element).fontSize)];
  })), enlargedSelectors);
  await page.addStyleTag({ content: enlargedSelectors.map(selector =>
    `${selector} { font-size: ${baseline[selector] * 2}px !important; }`).join('\n') });
  const enlarged = await page.evaluate(selectors => Object.fromEntries(selectors.map(selector => [
    selector, parseFloat(getComputedStyle(document.querySelector(selector)!).fontSize),
  ])), enlargedSelectors);
  for (const selector of enlargedSelectors) expect(enlarged[selector]).toBe(baseline[selector] * 2);

  await expect(panel).toBeVisible();
  await expect(close).toBeInViewport();
  const enlargedPanelBox = await panel.boundingBox();
  expect(enlargedPanelBox).not.toBeNull();
  expect(enlargedPanelBox!.x).toBeGreaterThanOrEqual(0);
  expect(enlargedPanelBox!.y).toBeGreaterThanOrEqual(0);
  expect(enlargedPanelBox!.x + enlargedPanelBox!.width).toBeLessThanOrEqual(420);
  expect(enlargedPanelBox!.y + enlargedPanelBox!.height).toBeLessThanOrEqual(240);
  expect(await scrollBody.evaluate(element => element.scrollHeight > element.clientHeight)).toBe(true);
  await scrollBody.focus();
  await page.keyboard.press('Home');
  const enlargedHomeScroll = await scrollBody.evaluate(element => element.scrollTop);
  await expectContainedInScrollBody(firstApplication, scrollBody);
  await page.keyboard.press('PageDown');
  expect(await scrollBody.evaluate(element => element.scrollTop)).toBeGreaterThan(enlargedHomeScroll);
  await keyboardBringIntoScrollBody(page, nextApplication, scrollBody);
  await expectContainedInScrollBody(nextApplication, scrollBody);
  await page.keyboard.press('End');
  await expectContainedInScrollBody(lastReason, scrollBody);
  await page.screenshot({ path: resolve('node_modules/.cache/E031-T01-after/break-credit-details-420x240-2x-r18-after.png') });

  await close.click();
  await expect(disclosure.locator('summary')).toBeFocused();
  await expect(page.getByTestId('one-bar-big-number')).toHaveText('52:00 / 40:00');
  await expect(page.getByTestId('one-bar-overtime')).toHaveText('Over limit 12m');
});

test('real App recovers a rejected settings load without exposing defaults', async ({ page }, info) => {
  await openSettings(page, 'load-retry');
  await expect(page.getByRole('alert')).toContainText('settings store unavailable');
  await expect(page.getByRole('button', { name: 'Save' })).toBeDisabled();
  await expect(page.getByLabel('Sitting height (mm)')).toHaveCount(0);
  await page.screenshot({ path: info.outputPath('settings-load-error-420x240.png') });

  await page.getByRole('button', { name: 'Retry' }).click();
  await page.getByRole('tab', { name: 'Calibr.' }).click();
  await expect(page.getByLabel('Sitting height (mm)')).toHaveValue('750');
  await expect(page.getByRole('button', { name: 'Save' })).toBeEnabled();
  expect(await page.evaluate(() => window.uxSettingsTest.calls.filter(call => call.command === 'get_settings').length)).toBeGreaterThanOrEqual(3);
  expect(await page.evaluate(() => window.uxSettingsTest.blocked)).toEqual([]);
});

test('real App corrects calibration in place and saves once', async ({ page }, info) => {
  await openSettings(page, 'invalid');
  const tab = page.getByRole('tab', { name: 'Calibr.' });
  await tab.focus();
  await page.keyboard.press('Enter');
  const sitting = page.getByLabel('Sitting height (mm)');
  await expect(sitting).toHaveValue('900');
  await expect(page.getByRole('alert')).toContainText('greater than sitting height');
  await expect(page.getByRole('button', { name: 'Save' })).toBeDisabled();

  await sitting.fill('700');
  await expect(page.getByRole('alert')).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Save' })).toBeEnabled();
  await page.screenshot({ path: info.outputPath('settings-calibration-corrected-420x240.png') });
  await page.getByRole('button', { name: 'Save' }).click();
  await expect(page.getByTestId('one-bar-widget')).toBeVisible();
  expect(await page.evaluate(() => window.uxSettingsTest.saveAttempts())).toBe(1);
  const save = await page.evaluate(() => window.uxSettingsTest.calls.find(call => call.command === 'save_settings')?.args as { config: { sitting_mm: number } });
  expect(save.config.sitting_mm).toBe(700);
  expect(await page.evaluate(() => window.uxSettingsTest.blocked)).toEqual([]);
});

test('real App shows current runtime profile limits and unsupported Snooze honestly', async ({ page }, info) => {
  await openSettings(page, 'runtime-profile');
  const body = page.locator('.settings-panel__body');
  const ergonomic = page.getByLabel('Effective ergonomic settings');
  await ergonomic.scrollIntoViewIfNeeded();
  await page.screenshot({ path: info.outputPath('settings-runtime-profile-420x240.png') });
  await expect(ergonomic).toContainText('Session engine sitting limit: 40 min');
  await expect(ergonomic).toContainText('current sitting alert threshold: 55 min');
  const override = ergonomic.getByText(/current sitting alert threshold: 55 min/i);
  const overrideBox = await override.boundingBox();
  const bodyBox = await body.boundingBox();
  expect(overrideBox).not.toBeNull();
  expect(bodyBox).not.toBeNull();
  expect(overrideBox!.y).toBeGreaterThanOrEqual(bodyBox!.y);
  expect(overrideBox!.y + overrideBox!.height).toBeLessThanOrEqual(bodyBox!.y + bodyBox!.height + 1);

  await page.getByRole('tab', { name: 'Notif.' }).click();
  const communication = page.getByLabel('Effective communication settings');
  await expect(communication).toContainText('Current Dismiss delay: 0 min');
  await expect(communication).toContainText('Snooze is unavailable for the current policy step');
  await page.screenshot({ path: info.outputPath('settings-runtime-policy-420x240.png') });
  expect(await page.evaluate(() => window.uxSettingsTest.blocked)).toEqual([]);
});

test('real App keeps profile switches immediate while Back discards a settings draft', async ({ page }, info) => {
  await openSettings(page, 'runtime-profile');
  await expect(page.getByText(/Profile changes apply immediately/i)).toBeVisible();
  await expect(page.getByText(/Changes here are drafts until Save/i)).toBeVisible();
  await page.getByLabel('Active profile').selectOption('standard');
  await expect(page.getByLabel('Active profile')).toHaveValue('standard');
  await page.getByRole('tab', { name: 'Calibr.' }).click();
  await page.getByLabel('Sitting height (mm)').fill('760');
  await page.screenshot({ path: info.outputPath('settings-profile-immediate-draft-pending-420x240.png') });
  await page.getByRole('button', { name: 'Back', exact: true }).click();
  await expect(page.getByTestId('one-bar-widget')).toBeVisible();

  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await expect(page.getByText('Settings', { exact: true })).toBeVisible();
  await expect(page.getByLabel('Active profile')).toHaveValue('standard');
  await page.getByRole('tab', { name: 'Calibr.' }).click();
  await expect(page.getByLabel('Sitting height (mm)')).toHaveValue('750');
  expect(await page.evaluate(() => window.uxSettingsTest.saveAttempts())).toBe(0);
  expect(await page.evaluate(() => window.uxSettingsTest.calls.filter(call => call.command === 'switch_ergonomic_profile'))).toHaveLength(1);
  expect(await page.evaluate(() => window.uxSettingsTest.blocked)).toEqual([]);
});

test('real App applies a saved activity preference and keeps it off when a save fails', async ({ page }, info) => {
  await openSettings(page, 'activity');
  await page.getByRole('tab', { name: 'More' }).click();
  const activity = page.getByRole('checkbox', { name: /Show keyboard\/mouse activity/ });
  await expect(activity).toBeChecked();
  await activity.uncheck();
  await page.screenshot({ path: info.outputPath('settings-activity-draft-420x240.png') });
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.getByTestId('one-bar-widget')).toBeVisible();
  await expect(page.locator('.state-indicator__activity')).toHaveCount(0);
  await expect(page.getByText('Estimated Sitting', { exact: true })).toBeVisible();
  expect(await page.evaluate(() => window.uxSettingsTest.storedActivityPreference())).toBe(false);
  await page.screenshot({ path: info.outputPath('app-activity-disabled-after-save-420x240.png') });

  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await expect(page.getByText('Settings', { exact: true })).toBeVisible();
  await page.getByRole('tab', { name: 'More' }).click();
  await expect(page.getByRole('checkbox', { name: /Show keyboard\/mouse activity/ })).not.toBeChecked();
  await page.getByRole('button', { name: 'Back', exact: true }).click();
  expect(await page.evaluate(() => window.uxSettingsTest.blocked)).toEqual([]);
});

test('real App does not apply the activity draft after a failed settings save', async ({ page }, info) => {
  await openSettings(page, 'activity-save-fail');
  await page.getByRole('tab', { name: 'More' }).click();
  const activity = page.getByRole('checkbox', { name: /Show keyboard\/mouse activity/ });
  await activity.uncheck();
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect.poll(() => page.evaluate(() => window.uxSettingsTest.saveAttempts())).toBe(1);
  await page.evaluate(() => window.uxSettingsTest.rejectPendingSave());
  await expect(page.getByRole('alert')).toContainText('settings write failed');
  await expect(activity).not.toBeChecked();
  expect(await page.evaluate(() => window.uxSettingsTest.storedActivityPreference())).toBe(true);
  await page.screenshot({ path: info.outputPath('settings-activity-save-error-420x240.png') });
  await page.getByRole('button', { name: 'Back', exact: true }).click();
  await expect(page.getByTestId('one-bar-widget')).toBeVisible();
  await expect(page.getByText('Recent input', { exact: true })).toBeVisible();
  expect(await page.evaluate(() => window.uxSettingsTest.blocked)).toEqual([]);
});

test('real App retains a corrected draft and prevents duplicate rejected saves', async ({ page }) => {
  await openSettings(page, 'save-fail');
  await page.getByRole('tab', { name: 'Calibr.' }).click();
  const sitting = page.getByLabel('Sitting height (mm)');
  await sitting.fill('740');
  const save = page.locator('.settings-toolbar__save');
  await save.click();
  await expect.poll(() => page.evaluate(() => window.uxSettingsTest.saveAttempts())).toBe(1);
  await expect(save).toBeDisabled();
  const box = await save.boundingBox();
  await page.mouse.click(box!.x + box!.width / 2, box!.y + box!.height / 2);
  await expect.poll(() => page.evaluate(() => window.uxSettingsTest.saveAttempts())).toBe(1);
  await page.evaluate(() => window.uxSettingsTest.rejectPendingSave());
  await expect(page.getByRole('alert')).toContainText('settings write failed');
  await expect(sitting).toHaveValue('740');
  await expect(save).toBeEnabled();
  await expect(page.getByTestId('one-bar-widget')).toHaveCount(0);
  expect(await page.evaluate(() => window.uxSettingsTest.blocked)).toEqual([]);
});

test('compact Settings stays keyboard reachable and protects its draft from a reminder', async ({ page }, info) => {
  await openSettings(page);
  await page.getByRole('tab', { name: 'Calibr.' }).click();
  const sitting = page.getByLabel('Sitting height (mm)');
  await sitting.fill('760');
  await page.evaluate(() => window.uxSettingsTest.publishReminder());
  await expect(sitting).toHaveValue('760');
  await expect(page.getByTestId('application-reminder')).toHaveCount(0);

  const baseline = await page.evaluate(() => Object.fromEntries([
    '.settings-toolbar__title', '.settings-tab', '.settings-toolbar__save',
    '.settings-panel__helper', '.settings-panel__label', '.settings-panel__input', '.settings-panel__body',
  ].map(selector => [selector, parseFloat(getComputedStyle(document.querySelector(selector)!).fontSize)])));
  await page.addStyleTag({ content: `
    body { font-size: ${baseline['.settings-panel__body'] * 2}px !important; }
    .settings-toolbar__title, .settings-tab, .settings-toolbar__save,
    .settings-panel__helper, .settings-panel__label { font-size: ${baseline['.settings-tab'] * 2}px !important; }
    .settings-panel__input { font-size: ${baseline['.settings-panel__input'] * 2}px !important; }
  ` });
  const enlarged = await page.evaluate(() => Object.fromEntries([
    '.settings-toolbar__title', '.settings-tab', '.settings-toolbar__save',
    '.settings-panel__helper', '.settings-panel__label', '.settings-panel__input', '.settings-panel__body',
  ].map(selector => [selector, parseFloat(getComputedStyle(document.querySelector(selector)!).fontSize)])));
  for (const selector of Object.keys(baseline)) expect(enlarged[selector]).toBe(baseline[selector] * 2);
  for (const name of ['Back', 'Profiles', 'Calibr.', 'Notif.', 'More', 'Debug', 'Save']) {
    const control = name === 'Back' ? page.getByRole('button', { name })
      : name === 'Save' ? page.getByRole('button', { name }) : page.getByRole('tab', { name });
    await expect(control).toBeVisible();
    const box = await control.boundingBox();
    expect(box).not.toBeNull();
    expect(box!.x).toBeGreaterThanOrEqual(0);
    expect(box!.y).toBeGreaterThanOrEqual(0);
    expect(box!.x + box!.width).toBeLessThanOrEqual(421);
    expect(box!.y + box!.height).toBeLessThanOrEqual(241);
  }
  const body = page.locator('.settings-panel__body');
  const standingLabel = page.locator('label[for="standing-height"]');
  const standing = page.getByLabel('Standing height (mm)');
  const standingField = page.locator('.settings-panel__field').filter({ has: standing });
  await standingField.evaluate(element => element.scrollIntoView({ block: 'center' }));
  const standingLabelBox = await standingLabel.boundingBox();
  const standingBox = await standing.boundingBox();
  const bodyBox = await body.boundingBox();
  expect(standingLabelBox).not.toBeNull();
  expect(standingBox).not.toBeNull();
  expect(standingLabelBox!.y).toBeGreaterThanOrEqual(bodyBox!.y);
  expect(standingLabelBox!.y + standingLabelBox!.height).toBeLessThanOrEqual(bodyBox!.y + bodyBox!.height + 1);
  expect(standingBox!.y).toBeGreaterThanOrEqual(bodyBox!.y);
  expect(standingBox!.y + standingBox!.height).toBeLessThanOrEqual(bodyBox!.y + bodyBox!.height + 1);
  await page.screenshot({ path: info.outputPath('settings-protected-200pct-joint-field-420x240.png') });

  await page.getByRole('button', { name: 'Back' }).click();
  await expect(page.getByTestId('application-reminder')).toBeVisible();
  await expect(page.getByTestId('one-bar-widget')).toBeHidden();
  expect(await page.evaluate(() => window.uxSettingsTest.blocked)).toEqual([]);
});

test('compact Today entry requests the current local Analyst day', async ({ page }) => {
  await openApp(page);
  await page.locator('.one-bar__today-btn').click();

  await expect.poll(() => page.evaluate(() => window.uxSettingsTest.calls.some(call => call.command === 'open_analyst_window'))).toBe(true);
  const request = await page.evaluate(() => JSON.parse(localStorage.getItem('desk:analyst:open-day') ?? 'null')) as {
    day: string; requestedAtMs: number; requestId: string;
  };
  const requestedDate = new Date(request.requestedAtMs);
  const expectedDay = `${requestedDate.getFullYear()}-${String(requestedDate.getMonth() + 1).padStart(2, '0')}-${String(requestedDate.getDate()).padStart(2, '0')}`;
  expect(request.day).toBe(expectedDay);
  expect(request.requestId).toBeTruthy();
  expect(await page.evaluate(() => window.uxSettingsTest.blocked)).toEqual([]);
});

test('a protected Share close failure stays visible and retryable inside the modal', async ({ page }) => {
  await openApp(page, 'share-close-error');
  await page.getByRole('button', { name: 'Share my stats', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'share my stats' });
  await expect(dialog).toBeVisible();

  await dialog.getByRole('button', { name: 'Close', exact: true }).click();
  await expect(dialog).toBeVisible();
  const closeError = dialog.getByTestId('share-close-error');
  await expect(closeError).toContainText('Could not change the Share view');
  await expect(closeError).toBeInViewport();
  await expect(dialog.getByRole('button', { name: 'Retry close' })).toBeVisible();
  expect(await page.locator('.one-bar-widget > [role="alert"]').count()).toBe(0);

  await dialog.getByRole('button', { name: 'Retry close' }).click();
  await expect(dialog).toHaveCount(0);
  const flowUpdates = await page.evaluate(() => window.uxSettingsTest.calls
    .filter(call => call.command === 'set_app_reminder_flow')
    .map(call => (call.args as { flow?: string | null }).flow));
  expect(flowUpdates.slice(-3)).toEqual(['share', null, null]);
  expect(await page.evaluate(() => window.uxSettingsTest.blocked)).toEqual([]);
});

test('Settings tabs use manual keyboard activation and More exposes the Health guides', async ({ page }) => {
  await openSettings(page);
  const profiles = page.getByRole('tab', { name: 'Profiles' });
  const calibration = page.getByRole('tab', { name: 'Calibr.' });
  const notifications = page.getByRole('tab', { name: 'Notif.' });
  const more = page.getByRole('tab', { name: 'More' });
  const debug = page.getByRole('tab', { name: 'Debug' });

  await profiles.focus();
  await page.keyboard.press('ArrowRight');
  await expect(calibration).toBeFocused();
  await expect(profiles).toHaveAttribute('aria-selected', 'true');
  await expect(calibration).toHaveAttribute('aria-selected', 'false');
  await page.keyboard.press('Space');
  await expect(calibration).toHaveAttribute('aria-selected', 'true');
  await page.keyboard.press('ArrowRight');
  await expect(notifications).toBeFocused();
  await expect(calibration).toHaveAttribute('aria-selected', 'true');
  await page.keyboard.press('End');
  await expect(debug).toBeFocused();
  await expect(calibration).toHaveAttribute('aria-selected', 'true');
  await page.keyboard.press('ArrowLeft');
  await expect(more).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(more).toHaveAttribute('aria-selected', 'true');

  const panel = page.getByRole('tabpanel');
  await expect(panel).toHaveAttribute('aria-labelledby', await more.getAttribute('id'));
  await expect(page.getByRole('heading', { name: 'Optional health readings' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Privacy guide' })).toHaveAttribute(
    'href', 'https://github.com/zentala/MoveUp/blob/main/docs/PRIVACY.md',
  );
  await expect(page.getByRole('link', { name: 'Remote Display guide' })).toHaveAttribute(
    'href', 'https://github.com/zentala/MoveUp/blob/main/docs/REMOTE_DISPLAY.md',
  );
  const privacy = page.getByRole('link', { name: 'Privacy guide' });
  const remoteDisplay = page.getByRole('link', { name: 'Remote Display guide' });
  const privacyStyle = await privacy.evaluate(link => ({
    color: getComputedStyle(link).color,
    helperColor: getComputedStyle(link.parentElement!).color,
    background: getComputedStyle(link.closest('.settings-panel__section')!).backgroundColor,
    decoration: getComputedStyle(link).textDecorationLine,
  }));
  expect(privacyStyle.color).toBe(privacyStyle.helperColor);
  const luminance = (color: string) => {
    const [r, g, b] = color.match(/[\d.]+/g)!.slice(0, 3).map(channel => {
      const value = Number(channel) / 255;
      return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
    });
    return 0.2126 * r + 0.7152 * g + 0.0722 * b;
  };
  const [lighter, darker] = [luminance(privacyStyle.color), luminance(privacyStyle.background)].sort((a, b) => b - a);
  expect((lighter + 0.05) / (darker + 0.05)).toBeGreaterThanOrEqual(4.5);
  expect(privacyStyle.decoration).toContain('underline');
  await privacy.focus();
  await page.keyboard.press('Tab');
  await expect(remoteDisplay).toBeFocused();
  const focusStyle = await remoteDisplay.evaluate(link => ({
    visible: link.matches(':focus-visible'),
    outlineStyle: getComputedStyle(link).outlineStyle,
    outlineWidth: getComputedStyle(link).outlineWidth,
  }));
  expect(focusStyle.visible).toBe(true);
  expect(focusStyle.outlineStyle).toBe('solid');
  expect(parseFloat(focusStyle.outlineWidth)).toBeGreaterThanOrEqual(2);
});

test('the existing Welcome entry queues its test in the main reminder flow and preserves dismissal choice', async ({ page }, info) => {
  await openWelcome(page, 'welcome-queued');
  const previewButton = page.getByRole('button', { name: 'Test reminder' });
  await previewButton.click();

  await expect(page.getByRole('status')).toHaveText('Test preview queued in the main window. Return to status to view it.');
  await expect(page.getByRole('status')).toBeInViewport();
  await expect(page.getByText('This is a test reminder. Your ergonomic session is unchanged.')).toHaveCount(0);
  await expect(page.getByTestId('one-bar-widget')).toHaveCount(0);
  await page.screenshot({ path: info.outputPath('welcome-test-preview-queued-480x420.png') });

  await page.getByRole('checkbox', { name: "Don't show this at next launch" }).check();
  await page.getByRole('button', { name: 'Open MoveUp' }).click();
  const commands = await page.evaluate(() => window.uxSettingsTest.calls.map(call => call.command));
  expect(commands).toContain('trigger_test_notification');
  expect(await page.evaluate(() => window.uxSettingsTest.calls.find(call => call.command === 'dismiss_welcome')?.args))
    .toMatchObject({ dontShowAgain: true });
  expect(await page.evaluate(() => window.uxSettingsTest.blocked)).toEqual([]);
});

test('the existing Welcome entry reports preview errors without blocking dismissal', async ({ page }) => {
  await openWelcome(page, 'welcome-error');
  await page.getByRole('button', { name: 'Test reminder' }).click();

  await expect(page.getByRole('alert')).toContainText('Could not start test preview');
  await expect(page.getByRole('alert')).toContainText('Rate limited: wait before testing again');
  await expect(page.getByRole('button', { name: 'Open MoveUp' })).toBeEnabled();
  await page.getByRole('button', { name: 'Open MoveUp' }).click();
  expect(await page.evaluate(() => window.uxSettingsTest.calls.find(call => call.command === 'dismiss_welcome')?.args))
    .toMatchObject({ dontShowAgain: false });
  expect(await page.evaluate(() => window.uxSettingsTest.blocked)).toEqual([]);
});

test('Welcome keeps Open MoveUp retryable after an activation failure', async ({ page }) => {
  await openWelcome(page, 'welcome-open-error');
  const openMoveUp = page.getByRole('button', { name: 'Open MoveUp' });
  await openMoveUp.click();
  await expect(page.getByRole('alert')).toContainText('MoveUp could not be focused. Try again.');
  await expect(openMoveUp).toBeEnabled();
  await openMoveUp.click();
  await expect(page.getByRole('alert')).toHaveCount(0);
  const attempts = await page.evaluate(() => window.uxSettingsTest.calls.filter(call => call.command === 'dismiss_welcome').length);
  expect(attempts).toBe(2);
  expect(await page.evaluate(() => window.uxSettingsTest.calls.find(call => call.command === 'dismiss_welcome')?.args))
    .toMatchObject({ dontShowAgain: false });
  expect(await page.evaluate(() => window.uxSettingsTest.blocked)).toEqual([]);
});

test('Welcome keeps the requested preference after a save failure so it can be changed and retried', async ({ page }) => {
  await openWelcome(page, 'welcome-preference-error');
  const checkbox = page.getByRole('checkbox', { name: "Don't show this at next launch" });
  await checkbox.check();
  await page.getByRole('button', { name: 'Open MoveUp' }).click();
  await expect(page.getByRole('alert')).toContainText('Could not save the Welcome preference. Try again.');
  await expect(checkbox).toBeChecked();

  await checkbox.uncheck();
  await page.getByRole('button', { name: 'Open MoveUp' }).click();
  await expect(page.getByRole('alert')).toHaveCount(0);
  const attempts = await page.evaluate(() => window.uxSettingsTest.calls.filter(call => call.command === 'dismiss_welcome'));
  expect(attempts).toHaveLength(2);
  expect(attempts.map(call => call.args)).toEqual([{ dontShowAgain: true }, { dontShowAgain: false }]);
  expect(await page.evaluate(() => window.uxSettingsTest.blocked)).toEqual([]);
});

test('420x240 calibration alert names both invalid ranges and tracks correction field by field', async ({ page }, info) => {
  await openSettings(page, 'invalid-both');
  await page.getByRole('tab', { name: 'Calibr.' }).click();

  const sitting = page.getByLabel('Sitting height (mm)');
  const standing = page.getByLabel('Standing height (mm)');
  const alert = page.locator('.settings-panel__validation-error');
  await expect(alert).toContainText('Sitting height must be between 400 and 900 mm');
  await expect(alert).toContainText('standing height must be between 900 and 1400 mm');
  await expect(sitting).toHaveAttribute('aria-invalid', 'true');
  await expect(standing).toHaveAttribute('aria-invalid', 'true');
  await expect(page.locator('.settings-toolbar__save')).toBeDisabled();
  await alert.evaluate(element => element.scrollIntoView({ block: 'end' }));
  await expect(alert).toBeInViewport();
  await page.screenshot({ path: info.outputPath('settings-both-calibration-errors-420x240.png') });

  await sitting.fill('750');
  await expect(alert).toHaveText('Standing height must be between 900 and 1400 mm');
  await expect(sitting).toHaveAttribute('aria-invalid', 'false');
  await expect(standing).toHaveAttribute('aria-invalid', 'true');
  await standing.fill('1100');
  await expect(alert).toHaveCount(0);
  await expect(page.locator('.settings-toolbar__save')).toBeEnabled();
});

test('More → Remote keeps loading and IPC failure distinct from a genuinely off relay', async ({ page }, info) => {
  await openSettings(page, 'relay-status-error');
  await page.getByRole('tab', { name: 'More' }).click();

  const heading = page.getByRole('heading', { name: 'Remote access' });
  await heading.scrollIntoViewIfNeeded();
  const state = page.getByTestId('relay-state');
  await expect(state).toHaveText('Checking relay status…');
  await expect(page.getByLabel('Licence key')).toHaveCount(0);
  await expect(state).not.toHaveText(/Relay is off/);
  await state.scrollIntoViewIfNeeded();
  await expect.poll(() => page.evaluate(() =>
    window.uxSettingsTest.calls.some(call => call.command === 'get_relay_status'),
  )).toBe(true);
  await page.screenshot({ path: info.outputPath('remote-status-loading-420x240.png') });

  await page.evaluate(() => window.uxSettingsTest.rejectRelayStatus());
  const statusError = page.getByTestId('relay-status-error');
  await expect(statusError).toContainText('Could not load relay status');
  await expect(statusError).toContainText('state is unknown');
  await expect(page.getByLabel('Licence key')).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Retry relay status' })).toBeEnabled();
  await statusError.scrollIntoViewIfNeeded();
  await page.screenshot({ path: info.outputPath('remote-status-error-420x240.png') });

  await page.getByRole('button', { name: 'Retry relay status' }).click();
  await expect(state).toHaveText('Relay is off');
  await expect(page.getByLabel('Licence key')).toBeVisible();
  await expect(page.getByText(/MoveUp does not report whether the server is listening or which adapter address to use/)).toBeVisible();
  await expect(page.getByText(/This build does not upload usage data/)).toBeAttached();
  await expect(page.getByRole('button', { name: 'Enable remote access' })).toBeDisabled();
  expect(await page.evaluate(() => window.uxSettingsTest.calls.filter(call => call.command === 'get_relay_status').length)).toBeGreaterThanOrEqual(2);
  expect(await page.evaluate(() => window.uxSettingsTest.calls.filter(call => call.command === 'relay_register' || call.command === 'relay_list_viewers'))).toEqual([]);
  expect(await page.evaluate(() => window.uxSettingsTest.blocked)).toEqual([]);
  await state.scrollIntoViewIfNeeded();
  await page.screenshot({ path: info.outputPath('remote-status-known-off-420x240.png') });
});

declare global {
  interface Window {
    uxSettingsTest: {
      calls: { command: string; args: unknown }[];
      blocked: string[];
      saveAttempts: () => number;
      failNextDashboardRefresh: () => void;
      dashboardRefreshPending: () => boolean;
      rejectDashboardRefresh: () => void;
      rejectNextSettingsLoad: () => void;
      rejectPendingSave: () => void;
      publishReminder: () => Promise<void>;
    };
  }
}
