import { test, expect, type Page } from '@playwright/test';

async function openApp(page: Page, scenario = 'normal') {
  await page.setViewportSize({ width: 420, height: 240 });
  await page.route('**/*', async route => {
    const url = new URL(route.request().url());
    if (url.origin !== 'http://127.0.0.1:15431' || url.pathname.startsWith('/display')) return route.abort();
    if (url.pathname === '/') {
      const response = await route.fetch();
      return route.fulfill({ response, body: (await response.text()).replace('/src/main.tsx', '/tests/e2e/application-reminders.fixture.ts') });
    }
    return route.continue();
  });
  await page.goto(`/?reminder-fixture=${scenario}`);
  if (scenario === 'race') await expect(page.getByTestId('application-reminder')).toBeVisible();
  else await expect(page.getByTestId('one-bar-widget')).toBeVisible();
  if (scenario !== 'remote' && scenario !== 'other-window') {
    await expect.poll(() => page.evaluate(() => window.reminderTest.calls.filter(call => call.command === 'set_app_reminder_flow').length)).toBeGreaterThan(0);
  }
}
const notice = (page: Page) => page.getByTestId('application-reminder');
async function geometry(page: Page, stress = false) {
  const problems = await page.evaluate(({ stress }) => {
    const issues: string[] = [];
    const section = document.querySelector<HTMLElement>('.application-reminder')!;
    const box = section.getBoundingClientRect();
    if (box.left < 0 || box.top < 0 || box.right > innerWidth + 1 || box.bottom > innerHeight + 1) issues.push('notice outside main');
    for (const element of section.querySelectorAll<HTMLElement>('h1, header, footer, button')) {
      const r = element.getBoundingClientRect();
      if (r.left < 0 || r.top < 0 || r.right > innerWidth + 1 || r.bottom > innerHeight + 1) issues.push(`clipped ${element.tagName}`);
      if (element.tagName === 'BUTTON') {
        if (r.height < 24 || r.width < 24 || element.scrollWidth > element.clientWidth + 1) issues.push(`unreadable control ${element.textContent}`);
        if (!element.contains(document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2))) issues.push('occluded control');
      }
    }
    const body = section.querySelector<HTMLElement>('.application-reminder__body')!;
    if (!stress && body.scrollHeight > body.clientHeight + 1) issues.push('ordinary body unexpectedly scrolls');
    if (section.scrollHeight > section.clientHeight + 1) issues.push('whole notice scrolls');
    if (body.scrollWidth > body.clientWidth + 1) issues.push('horizontal body clipping');
    return issues;
  }, { stress });
  expect(problems).toEqual([]);
}

for (const category of ['info', 'warning', 'urgent', 'success'] as const) {
  test(`actual main ${category} has readable complete controls and opaque category surface`, async ({ page }, info) => {
    await openApp(page);
    await page.getByRole('button', { name: 'Settings', exact: true }).click();
    await page.getByRole('tab', { name: 'Notif.' }).click();
    await page.getByLabel('Test appearance').selectOption(category);
    await page.getByRole('button', { name: 'Test reminder', exact: true }).click();
    await expect(page.getByRole('status')).toContainText('Test queued');
    await page.getByRole('button', { name: 'Back', exact: true }).click();
    await expect(notice(page)).toBeVisible();
    await expect(notice(page)).toContainText('Test ·');
    await geometry(page);
    const colors = await notice(page).evaluate(element => ({ background: getComputedStyle(element).backgroundColor, color: getComputedStyle(element).color }));
    expect(colors.background).not.toBe('rgba(0, 0, 0, 0)');
    const contrast = await notice(page).evaluate(element => {
      const canvas = document.createElement('canvas'); canvas.width = canvas.height = 1;
      const context = canvas.getContext('2d')!;
      const luminance = (color: string) => {
        context.fillStyle = color; context.fillRect(0, 0, 1, 1);
        const rgb = Array.from(context.getImageData(0, 0, 1, 1).data).slice(0, 3).map(value => {
          const v = value / 255; return v <= .04045 ? v / 12.92 : ((v + .055) / 1.055) ** 2.4;
        });
        return rgb[0] * .2126 + rgb[1] * .7152 + rgb[2] * .0722;
      };
      const background = luminance(getComputedStyle(element).backgroundColor);
      return [...element.querySelectorAll('h1, p, .application-reminder__category, [role="status"]')].map(text => {
        const foreground = luminance(getComputedStyle(text).color);
        return (Math.max(foreground, background) + .05) / (Math.min(foreground, background) + .05);
      });
    });
    expect(Math.min(...contrast)).toBeGreaterThanOrEqual(4.5);
    await page.getByRole('button', { name: 'Dismiss', exact: true }).hover();
    await page.screenshot({ path: info.outputPath(`main-${category}-420x240.png`) });
    await page.getByRole('button', { name: 'Dismiss', exact: true }).click();
    await expect(notice(page)).toHaveCount(0); await expect(page.getByTestId('one-bar-widget')).toBeVisible();
    expect(await page.evaluate(() => window.reminderTest.applied())).toBe(0);
    expect(await page.evaluate(() => window.reminderTest.blocked)).toEqual([]);
  });
}

test('actual App keeps a Settings draft and queues selected Test with one shared cooldown', async ({ page }, info) => {
  await openApp(page); await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await page.getByRole('tab', { name: 'Calibr.' }).click();
  await page.getByLabel('Sitting height (mm)').fill('760');
  await page.evaluate(() => window.reminderTest.publish('urgent'));
  await expect(page.getByLabel('Sitting height (mm)')).toHaveValue('760');
  await expect(notice(page)).toHaveCount(0);
  await page.getByRole('tab', { name: 'Notif.' }).click();
  await page.getByLabel('Test appearance').selectOption('warning');
  await page.getByRole('button', { name: 'Test reminder', exact: true }).click();
  await expect(page.getByRole('status')).toContainText('Test queued');
  const controls = await page.locator('.reminder-preview__controls').evaluate(row => [...row.querySelectorAll('select, button')].map(child => child.getBoundingClientRect().height));
  expect(controls[0]).toBe(controls[1]);
  const queued = await page.getByRole('status').boundingBox();
  const settingsBody = await page.locator('.settings-panel__body').boundingBox();
  expect(queued!.y + queued!.height).toBeLessThanOrEqual(Math.min(240, settingsBody!.y + settingsBody!.height) + 1);
  await page.screenshot({ path: info.outputPath('settings-queued-preview-420x240.png') });
  await page.getByLabel('Test appearance').selectOption('success');
  await page.getByRole('button', { name: 'Test reminder', exact: true }).click();
  await expect(page.getByRole('status')).toContainText('Wait 60 seconds');
  await page.getByRole('tab', { name: 'Calibr.' }).click(); await expect(page.getByLabel('Sitting height (mm)')).toHaveValue('760');
  await page.getByRole('button', { name: 'Back', exact: true }).click();
  await expect(notice(page)).toHaveClass(/warning/); await expect(notice(page)).toContainText('Test');
  expect(await page.evaluate(() => window.reminderTest.calls.filter(call => call.command === 'update_settings'))).toHaveLength(0);
});

test('actual App retains the same open Share dialog when a reminder arrives', async ({ page }, info) => {
  await openApp(page); await page.getByRole('button', { name: 'Share my stats', exact: true }).click();
  const dialog = page.getByRole('dialog', { name: 'share my stats' }); await expect(dialog).toBeVisible();
  await dialog.evaluate(element => { (element as HTMLElement).dataset.fixtureInstance = 'original'; });
  await page.evaluate(() => window.reminderTest.publish('urgent'));
  await expect(dialog).toHaveAttribute('data-fixture-instance', 'original'); await expect(notice(page)).toHaveCount(0);
  await page.screenshot({ path: info.outputPath('share-protected-420x240.png') });
  await page.getByRole('button', { name: 'Close', exact: true }).click();
  await expect(dialog).toHaveCount(0); await expect(notice(page)).toBeVisible();
});

test('actual App recovers persisted ACK failure after reload with generic Retry and rejects late events', async ({ page }, info) => {
  await openApp(page); await page.evaluate(() => window.reminderTest.publish('urgent'));
  const old = await page.evaluate(() => window.reminderTest.snapshot());
  await page.evaluate(() => window.reminderTest.failNextSave());
  await page.getByRole('button', { name: 'Snooze', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('Could not save');
  await geometry(page, true);
  await page.screenshot({ path: info.outputPath('main-save-error-retry-420x240.png') });
  await page.reload(); await expect(page.getByRole('button', { name: 'Retry saving' })).toBeVisible();
  await page.screenshot({ path: info.outputPath('main-remounted-retry-420x240.png') });
  await page.getByRole('button', { name: 'Retry saving' }).click(); await expect(notice(page)).toHaveCount(0);
  expect(await page.evaluate(() => window.reminderTest.applied())).toBe(1);
  await page.evaluate(old => window.reminderTest.late(old), old); await expect(notice(page)).toHaveCount(0);
  const actions = await page.evaluate(() => window.reminderTest.calls.filter(call => call.command === 'acknowledge_app_reminder').map(call => (call.args as { action: string }).action));
  expect(actions).toEqual(['retry']);
});

test('automatic message rendering leaves DOM focus alone and does not expire', async ({ page }, info) => {
  await openApp(page); const focused = await page.evaluate(() => document.activeElement?.tagName);
  await page.evaluate(() => window.reminderTest.publish('warning'));
  await expect(notice(page)).toBeVisible();
  expect(await page.evaluate(() => document.activeElement?.tagName)).toBe(focused);
  await page.waitForTimeout(1200); await expect(notice(page)).toBeVisible();
  await page.screenshot({ path: info.outputPath('main-automatic-no-autofocus-420x240.png') });
});

test('event-before-snapshot and intervening client claim preserve the current main reminder', async ({ page }) => {
  await openApp(page, 'race'); await expect(notice(page)).toBeVisible();
  await expect(notice(page)).toContainText('Time to change position');
  await page.getByRole('button', { name: 'Dismiss', exact: true }).click(); await expect(notice(page)).toHaveCount(0);
  await page.goto('/?reminder-fixture=claim-race');
  await page.evaluate(() => window.reminderTest.publish('warning')); await expect(notice(page)).toBeVisible();
});

test('constrained enlarged text scrolls only the labelled body and keeps whole heading/actions reachable', async ({ page }, info) => {
  await openApp(page); await page.setViewportSize({ width: 300, height: 240 });
  await page.evaluate(() => window.reminderTest.publish('urgent', false, true));
  await page.addStyleTag({ content: '.application-reminder h1 { font-size: 26px; } .application-reminder__body { font-size: 18px; } .application-reminder .btn { font-size: 14px; }' });
  await geometry(page, true);
  await expect(page.getByRole('region', { name: 'Reminder details' })).toBeVisible();
  await page.getByRole('button', { name: 'Snooze', exact: true }).focus();
  await page.screenshot({ path: info.outputPath('main-stress-300x240.png') });
});

for (const scenario of ['remote', 'other-window']) {
  test(`${scenario} never subscribes to or controls main reminders`, async ({ page }) => {
    await openApp(page, scenario); await expect(notice(page)).toHaveCount(0);
    expect(await page.evaluate(() => window.reminderTest.calls.filter(call => /app_reminder|trigger_test_notification/.test(call.command)))).toEqual([]);
  });
}

for (const width of [420, 300]) {
  test(`active presentation recovery keeps the current ID and has explicit reconnect at ${width}x240`, async ({ page }, info) => {
    await openApp(page); await page.setViewportSize({ width, height: 240 });
    await page.evaluate(() => window.reminderTest.publish('warning'));
    const id = await page.evaluate(() => window.reminderTest.snapshot().active!.id);
    await page.evaluate(() => window.reminderTest.presentationFailure());
    await expect(page.getByRole('alert')).toContainText('Retry the connection');
    const reconnect = page.getByRole('button', { name: 'Retry connection', exact: true });
    await expect(reconnect).toBeVisible(); await geometry(page, true); await reconnect.focus();
    await page.screenshot({ path: info.outputPath(`main-presentation-retry-${width}x240.png`) });
    await reconnect.click();
    await expect(notice(page)).toBeVisible(); await expect(page.getByRole('alert')).toHaveCount(0);
    expect(await page.evaluate(() => window.reminderTest.snapshot().active!.id)).toBe(id);
    expect(await page.evaluate(() => window.reminderTest.calls.filter(call => call.command === 'acknowledge_app_reminder'))).toEqual([]);
    expect(await page.evaluate(() => window.reminderTest.applied())).toBe(0);
    await geometry(page);
    await page.screenshot({ path: info.outputPath(`main-reconnected-${width}x240.png`) });
  });
}

test('real non-test halfway Praise uses a generic positive label without claiming the full goal', async ({ page }, info) => {
  await openApp(page); await page.evaluate(() => window.reminderTest.praise());
  await expect(page.getByRole('heading', { name: 'Halfway through your standing goal!' })).toBeVisible();
  await expect(notice(page)).toContainText('Progress'); await expect(notice(page)).toContainText('7m 30s / 15m 0s');
  await expect(notice(page)).not.toContainText('Goal reached'); await expect(notice(page)).not.toContainText('Test ·');
  await geometry(page);
  await page.screenshot({ path: info.outputPath('main-real-praise-halfway-420x240.png') });
});
