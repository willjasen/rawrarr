import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const root = new URL('../../rust-arr/src/', import.meta.url);
const html = await readFile(new URL('coordinator_workspace.html', root), 'utf8');
const modules = ['coordinator-workspace', 'coordinator-availability', 'coordinator-device-routes', 'coordinator-node-avatar', 'profile-color', 'coordinator-ai'];
const assets = Object.fromEntries(await Promise.all(modules.map(async name => [
  `/assets/${name}.mjs`, await readFile(new URL(`${name}.mjs`, root), 'utf8'),
])));

async function persistenceFixture(t, viewport) {
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const context = await browser.newContext({ viewport });
  t.after(() => context.close());
  const identity = 'AGE-SECRET-KEY-PQ-1persistence-test-fixture';
  const logoutCopies = [];
  const state = {
    nodes: [{ nodeId: 'metadata-one', role: 'metadata', online: true, relayAvailable: true, directUrl: 'http://rawrarr.test/direct-api/' }],
    delayLibrary: '',
    releaseLibrary: null,
  };
  await context.route('http://rawrarr.test/**', async route => {
    const url = new URL(route.request().url());
    if (assets[url.pathname]) return route.fulfill({ contentType: 'text/javascript', body: assets[url.pathname] });
    if (url.pathname === '/assets/metadata-export.js') return route.fulfill({
      contentType: 'text/javascript',
      body: `window.rawrarrMiniExport = {
        deriveProfileRecipient: async () => {
          if (window.delayAgeValidation) await new Promise(resolve => { window.releaseAgeValidation = resolve; });
          return 'age1pq1testrecipient';
        },
        runMiniExport: async () => {
          await new Promise(resolve => { window.releaseAgeExport = resolve; });
          return 1;
        }
      };`,
    });
    if (url.pathname === '/') return route.fulfill({ contentType: 'text/html', body: html });
    if (url.pathname === '/auth') return route.fulfill({ contentType: 'text/html', body: '<h1>Signed out</h1>' });
    if (url.pathname === '/api/auth/logout') {
      logoutCopies.push(await route.request().frame().evaluate(() => localStorage.getItem('rawrarr.browser.age-id')));
      return route.fulfill({ json: {} });
    }
    if (url.pathname === '/health' || url.pathname === '/direct-api/health') return route.fulfill({ status: 204 });
    if (url.pathname === '/api/workspace/profile') return route.fulfill({ json: { displayName: 'Test user' } });
    if (url.pathname === '/api/workspace/nodes') return route.fulfill({ json: state.nodes });
    if (url.pathname === '/api/workspace/pending') return route.fulfill({ json: [] });
    if (url.pathname === '/api/workspace/direct-session') return route.fulfill({ status: 204, headers: { 'set-cookie': `rawrarr_direct_session=${'a'.repeat(64)}; Path=/direct-api/; SameSite=Strict` } });
    if (url.pathname === '/direct-api/api/settings/age') return route.fulfill({ json: { unlocked: true } });
    if (url.pathname === '/direct-api/api/settings/sonarr') return route.fulfill({ json: { configured: true, url: 'http://sonarr.test' } });
    if (url.pathname === '/direct-api/api/radarr/live-library' || url.pathname === '/direct-api/api/sonarr/series') {
      if (url.pathname.endsWith(state.delayLibrary) && state.delayLibrary) {
        await new Promise(resolve => { state.releaseLibrary = resolve; });
      }
      return route.fulfill({ json: [{ tmdbId: 123, tvdbId: 456, title: 'Delayed library result', genres: [] }] });
    }
    if (url.pathname.startsWith('/direct-api/api/')) return route.fulfill({ json: {} });
    if (url.pathname.startsWith('/api/workspace/')) return route.fulfill({ json: {} });
    return route.fulfill({ status: 404 });
  });
  const page = await context.newPage();
  await page.goto('http://rawrarr.test/#settings');
  async function save(target = page) {
    await target.locator('#settings-age-id').fill(identity);
    await target.locator('#age-storage-form button[type="submit"]').click();
    await target.waitForFunction(() => document.getElementById('settings-age-id').hidden);
  }
  return { context, page, identity, logoutCopies, save, state };
}

for (const viewport of [{ width: 1280, height: 800 }, { width: 375, height: 812 }]) {
  test(`age ID persists until clear or sign-out at ${viewport.width}px`, async t => {
    const { context, page, identity, logoutCopies, save } = await persistenceFixture(t, viewport);
    await save();
    await page.reload();
    await page.waitForFunction(() => document.getElementById('settings-age-id').hidden);
    assert.equal(await page.evaluate(() => localStorage.getItem('rawrarr.browser.age-id')), identity);
    assert.match(await page.locator('#age-storage-status').textContent(), /until you clear it or sign out/);
    const other = await context.newPage();
    await other.goto('http://rawrarr.test/#settings');
    await other.waitForFunction(() => document.getElementById('settings-age-id').hidden);
    for (const selector of ['#remove-age-id', '#sign-out']) {
      const rect = await other.locator(selector).boundingBox();
      assert.ok(rect && rect.width > 0 && rect.x >= 0 && rect.x + rect.width <= viewport.width, `${selector} fits`);
    }
    assert.equal(await other.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true, 'privacy copy and controls do not overflow');
    assert.equal((await other.locator('body').innerText()).includes(identity), false);

    await page.goto('http://rawrarr.test/#movies');
    await page.locator('#clear-key').click();
    await other.waitForFunction(() => !document.getElementById('settings-age-id').hidden);
    assert.equal(await page.evaluate(() => localStorage.getItem('rawrarr.browser.age-id')), null);
    await page.reload();
    await page.locator('#age-id-entry').waitFor({ state: 'visible' });

    await save(other);
    await other.locator('#remove-age-id').click();
    assert.equal(await other.evaluate(() => localStorage.getItem('rawrarr.browser.age-id')), null);
    await other.reload();
    await other.locator('#settings-age-id').waitFor({ state: 'visible' });

    await save(other);
    await page.locator('#clear-key').waitFor({ state: 'visible' });
    await other.locator('#sign-out').click();
    await other.waitForURL('**/auth');
    assert.deepEqual(logoutCopies, [null], 'browser ID is removed before logout request');
    await page.locator('#age-id-entry').waitFor({ state: 'visible' });
    await page.reload();
    assert.equal(await page.evaluate(() => localStorage.getItem('rawrarr.browser.age-id')), null);

    await page.goto('http://rawrarr.test/#settings');
    await save();
    await other.goto('http://rawrarr.test/#settings');
    await other.waitForFunction(() => document.getElementById('settings-age-id').hidden);
    await other.evaluate(() => localStorage.clear());
    await page.locator('#settings-age-id').waitFor({ state: 'visible' });
    assert.equal(await page.locator('#remove-age-id').isVisible(), false, 'whole-site storage clearing also clears the other tab');
  });
}

test('storage failures do not claim persistence or successful clear/sign-out', async t => {
  const { page, identity, logoutCopies, save } = await persistenceFixture(t, { width: 375, height: 812 });
  await page.evaluate(() => {
    const setItem = Storage.prototype.setItem;
    Storage.prototype.setItem = function(key, value) {
      if (key === 'rawrarr.browser.age-id') throw new DOMException('Storage blocked', 'QuotaExceededError');
      return setItem.call(this, key, value);
    };
  });
  await save();
  assert.match(await page.locator('#age-storage-status').textContent(), /available for this page.*could not save it/);
  assert.equal(await page.evaluate(() => localStorage.getItem('rawrarr.browser.age-id')), null);
  await page.reload();
  await page.locator('#settings-age-id').waitFor({ state: 'visible' });
  await save();
  await page.evaluate(() => {
    const removeItem = Storage.prototype.removeItem;
    Storage.prototype.removeItem = function(key) {
      if (key === 'rawrarr.browser.age-id') throw new DOMException('Storage blocked', 'SecurityError');
      return removeItem.call(this, key);
    };
  });
  await page.locator('#remove-age-id').click();
  assert.match(await page.locator('#age-storage-message').textContent(), /Could not remove the saved age ID/);
  assert.equal(await page.evaluate(() => localStorage.getItem('rawrarr.browser.age-id')), identity);
  assert.equal(await page.locator('#remove-age-id').isVisible(), true, 'clear remains available to retry');
  await page.locator('.nav-item[data-view="overview"]').click();
  await page.locator('#sign-out').click();
  assert.deepEqual(logoutCopies, []);
  assert.equal(new URL(page.url()).pathname, '/', 'do not navigate while browser cleanup has failed');
  assert.equal(new URL(page.url()).hash, '#settings', 'cleanup errors are visible even when signing out from another view');
  assert.equal(await page.locator('#age-storage-message').isVisible(), true);
});

for (const action of ['clear', 'sign-out']) {
  test(`pending validation cannot restore an age ID after ${action}`, async t => {
    const { page, save } = await persistenceFixture(t, { width: 1280, height: 800 });
    await save();
    await page.evaluate(() => {
      window.delayAgeValidation = true;
      document.getElementById('age-storage-form').requestSubmit();
    });
    await page.waitForFunction(() => window.releaseAgeValidation);
    if (action === 'clear') {
      await page.locator('#remove-age-id').click();
      await page.evaluate(() => window.releaseAgeValidation());
      await page.waitForFunction(() => !document.getElementById('save-age-id').disabled);
      assert.equal(await page.evaluate(() => localStorage.getItem('rawrarr.browser.age-id')), null);
    } else {
      await page.route('**/api/auth/logout', async route => {
        await page.evaluate(() => window.releaseAgeValidation());
        await page.waitForFunction(() => !document.getElementById('save-age-id').disabled);
        assert.equal(await page.evaluate(() => localStorage.getItem('rawrarr.browser.age-id')), null);
        await route.fulfill({ json: {} });
      });
      await page.locator('#sign-out').click();
      await page.waitForURL('**/auth');
    }
    await page.goto('http://rawrarr.test/#settings');
    await page.locator('#settings-age-id').waitFor({ state: 'visible' });
  });
}

for (const view of ['movies', 'series']) {
  test(`pending ${view} response cannot repopulate the library after clear`, async t => {
    const { page, save, state } = await persistenceFixture(t, { width: 1280, height: 800 });
    await save();
    const endpoint = view === 'movies' ? '/radarr/live-library' : '/sonarr/series';
    state.delayLibrary = endpoint;
    const requested = page.waitForRequest(request => request.url().endsWith(endpoint));
    await page.goto(`http://rawrarr.test/#${view}`);
    t.after(() => state.releaseLibrary?.());
    await requested;
    await page.locator('.nav-item[data-view="settings"]').click();
    await page.locator('#remove-age-id').click();
    const responded = page.waitForResponse(response => response.url().endsWith(endpoint));
    state.releaseLibrary();
    await (await responded).finished();
    await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
    assert.equal(await page.locator('#movies .library-card, #sonarr-series .library-card').count(), 0);
    assert.equal(await page.evaluate(() => localStorage.getItem('rawrarr.browser.age-id')), null);
  });
}

test('pending metadata export cannot save the ID again after clear', async t => {
  const { page, save } = await persistenceFixture(t, { width: 1280, height: 800 });
  await save();
  await page.reload();
  await page.waitForFunction(() => document.querySelector('[data-node-id="metadata-one"]')?.dataset.selected === 'true');
  await page.locator('#export-metadata').click();
  await page.waitForFunction(() => window.releaseAgeExport);
  await page.locator('#remove-age-id').click();
  await page.evaluate(() => window.releaseAgeExport());
  await page.waitForFunction(() => !document.getElementById('export-metadata').disabled);
  assert.equal(await page.evaluate(() => localStorage.getItem('rawrarr.browser.age-id')), null);
});
