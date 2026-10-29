import { test, expect, type Page } from '@playwright/test';

async function openShare(page: Page, scenario = 'normal') {
  await page.route('**/*', async route => {
    const url = new URL(route.request().url());
    if (url.origin !== 'http://127.0.0.1:15431' || url.pathname.startsWith('/display')) return route.abort();
    if (url.pathname === '/') {
      const response = await route.fetch();
      return route.fulfill({ response, body: (await response.text()).replace('/src/main.tsx', '/tests/e2e/share-popup.fixture.ts') });
    }
    return route.continue();
  });
  await page.goto(`/?share-fixture=${scenario}`);
  await expect.poll(() => page.evaluate(() => window.shareTest.healthLoaded)).toBe(true);
  await expect(page.getByTestId('one-bar-widget')).toBeVisible();
  await expect(page.getByText('72 cm')).toBeVisible();
  await expect(page.getByTestId('health-widget')).toHaveCount(0);
  await page.getByRole('button', { name: 'Share my stats', exact: true }).click();
  await expect(page.locator('.share-modal')).toBeVisible();
  await expect(page.locator('.share-card__row')).toHaveCount(4);
}

/** Inspect real browser text fragments, every clipping ancestor, and viewport. */
async function layoutIssues(page: Page, allowStatsScroll = false) {
  return page.evaluate(({ allowStatsScroll }) => {
    const issues: string[] = [];
    const dialog = document.querySelector('.share-modal')!;
    const viewport = { left: 0, top: 0, right: innerWidth, bottom: innerHeight };
    const inside = (a: DOMRect | typeof viewport, b: DOMRect | typeof viewport) =>
      a.left >= b.left - 1 && a.top >= b.top - 1 && a.right <= b.right + 1 && a.bottom <= b.bottom + 1;
    const scroller = document.querySelector('.share-card');
    const elements = [dialog, ...dialog.querySelectorAll<HTMLElement>('button, .share-modal__header, .share-card, .share-actions')];
    for (const element of elements) {
      if (!inside(element.getBoundingClientRect(), viewport)) issues.push(`outside viewport: ${element.className}`);
    }
    for (const button of dialog.querySelectorAll('button')) {
      const r = button.getBoundingClientRect();
      if (r.width < 24 || r.height < 24) issues.push(`small hit target: ${button.textContent}`);
      if (!button.contains(document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2))) issues.push(`occluded control: ${button.textContent}`);
      if (button.scrollWidth > button.clientWidth + 1 || button.scrollHeight > button.clientHeight + 1) issues.push(`control overflow: ${button.textContent}`);
    }
    const buttons = [...dialog.querySelectorAll('button')];
    for (let a = 0; a < buttons.length; a++) for (let b = a + 1; b < buttons.length; b++) {
      const x = buttons[a].getBoundingClientRect(), y = buttons[b].getBoundingClientRect();
      if (Math.min(x.right, y.right) > Math.max(x.left, y.left) && Math.min(x.bottom, y.bottom) > Math.max(x.top, y.top)) issues.push('overlapping controls');
    }
    const walker = document.createTreeWalker(dialog, NodeFilter.SHOW_TEXT);
    for (let node = walker.nextNode(); node; node = walker.nextNode()) {
      if (!node.textContent?.trim()) continue;
      if (parseFloat(getComputedStyle(node.parentElement!).fontSize) < 10) issues.push(`small text: ${node.textContent}`);
      const range = document.createRange(); range.selectNodeContents(node);
      for (const rect of range.getClientRects()) {
        const inScroller = !!scroller?.contains(node);
        if (!(allowStatsScroll && inScroller) && !inside(rect, viewport)) issues.push(`text outside viewport: ${node.textContent}`);
        for (let parent = node.parentElement; parent; parent = parent.parentElement) {
          const style = getComputedStyle(parent);
          const box = parent.getBoundingClientRect();
          if (/(hidden|clip|auto|scroll)/.test(style.overflowX) && (rect.left < box.left - 1 || rect.right > box.right + 1)) issues.push(`text clipped horizontally: ${node.textContent}`);
          if (!(allowStatsScroll && inScroller) && /(hidden|clip|auto|scroll)/.test(style.overflowY) && (rect.top < box.top - 1 || rect.bottom > box.bottom + 1)) issues.push(`text clipped vertically: ${node.textContent}`);
        }
      }
    }
    if (document.documentElement.scrollWidth > innerWidth || document.documentElement.scrollHeight > innerHeight) issues.push('document overflow');
    if (!allowStatsScroll && scroller && scroller.scrollHeight > scroller.clientHeight + 1) issues.push('ordinary stats scroll');
    return issues;
  }, { allowStatsScroll });
}

async function brokenActionWords(page: Page) {
  return page.locator('.share-actions').evaluate(footer => {
    const broken: string[] = [];
    const walker = document.createTreeWalker(footer, NodeFilter.SHOW_TEXT);
    for (let node = walker.nextNode(); node; node = walker.nextNode()) {
      for (const word of node.textContent!.matchAll(/\S+/g)) {
        const range = document.createRange();
        range.setStart(node, word.index!); range.setEnd(node, word.index! + word[0].length);
        if (range.getClientRects().length !== 1) broken.push(word[0]);
      }
    }
    return broken;
  });
}

test('320x240 enlarged action words stay whole before and after Copy', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 320, height: 240 });
  await openShare(page, 'stress');
  await page.addStyleTag({ content: 'html { font-size: 200%; }' });
  await page.screenshot({ path: testInfo.outputPath('before-copy.png') });
  expect(await brokenActionWords(page)).toEqual([]);
  expect(await layoutIssues(page, true)).toEqual([]);
  await page.getByRole('button', { name: 'Copy Text' }).click();
  await expect(page.getByRole('button', { name: 'Copied!' })).toBeVisible();
  await page.screenshot({ path: testInfo.outputPath('after-copy.png') });
  expect(await brokenActionWords(page)).toEqual([]);
  expect(await layoutIssues(page, true)).toEqual([]);
});

test('ordinary 420x240 complete Share geometry', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 420, height: 240 });
  await openShare(page);
  await expect(page.locator('.share-card')).toContainText('42%');
  await expect(page.locator('.share-card')).toContainText('+12 pts');
  await page.screenshot({ path: testInfo.outputPath('share-420x240.png') });
  expect(await layoutIssues(page)).toEqual([]);
});

for (const size of [{ width: 420, height: 240 }, { width: 320, height: 240 }, { width: 420, height: 200 }]) {
  for (const scenario of ['normal', 'empty', 'stress']) {
    for (const enlarged of [false, true]) {
      test(`${size.width}x${size.height} ${scenario} ${enlarged ? '200% text' : 'default text'} remains reachable`, async ({ page }, testInfo) => {
        await page.setViewportSize(size);
        await openShare(page, scenario);
        if (enlarged) await page.addStyleTag({ content: 'html { font-size: 200%; }' });
        await expect(page.getByRole('dialog', { name: 'share my stats' })).toBeVisible();
        for (const name of ['Close', 'Copy Text', 'Post on X', 'Reddit']) await expect(page.getByRole('button', { name, exact: true })).toBeVisible();
        for (const label of ['Standing', 'Position changes', 'Longest session', 'Score']) await expect(page.locator('.share-card__label', { hasText: label })).toHaveCount(1);
        await expect(page.locator('.share-card')).toContainText('tracked');
        await expect(page.locator('.share-card')).toContainText('Tracked by MoveUp');
        if (scenario === 'stress') {
          for (const value of ['123456789', '123456h 59m', '-123456789 pts']) await expect(page.locator('.share-card')).toContainText(value);
        } else if (scenario === 'empty') await expect(page.locator('.share-card__value').filter({ hasText: '—' })).toHaveCount(2);
        expect(await layoutIssues(page, true)).toEqual([]);
        expect(await brokenActionWords(page)).toEqual([]);
        // Traverse every actual text fragment, including wrapped long values,
        // within the bounded statistics scroller. Hidden/ellipsis is a failure.
        expect(await page.locator('.share-card').evaluate(card => {
          const problems: string[] = [];
          const walker = document.createTreeWalker(card, NodeFilter.SHOW_TEXT);
          for (let node = walker.nextNode(); node; node = walker.nextNode()) {
            if (!node.textContent?.trim()) continue;
            const style = getComputedStyle(node.parentElement!);
            if (style.display === 'none' || style.visibility !== 'visible' || style.textOverflow === 'ellipsis') problems.push(`hidden: ${node.textContent}`);
            const range = document.createRange(); range.selectNodeContents(node);
            const count = range.getClientRects().length;
            if (!count) problems.push(`no text geometry: ${node.textContent}`);
            for (let i = 0; i < count; i++) {
              const before = range.getClientRects()[i];
              card.scrollTop += before.top - card.getBoundingClientRect().top - 4;
              const rect = range.getClientRects()[i], box = card.getBoundingClientRect();
              if (rect.left < box.left - 1 || rect.right > box.right + 1 || rect.top < box.top - 1 || rect.bottom > box.bottom + 1) problems.push(`unreachable: ${node.textContent}`);
            }
          }
          card.scrollTop = 0;
          return problems;
        })).toEqual([]);
        await page.getByRole('button', { name: 'Copy Text' }).click();
        await expect(page.getByRole('button', { name: 'Copied!' })).toBeVisible();
        expect(await brokenActionWords(page)).toEqual([]);
        expect(await layoutIssues(page, true)).toEqual([]);
        await page.screenshot({ path: testInfo.outputPath('share.png') });
      });
    }
  }
}

test('clipboard and exact social destinations preserve sharing semantics', async ({ page }) => {
  await page.setViewportSize({ width: 420, height: 240 });
  await openShare(page);
  const expected = 'My desk stats today:\n🧍 Standing: 42%\n🔄 Position changes: 7\n⏱️ Longest session: 30m\n🏆 Score: +12 points\n\nTracked by MoveUp — desk.zentala.io';
  await page.getByRole('button', { name: 'Copy Text' }).click();
  expect(await page.evaluate(() => window.shareTest.copied)).toBe(expected);
  await page.getByRole('button', { name: 'Post on X' }).click();
  await page.getByRole('button', { name: 'Reddit', exact: true }).click();
  const urls = await page.evaluate(() => window.shareTest.opened);
  expect(urls).toHaveLength(2);
  const x = new URL(urls[0]), reddit = new URL(urls[1]);
  expect(x.origin + x.pathname).toBe('https://twitter.com/intent/tweet');
  expect(x.searchParams.get('text')).toBe(expected);
  expect(x.searchParams.get('url')).toBe('https://desk.zentala.io');
  expect(reddit.origin + reddit.pathname).toBe('https://reddit.com/submit');
  expect(reddit.searchParams.get('title')).toBe('My desk ergonomics stats today — tracked by MoveUp');
  expect(reddit.searchParams.get('text')).toBe(expected);
  expect(reddit.searchParams.get('url')).toBe('https://desk.zentala.io');
  expect(reddit.searchParams.get('selftext')).toBe('true');
});

test('both copy methods can fail visibly and Retry can recover', async ({ page }, info) => {
  await page.setViewportSize({ width: 420, height: 240 });
  await openShare(page, 'clipboard-fail');
  await page.getByRole('button', { name: 'Copy Text' }).click();
  const copyError = page.getByRole('alert');
  await expect(copyError).toHaveText('Copy failed.');

  const scaledSelectors = [
    '.share-modal__title', '.share-modal__copy-error', '.share-card__copy-guidance', '.share-card__time',
    '.share-card__label', '.share-card__value', '.share-card__watermark',
    '.share-card__manual-label', '.share-card__manual-copy', '.share-actions__btn',
  ];
  const baselineFonts = await page.evaluate(selectors => Object.fromEntries(
    selectors.map(selector => [selector, parseFloat(getComputedStyle(document.querySelector(selector)!).fontSize)]),
  ), scaledSelectors);
  await page.addStyleTag({ content: scaledSelectors.map(selector =>
    `${selector} { font-size: ${baselineFonts[selector] * 2}px !important; }`,
  ).join('\n') });
  const enlargedFonts = await page.evaluate(selectors => Object.fromEntries(
    selectors.map(selector => [selector, parseFloat(getComputedStyle(document.querySelector(selector)!).fontSize)]),
  ), scaledSelectors);
  for (const selector of scaledSelectors) expect(enlargedFonts[selector]).toBe(baselineFonts[selector] * 2);

  await expect(copyError).toHaveText('Copy failed.');
  const shareCard = page.locator('.share-card');
  await expect(shareCard).toContainText('Automatic copy failed. Retry or select the text below and copy it.');
  for (const value of ['42%', 'Position changes', '7', '30m', '+12 pts']) await expect(shareCard).toContainText(value);
  expect(await shareCard.evaluate(element => element.scrollHeight > element.clientHeight)).toBe(true);
  await shareCard.evaluate(element => { element.scrollTop = 0; });
  const errorTextGeometry = await copyError.evaluate(element => {
    const box = element.getBoundingClientRect();
    const range = document.createRange();
    range.selectNodeContents(element);
    const lines = [...range.getClientRects()].map(rect => ({ left: rect.left, top: rect.top, right: rect.right, bottom: rect.bottom }));
    return { box: { left: box.left, top: box.top, right: box.right, bottom: box.bottom }, lines };
  });
  expect(errorTextGeometry.lines.length).toBeGreaterThan(0);
  expect(errorTextGeometry.lines.every(line =>
    line.left >= errorTextGeometry.box.left - 1 && line.top >= errorTextGeometry.box.top - 1
      && line.right <= errorTextGeometry.box.right + 1 && line.bottom <= errorTextGeometry.box.bottom + 1
      && line.left >= 0 && line.top >= 0 && line.right <= 420 && line.bottom <= 240,
  )).toBe(true);
  await page.screenshot({ path: info.outputPath('share-copy-failed-200pct-immediate-420x240.png') });
  await expect(page.getByRole('button', { name: 'Copied!' })).toHaveCount(0);
  const manualCopy = page.getByRole('textbox', { name: 'Share text to copy manually' });
  await expect(manualCopy).toHaveValue(/Score: \+12 points/);
  expect(await page.evaluate(() => window.shareTest.copied)).toBe('');

  await page.keyboard.press('Shift+Tab');
  await expect(manualCopy).toBeFocused();
  const selectedManualText = await manualCopy.evaluate(element => ({
    start: (element as HTMLTextAreaElement).selectionStart,
    end: (element as HTMLTextAreaElement).selectionEnd,
    length: (element as HTMLTextAreaElement).value.length,
  }));
  expect(selectedManualText).toEqual({ start: 0, end: selectedManualText.length, length: selectedManualText.length });
  const manualGeometry = await manualCopy.evaluate(element => {
    const box = element.getBoundingClientRect();
    const scroller = document.querySelector('.share-card')!.getBoundingClientRect();
    return { box: { left: box.left, top: box.top, right: box.right, bottom: box.bottom }, scroller: { left: scroller.left, top: scroller.top, right: scroller.right, bottom: scroller.bottom } };
  });
  expect(manualGeometry.box.left).toBeGreaterThanOrEqual(manualGeometry.scroller.left - 1);
  expect(manualGeometry.box.top).toBeGreaterThanOrEqual(manualGeometry.scroller.top - 1);
  expect(manualGeometry.box.right).toBeLessThanOrEqual(manualGeometry.scroller.right + 1);
  expect(manualGeometry.box.bottom).toBeLessThanOrEqual(manualGeometry.scroller.bottom + 1);
  await page.keyboard.press('Shift+Tab');
  await expect(page.getByRole('button', { name: 'Retry copy' })).toBeInViewport();
  for (const name of ['Close', 'Copy Text', 'Post on X', 'Reddit']) await expect(page.getByRole('button', { name, exact: true })).toBeVisible();
  expect(await layoutIssues(page, true)).toEqual([]);
  await page.screenshot({ path: info.outputPath('share-copy-failed-200pct-manual-420x240.png') });

  await page.getByRole('button', { name: 'Retry copy' }).click();
  await expect(page.getByRole('button', { name: 'Copied!' })).toBeVisible();
  await expect(page.getByTestId('copy-error')).toHaveCount(0);
  expect(await page.evaluate(() => window.shareTest.copied)).toContain('Score: +12 points');
});

test('native modal contains keyboard focus, Escape and Close restore Share focus', async ({ page }) => {
  await page.setViewportSize({ width: 420, height: 240 });
  await openShare(page);
  const dialog = page.getByRole('dialog', { name: 'share my stats' });
  await expect(page.getByRole('button', { name: 'Close' })).toBeFocused();
  await page.keyboard.press('Shift+Tab');
  await expect(page.getByRole('button', { name: 'Reddit', exact: true })).toBeFocused();
  await page.keyboard.press('Tab');
  await expect(page.getByRole('button', { name: 'Close' })).toBeFocused();
  // Native inert background rejects programmatic focus as well as Tab.
  await page.getByRole('button', { name: 'Settings', exact: true }).evaluate(button => button.focus());
  await expect(page.getByRole('button', { name: 'Close' })).toBeFocused();
  for (let i = 0; i < 8; i++) {
    await page.keyboard.press('Tab');
    expect(await dialog.evaluate(element => element.contains(document.activeElement))).toBe(true);
    expect(await page.evaluate(() => {
      const focused = document.activeElement as HTMLElement;
      const rect = focused.getBoundingClientRect();
      return getComputedStyle(focused).outlineStyle !== 'none' && rect.left >= 4 && rect.top >= 4 && rect.right <= innerWidth - 4 && rect.bottom <= innerHeight - 4;
    })).toBe(true);
  }
  await page.keyboard.press('Escape');
  await expect(dialog).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Share my stats', exact: true })).toBeFocused();
  await page.keyboard.press('Enter');
  await page.getByRole('button', { name: 'Close' }).click();
  await expect(page.getByRole('button', { name: 'Share my stats', exact: true })).toBeFocused();
});
