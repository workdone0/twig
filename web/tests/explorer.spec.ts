import { test, expect, type Page } from '@playwright/test';
async function ready(page: Page) {
  await page.goto('/');
  await expect(page.getByRole('status')).toContainText('Example loaded');
}
async function open(page: Page, text: string, name = 'data.json') {
  await page.getByLabel('Choose JSON, YAML or HAR file').setInputFiles({ name, mimeType: 'application/octet-stream', buffer: Buffer.from(text) });
  await expect(page.getByRole('status')).toContainText('Ready.');
}
test('example navigation, search, paths and keyboard work with no external requests', async ({ page }) => {
  const requests: string[] = []; const errors: string[] = [];
  page.on('request', request => { requests.push(request.url()); expect(request.method()).toBe('GET'); expect(request.postData()).toBeNull(); });
  page.on('pageerror', error => errors.push(error.message));
  await ready(page);
  await page.getByRole('option').filter({ hasText: 'environments' }).click();
  await expect(page.getByLabel('Value preview')).toContainText('production');
  await page.getByRole('option').filter({ hasText: 'environments' }).click();
  await expect(page.getByRole('listbox', { name: 'environments entries' })).toBeVisible();
  await page.getByLabel('Jump to path').fill('.environments.production.region');
  await page.getByRole('button', { name: 'Go to path' }).click();
  await expect(page.getByLabel('Value preview')).toHaveText('"ap-south-1"');
  await page.getByLabel('Search keys and values').fill('Ada');
  await page.getByRole('button', { name: 'Next match' }).click();
  await expect(page.getByLabel('Value preview')).toHaveText('"Ada"');
  await page.getByRole('region', { name: 'Data explorer' }).focus();
  await page.keyboard.press('ArrowDown');
  await expect(page.getByLabel('Value preview')).toHaveText('"Engineering"');
  expect(errors).toEqual([]);
  expect(requests.every(url => url.startsWith('http://127.0.0.1:4321/'))).toBe(true);
});
test('file contents remain local, numeric precision and quoted paths survive', async ({ page }) => {
  await ready(page);
  const requests: string[] = [];
  page.on('request', request => requests.push(request.url()));
  await open(page, '{"secret-123456":"private-canary","a.b":18446744073709551615}');
  await page.getByLabel('Jump to path').fill('.["a.b"]');
  await page.getByRole('button', { name: 'Go to path' }).click();
  await expect(page.getByLabel('Value preview')).toHaveText('18446744073709551615');
  expect(requests.every(url => !url.includes('private-canary') && !url.includes('secret-123456'))).toBe(true);
  expect(requests.every(url => /\/assets\/[^?]+\.(js|wasm)$/.test(url))).toBe(true);
  await expect(page.getByRole('link', { name: 'Install the terminal app' })).toHaveAttribute('href', '/install/');
});
test('paste YAML, HAR and scalar and empty documents', async ({ page }) => {
  await ready(page);
  await page.getByRole('button', { name: 'Paste data', exact: true }).click();
  await page.getByLabel('Paste your data', { exact: true }).fill('kind: Deployment\n---\nkind: Service\n');
  await page.getByLabel('Pasted data format').selectOption('yaml');
  await page.getByRole('button', { name: 'Explore data' }).click();
  await expect(page.getByRole('status')).toContainText('Ready.');
  await page.getByLabel('Jump to path').fill('.[1].kind');
  await page.getByRole('button', { name: 'Go to path' }).click();
  await expect(page.getByLabel('Value preview')).toHaveText('"Service"');
  await page.getByRole('button', { name: 'HAR example' }).click();
  await expect(page.getByRole('status')).toContainText('Example loaded');
  await expect(page.getByLabel('Value preview')).toContainText('statusText');
  await open(page, 'false'); await expect(page.getByLabel('Value preview')).toHaveText('false');
  await open(page, '{}'); await expect(page.getByLabel('Value preview')).toHaveText('{}');
  await open(page, '[]'); await expect(page.getByLabel('Value preview')).toHaveText('[]');
});
test('invalid files, size limits, node limits and cancellation recover', async ({ page }) => {
  await ready(page);
  const input = page.getByLabel('Choose JSON, YAML or HAR file');
  await input.setInputFiles({ name: 'bad.json', mimeType: 'application/json', buffer: Buffer.from('{"a":1,"a":2}') });
  await expect(page.getByRole('alert')).toContainText('Duplicate');
  await input.setInputFiles({ name: 'large.json', mimeType: 'application/json', buffer: Buffer.alloc(20 * 1024 * 1024 + 1, 32) });
  await expect(page.getByRole('alert')).toContainText('exceeds 20 MiB');
  await input.setInputFiles({ name: 'dense.json', mimeType: 'application/json', buffer: Buffer.from('[' + '0,'.repeat(250_000) + '0]') });
  await expect(page.getByRole('alert')).toContainText('resource limit');
  let release!: () => void;
  const gate = new Promise<void>(resolve => { release = resolve; });
  await page.route('**/*.wasm', async route => { await gate; await route.abort().catch(() => {}); });
  await input.setInputFiles({ name: 'cancel.json', mimeType: 'application/json', buffer: Buffer.from('{"ok":true}') });
  await page.getByRole('button', { name: 'Cancel', exact: true }).click();
  await expect(page.getByRole('status')).toContainText('cancelled');
  release();
  await page.unrouteAll({ behavior: 'wait' });
  await open(page, '{"recovered":true}');
  await expect(page.getByLabel('Value preview')).toHaveText('true');
  await page.getByRole('button', { name: 'Close', exact: true }).click();
  await expect(page.getByText('No file open')).toBeVisible();
});
test('pagination and narrow layout keep selected data accessible', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await ready(page);
  await open(page, JSON.stringify(Array.from({ length: 600 }, (_, i) => ({ index: i }))));
  await page.getByRole('button', { name: 'Next page', exact: true }).click();
  await expect(page.getByLabel('Value preview')).toContainText('256');
  await page.getByRole('region', { name: 'Data explorer' }).focus();
  await page.keyboard.press('End');
  await expect(page.getByLabel('Value preview')).toContainText('599');
  await page.keyboard.press('ArrowRight');
  await expect(page.getByRole('listbox', { name: '599 entries' })).toBeVisible();
  await expect(page.getByRole('listbox', { name: 'root entries' })).not.toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
});
test('clipboard exports full precision and reports denied permission', async ({ page }) => {
  await page.addInitScript(() => {
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { write: async (items: ClipboardItem[]) => { (window as unknown as { copied: string }).copied = await (await items[0].getType('text/plain')).text(); } } });
  });
  await ready(page); await open(page, '{"n":18446744073709551615}');
  await page.getByRole('button', { name: 'Copy value', exact: true }).click();
  await expect(page.getByRole('status')).toContainText('Complete selected value copied');
  expect(await page.evaluate(() => (window as unknown as { copied: string }).copied)).toBe('18446744073709551615');
  await page.evaluate(() => Object.defineProperty(navigator, 'clipboard', { value: { write: async () => { throw new Error('Clipboard permission denied'); } } }));
  await page.getByRole('button', { name: 'Copy path', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('Clipboard permission denied');
});
test('near-20 MiB structured files load in the actual browser worker', async ({ page }) => {
  test.setTimeout(90_000);
  await ready(page);
  const row = { id: 1, text: 'x'.repeat(2048), active: true };
  const text = JSON.stringify(Array.from({ length: 9500 }, () => row));
  await open(page, text);
  await expect(page.getByLabel('Value preview')).toContainText('active');
  await page.getByLabel('Jump to path').fill('.[9499].id');
  await page.getByRole('button', { name: 'Go to path' }).click();
  await expect(page.getByLabel('Value preview')).toHaveText('1');
});
test('installation and guide routes remain usable', async ({ page }) => {
  await page.goto('/install/');
  await page.getByRole('button', { name: 'Windows', exact: true }).click();
  await expect(page.locator('#install-command')).toContainText('install.ps1');
  await page.getByRole('link', { name: 'Full installation guide' }).click();
  await expect(page.locator('h2#installation')).toBeVisible();
});
test('desktop and mobile visual review', async ({ page }, testInfo) => {
  test.skip(testInfo.project.name !== 'chromium', 'Visual artifacts captured once; functional layout coverage runs on all engines.');
  await page.setViewportSize({ width: 1440, height: 1100 });
  await ready(page);
  await page.screenshot({ path: testInfo.outputPath('desktop.png'), fullPage: true });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.screenshot({ path: testInfo.outputPath('mobile.png'), fullPage: true });
});
test('drop, replacement during file reading, and reload keep sessions isolated', async ({ page }) => {
  await ready(page);
  await page.evaluate(() => {
    const transfer = new DataTransfer();
    transfer.items.add(new File(['{"dropped":123}'], 'dropped.json', { type: 'application/json' }));
    document.querySelector('.app')!.dispatchEvent(new DragEvent('drop', { dataTransfer: transfer, bubbles: true, cancelable: true }));
  });
  await expect(page.getByRole('status')).toContainText('Ready.');
  await expect(page.getByLabel('Value preview')).toHaveText('123');
  await page.evaluate(() => {
    const original = File.prototype.arrayBuffer;
    File.prototype.arrayBuffer = function () {
      if (this.name === 'slow.json') return new Promise(resolve => {
        (window as unknown as { releaseRead: () => void }).releaseRead = () => resolve(new TextEncoder().encode('{"old":false}').buffer);
      });
      return original.call(this);
    };
  });
  await page.getByLabel('Choose JSON, YAML or HAR file').setInputFiles({ name: 'slow.json', mimeType: 'application/json', buffer: Buffer.from('{}') });
  await open(page, '{"replacement":true}');
  await page.evaluate(() => (window as unknown as { releaseRead: () => void }).releaseRead());
  await expect(page.getByLabel('Value preview')).toHaveText('true');
  expect(await page.evaluate(() => [localStorage.length, sessionStorage.length])).toEqual([0, 0]);
  await page.reload();
  await expect(page.getByRole('status')).toContainText('Example loaded');
  await expect(page.getByRole('option').filter({ hasText: 'replacement' })).toHaveCount(0);
});
test('full-page workspace shares themes and terminal keyboard flow', async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await ready(page);
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
  expect(await page.evaluate(() => getComputedStyle(document.body).backgroundColor)).toBe('rgb(21, 23, 27)');
  expect(await page.evaluate(() => ({ page: document.documentElement.scrollHeight, viewport: innerHeight }))).toEqual({ page: 800, viewport: 800 });
  expect(await page.locator('.explorer-body').evaluate(el => el.getBoundingClientRect().height)).toBeGreaterThan(500);
  await page.keyboard.press('t');
  await expect(page.getByRole('button', { name: 'Switch to dark theme' })).toBeVisible();
  expect(await page.evaluate(() => getComputedStyle(document.body).backgroundColor)).toBe('rgb(250, 251, 252)');
  await page.reload(); await expect(page.getByRole('status')).toContainText('Example loaded');
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
  await page.keyboard.press('/');
  await expect(page.getByLabel('Search keys and values')).toBeFocused();
  await page.keyboard.type('Ada'); await page.keyboard.press('Enter');
  await expect(page.getByLabel('Value preview')).toHaveText('"Ada"');
  await expect(page.getByRole('region', { name: 'Data explorer' })).toBeFocused();
  await page.keyboard.press('j'); await expect(page.getByLabel('Value preview')).toHaveText('"Engineering"');
  await page.keyboard.press(':'); await page.keyboard.type('.environments.production.replicas'); await page.keyboard.press('Enter');
  await expect(page.getByLabel('Value preview')).toHaveText('3');
  await page.keyboard.press('/'); await page.keyboard.press('Escape');
  await expect(page.getByRole('region', { name: 'Data explorer' })).toBeFocused();
  await expect(page.getByLabel('Value preview')).toHaveText('3');
  await page.keyboard.press('?'); await expect(page.getByRole('dialog', { name: 'Keyboard shortcuts' })).toBeVisible();
  await page.keyboard.press('Escape'); await expect(page.getByRole('dialog')).toHaveCount(0);
  await page.keyboard.press('t'); await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
  await page.keyboard.press('q'); await expect(page.getByText('No file open')).toBeVisible();
});
