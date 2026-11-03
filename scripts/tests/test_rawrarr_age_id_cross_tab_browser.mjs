import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const root = new URL('../../', import.meta.url);
const [html, script, deviceRoutes, nodeAvatar, profileColor, aiModule, availabilityModule] = await Promise.all([
  readFile(new URL('rust-arr/src/coordinator_workspace.html', root), 'utf8'),
  readFile(new URL('rust-arr/src/coordinator-workspace.mjs', root), 'utf8'),
  readFile(new URL('rust-arr/src/coordinator-device-routes.mjs', root), 'utf8'),
  readFile(new URL('rust-arr/src/coordinator-node-avatar.mjs', root), 'utf8'),
  readFile(new URL('rust-arr/src/profile-color.mjs', root), 'utf8'),
  readFile(new URL('rust-arr/src/coordinator-ai.mjs', root), 'utf8'),
  readFile(new URL('rust-arr/src/coordinator-availability.mjs', root), 'utf8'),
]);

test('Movies picks up an age ID imported in another tab', async t => {
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const context = await browser.newContext();
  t.after(() => context.close());
  const page = await context.newPage();
  const identity = 'AGE-SECRET-KEY-PQ-1cross-tab-test-fixture';
  let unlocked = false;
  let loadedMovies = false;

  await context.route('http://rawrarr.test/**', async route => {
    const request = route.request();
    const url = new URL(request.url());
    if (url.pathname === '/health') return route.fulfill({ status: 200, body: 'ok' });
    if (url.pathname === '/') return route.fulfill({ body: html, contentType: 'text/html' });
    if (url.pathname === '/assets/coordinator-workspace.mjs') return route.fulfill({ body: script, contentType: 'text/javascript' });
    if (url.pathname === '/assets/coordinator-availability.mjs') return route.fulfill({ body: availabilityModule, contentType: 'text/javascript' });
    if (url.pathname === '/assets/coordinator-device-routes.mjs') return route.fulfill({ body: deviceRoutes, contentType: 'text/javascript' });
    if (url.pathname === '/assets/coordinator-node-avatar.mjs') return route.fulfill({ body: nodeAvatar, contentType: 'text/javascript' });
    if (url.pathname === '/assets/profile-color.mjs') return route.fulfill({ body: profileColor, contentType: 'text/javascript' });
    if (url.pathname === '/assets/coordinator-ai.mjs') return route.fulfill({ body: aiModule, contentType: 'text/javascript' });
    if (url.pathname === '/assets/metadata-export.js') return route.fulfill({ contentType: 'text/javascript', body: "window.rawrarrMiniExport = { deriveProfileRecipient: async () => 'age1pq1testrecipient', prepareGroupIdentityForDevice: async () => false, encryptIdentityForDevice: async value => `age-ciphertext:${value}` };" });
    if (url.pathname === '/api/workspace/profile') return route.fulfill({ json: { displayName: 'Test user' } });
    if (url.pathname === '/api/workspace/nodes') return route.fulfill({ json: [{ nodeId: 'metadata-one', role: 'metadata', online: true, relayAvailable: true, directUrl: 'http://rawrarr.test/direct-api/' }] });
    if (url.pathname === '/api/workspace/pending') return route.fulfill({ json: [] });
    if (url.pathname === '/api/workspace/direct-session') return route.fulfill({ status: 204, headers: { 'set-cookie': `rawrarr_direct_session=${'a'.repeat(64)}; Path=/direct-api/; SameSite=Strict` } });
    if (url.pathname === '/direct-api/health') return route.fulfill({ status: 204 });
    if (url.pathname === '/direct-api/api/settings/age' && request.method() === 'POST') {
      unlocked = true;
      return route.fulfill({ json: { unlocked: true, encryptedConfigExists: true } });
    }
    if (url.pathname === '/direct-api/api/settings/age' && request.method() === 'GET') return route.fulfill({ json: { unlocked, encryptedConfigExists: true } });
    if (url.pathname === '/direct-api/api/radarr/live-library') {
      loadedMovies = true;
      return route.fulfill({ json: [] });
    }
    if (url.pathname.startsWith('/direct-api/api/')) return route.fulfill({ json: {} });
    if (url.pathname.startsWith('/api/workspace/')) return route.fulfill({ json: {} });
    return route.fulfill({ status: 404, body: 'not found' });
  });

  await page.goto('http://rawrarr.test/#movies');
  await page.waitForFunction(() => document.querySelector('[data-node-id="metadata-one"]')?.dataset.selected === 'true');
  assert.equal(await page.locator('#age-id-entry').isVisible(), true);

  // A real storage event is dispatched by the browser to other same-origin tabs.
  await page.evaluate(value => {
    localStorage.setItem('rawrarr.browser.age-id', value);
    window.dispatchEvent(new StorageEvent('storage', {
      key: 'rawrarr.browser.age-id',
      oldValue: null,
      newValue: value,
      storageArea: localStorage,
      url: location.href,
    }));
  }, identity);

  await page.waitForFunction(() => document.getElementById('age-id-entry').hidden);
  await page.waitForFunction(() => document.getElementById('content-status').textContent.includes('Radarr library entries loaded'));
  assert.equal(loadedMovies, true, 'the age identity imported in Settings unlocks the Movies direct route');
  assert.equal(await page.locator('#age-id').inputValue(), '');
  assert.equal((await page.locator('body').innerText()).includes(identity), false, 'the private ID is never rendered');

  for (const viewport of [{ width: 1280, height: 800 }, { width: 375, height: 812 }]) {
    await page.setViewportSize(viewport);
    const geometry = await page.evaluate(() => ({ width: innerWidth, documentWidth: document.documentElement.scrollWidth, clear: document.querySelector('#clear-key').getBoundingClientRect().toJSON() }));
    assert.ok(geometry.documentWidth <= geometry.width, `Movies overflows at ${viewport.width}px`);
    assert.ok(geometry.clear.width > 0 && geometry.clear.left >= 0 && geometry.clear.right <= viewport.width, `clear button fits at ${viewport.width}px`);
  }

  await page.evaluate(() => {
    localStorage.removeItem('rawrarr.browser.age-id');
    window.dispatchEvent(new StorageEvent('storage', {
      key: 'rawrarr.browser.age-id',
      oldValue: 'AGE-SECRET-KEY-PQ-1cross-tab-test-fixture',
      newValue: null,
      storageArea: localStorage,
      url: location.href,
    }));
  });
  await page.waitForFunction(() => !document.getElementById('age-id-entry').hidden);
  assert.equal(await page.locator('#movies .library-empty').count(), 0);
  assert.match(await page.locator('#content-status').textContent(), /metadata is locked/);
});
