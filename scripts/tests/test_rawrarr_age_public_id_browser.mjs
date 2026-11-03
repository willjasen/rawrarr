import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const root = new URL('../../', import.meta.url);
const html = await readFile(new URL('rust-arr/src/coordinator_workspace.html', root), 'utf8');
const modules = [
  'coordinator-workspace.mjs', 'coordinator-availability.mjs', 'coordinator-device-routes.mjs',
  'coordinator-node-avatar.mjs', 'profile-color.mjs', 'coordinator-ai.mjs',
];
const assets = new Map(await Promise.all(modules.map(async name => [
  `/assets/${name}`, await readFile(new URL(`rust-arr/src/${name}`, root), 'utf8'),
])));
assets.set('/assets/metadata-export.js', await readFile(new URL('rust-arr/src/assets/metadata-export.js', root), 'utf8'));
const generated = execFileSync('age-keygen', ['-pq'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
const identity = generated.split('\n').find(line => line.startsWith('AGE-SECRET-KEY-PQ-1'));
assert.ok(identity);
const recipient = execFileSync('age-keygen', ['-y'], { input: identity, encoding: 'utf8' }).trim();

for (const viewport of [{ width: 1280, height: 800 }, { width: 375, height: 812 }]) {
  test(`Settings shows and copies a real age public ID at ${viewport.width}px`, async t => {
    const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
    t.after(() => browser.close());
    const context = await browser.newContext({ viewport, permissions: ['clipboard-read', 'clipboard-write'] });
    const page = await context.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    await context.route('http://localhost/**', async route => {
      const request = route.request();
      assert.equal((request.postData() || '').includes(identity), false, 'private ID stays in the browser');
      const { pathname } = new URL(request.url());
      if (pathname === '/') return route.fulfill({ contentType: 'text/html', body: html });
      if (assets.has(pathname)) return route.fulfill({ contentType: 'text/javascript', body: assets.get(pathname) });
      if (pathname === '/health') return route.fulfill({ status: 204 });
      if (pathname === '/api/workspace/profile') return route.fulfill({ json: { displayName: 'Test user' } });
      if (pathname === '/api/workspace/nodes' || pathname === '/api/workspace/pending') return route.fulfill({ json: [] });
      if (pathname.startsWith('/api/workspace/')) return route.fulfill({ json: {} });
      return route.fulfill({ status: 404 });
    });
    await page.addInitScript(value => localStorage.setItem('rawrarr.browser.age-id', value), identity);
    await page.goto('http://localhost/#settings');
    const show = page.locator('#show-age-recipient');
    await show.click();
    await page.locator('#age-recipient-display').waitFor({ state: 'visible', timeout: 5000 });
    assert.equal(await show.textContent(), 'Hide age public ID');
    assert.equal(await page.locator('#age-recipient-short').textContent(), `${recipient.slice(0, 12)}…${recipient.slice(-8)}`);
    await page.locator('#copy-age-recipient').click();
    await page.waitForFunction(() => document.getElementById('age-recipient-message').textContent === 'Age public ID copied.');
    assert.equal(await page.evaluate(() => navigator.clipboard.readText()), recipient);
    assert.equal((await page.locator('body').innerText()).includes(identity), false);
    const geometry = await page.evaluate(() => ({
      width: innerWidth, documentWidth: document.documentElement.scrollWidth,
      buttons: ['show-age-recipient', 'copy-age-recipient', 'remove-age-id'].map(id => document.getElementById(id).getBoundingClientRect().toJSON()),
    }));
    assert.ok(geometry.documentWidth <= geometry.width, `no horizontal overflow at ${viewport.width}px`);
    for (const bounds of geometry.buttons) {
      assert.ok(bounds.width > 0 && bounds.left >= 0 && bounds.right <= viewport.width, 'age actions fit the viewport');
    }
    await show.click();
    assert.equal(await page.locator('#age-recipient-display').isVisible(), false);
    assert.equal(await page.locator('#age-recipient-short').textContent(), '');
    await show.click();
    await page.locator('#age-recipient-display').waitFor({ state: 'visible' });
    await page.locator('#remove-age-id').click();
    assert.equal(await show.isVisible(), false);
    assert.equal(await page.locator('#age-recipient-display').isVisible(), false);
    assert.equal(await page.evaluate(() => localStorage.getItem('rawrarr.browser.age-id')), null);
    assert.deepEqual(errors, []);
  });
}
