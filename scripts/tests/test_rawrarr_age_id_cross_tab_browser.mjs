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
  let nodes = [{ nodeId: 'metadata-one', role: 'metadata', online: true, relayAvailable: true, directUrl: 'http://rawrarr.test/direct-api/' }];
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
    if (url.pathname === '/api/workspace/nodes') return route.fulfill({ json: nodes });
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

  nodes = [];
  const noNodePage = await context.newPage();
  await noNodePage.addInitScript(value => localStorage.setItem('rawrarr.browser.age-id', value), identity);
  await noNodePage.goto('http://rawrarr.test/#movies');
  await noNodePage.waitForFunction(() => document.getElementById('age-id-entry').hidden);
  assert.match(await noNodePage.locator('#content-status').textContent(), /Age private ID is saved in this browser/);
  assert.doesNotMatch(await noNodePage.locator('#content-status').textContent(), /Enter your age private ID/);
});

test('Settings-saved age ID is recognized when navigating to Movies in the same tab', async t => {
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const context = await browser.newContext();
  t.after(() => context.close());
  const page = await context.newPage();
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

  await page.goto('http://rawrarr.test/#settings');
  await page.locator('#settings-age-id').fill('AGE-SECRET-KEY-PQ-1same-tab-test-fixture');
  await page.locator('#age-storage-form button[type="submit"]').click();
  await page.waitForFunction(() => localStorage.getItem('rawrarr.browser.age-id')?.includes('same-tab-test-fixture'));
  await page.locator('.nav-item[data-view="movies"]').click();
  await page.waitForFunction(() => document.getElementById('age-id-entry').hidden);
  await page.waitForFunction(() => document.querySelector('[data-node-id="metadata-one"]')?.dataset.selected === 'true');
  await page.waitForFunction(() => document.getElementById('content-status').textContent.includes('Radarr library entries loaded'));
  assert.equal(loadedMovies, true);
  assert.equal((await page.locator('body').innerText()).includes('Enter your age private ID to unlock'), false);
});

for (const viewport of [{ width: 1280, height: 800 }, { width: 375, height: 812 }]) {
  test(`Movies loads with a saved age ID before diagnostics at ${viewport.width}px`, { timeout: 20000 }, async t => {
    const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
    t.after(() => browser.close());
    const context = await browser.newContext({ viewport });
    t.after(() => context.close());
    const page = await context.newPage();
    const identity = 'AGE-SECRET-KEY-PQ-1initial-load-test-fixture';
    const movie = { tmdbId: 123, title: 'A deliberately long movie title that wraps on a narrow screen', year: 2026, hasFile: true, sizeOnDisk: 1024, genres: [] };
    let releaseDiagnostics;
    const diagnosticsReady = new Promise(resolve => { releaseDiagnostics = resolve; });
    t.after(() => releaseDiagnostics());
    let diagnosticResponses = 0;
    let libraryRequests = 0;
    let pendingStatus = 200;
    const pageErrors = [];
    page.on('pageerror', error => pageErrors.push(error.message));
    const assets = {
      '/assets/coordinator-workspace.mjs': `${script}\nwindow.rawrarrTestWorkspaceReady = true; window.rawrarrTestLoadPending = loadPending;`,
      '/assets/coordinator-availability.mjs': availabilityModule,
      '/assets/coordinator-device-routes.mjs': deviceRoutes,
      '/assets/coordinator-node-avatar.mjs': nodeAvatar,
      '/assets/profile-color.mjs': profileColor,
      '/assets/coordinator-ai.mjs': aiModule,
      '/assets/metadata-export.js': "window.rawrarrMiniExport = { deriveProfileRecipient: async () => 'age1pq1testrecipient', loadDeviceNames: async () => ({}) };",
    };
    await context.route('http://localhost/**', async route => {
      const request = route.request();
      const url = new URL(request.url());
      if (assets[url.pathname]) return route.fulfill({ contentType: 'text/javascript', body: assets[url.pathname] });
      if (url.pathname === '/') return route.fulfill({ contentType: 'text/html', body: html });
      if (url.pathname === '/health' || url.pathname === '/direct-api/health') return route.fulfill({ status: 204 });
      if (url.pathname === '/api/workspace/profile') return route.fulfill({ json: { displayName: 'Test user' } });
      if (url.pathname === '/api/workspace/nodes') return route.fulfill({ json: [{ nodeId: 'metadata-one', role: 'metadata', online: true, relayAvailable: true, directUrl: 'http://localhost/direct-api/' }] });
      if (url.pathname === '/api/workspace/pending') return route.fulfill({ status: pendingStatus, json: [] });
      if (url.pathname === '/api/workspace/direct-session') return route.fulfill({ status: 204, headers: { 'set-cookie': `rawrarr_direct_session=${'a'.repeat(64)}; Path=/direct-api/; SameSite=Strict` } });
      if (url.pathname.endsWith('/syncthing/shared-health') || url.pathname.endsWith('/config-peer/status')) {
        await diagnosticsReady;
        diagnosticResponses += 1;
        return route.fulfill({ status: 401, json: { error: 'Node diagnostic session rejected' } });
      }
      if (url.pathname === '/direct-api/api/settings/age') return route.fulfill({ json: { unlocked: true, encryptedConfigExists: true } });
      if (url.pathname === '/direct-api/api/radarr/live-library') {
        libraryRequests += 1;
        return route.fulfill({ json: [movie] });
      }
      if (url.pathname.startsWith('/nodes/')) return route.fulfill({ status: 401, json: { error: 'Node request rejected' } });
      if (url.pathname.startsWith('/direct-api/api/')) return route.fulfill({ json: {} });
      if (url.pathname.startsWith('/api/workspace/')) return route.fulfill({ json: {} });
      return route.fulfill({ status: 404 });
    });
    await page.addInitScript(({ identity, restoreSelection }) => {
      localStorage.setItem('rawrarr.browser.age-id', identity);
      if (restoreSelection) sessionStorage.setItem('rawrarr.coordinator.selected-node', 'metadata-one');
    }, { identity, restoreSelection: viewport.width === 1280 });

    await page.goto('http://localhost/#movies', { waitUntil: 'commit' });
    await page.waitForFunction(() => document.querySelector('#movies .library-card'), null, { timeout: 2500 });
    assert.equal(diagnosticResponses, 0, 'library loads while optional diagnostics are still pending');
    assert.equal(await page.locator('#age-id-entry').isVisible(), false);
    assert.equal(await page.locator('#movies .library-card-title').textContent(), movie.title);
    releaseDiagnostics();
    await page.waitForFunction(() => window.rawrarrTestWorkspaceReady);
    assert.equal(diagnosticResponses, 2);
    assert.equal(await page.evaluate(() => localStorage.getItem('rawrarr.browser.age-id')), identity, 'node 401 responses must not erase the imported identity');
    assert.equal(await page.locator('#age-id-entry').isVisible(), false);

    const geometry = await page.evaluate(() => ({
      width: innerWidth, documentWidth: document.documentElement.scrollWidth,
      card: document.querySelector('#movies .library-card').getBoundingClientRect().toJSON(),
      clear: document.querySelector('#clear-key').getBoundingClientRect().toJSON(),
      rescan: document.querySelector('#refresh-movies').getBoundingClientRect().toJSON(),
    }));
    assert.ok(geometry.documentWidth <= geometry.width, 'Movies must not overflow horizontally');
    for (const rect of [geometry.card, geometry.clear, geometry.rescan]) {
      assert.ok(rect.width > 0 && rect.left >= 0 && rect.right <= viewport.width, 'movie and primary actions fit the viewport');
    }

    // Hash navigation and back/forward-cache restoration must start the same load flow.
    await page.locator('.nav-item[data-view="settings"]').click();
    const beforeHashNavigation = libraryRequests;
    await page.evaluate(() => { location.hash = '#movies'; });
    await page.waitForFunction(() => document.querySelector('#content-status').textContent.includes('Radarr library entries loaded'));
    assert.ok(libraryRequests > beforeHashNavigation);
    const beforeRestore = libraryRequests;
    await page.evaluate(() => {
      window.dispatchEvent(new PageTransitionEvent('pagehide', { persisted: true }));
      window.dispatchEvent(new PageTransitionEvent('pageshow', { persisted: true }));
    });
    await page.waitForFunction(() => document.querySelector('#content-status').textContent.includes('Radarr library entries loaded'));
    assert.ok(libraryRequests > beforeRestore);
    assert.equal(await page.locator('#age-id-entry').isVisible(), false);

    pendingStatus = 401;
    await page.evaluate(() => window.rawrarrTestLoadPending());
    assert.equal(await page.evaluate(() => localStorage.getItem('rawrarr.browser.age-id')), identity, 'an expired coordinator session is not an explicit sign-out');
    assert.equal((await page.locator('body').innerText()).includes(identity), false);
    await page.evaluate(() => {
      window.dispatchEvent(new PageTransitionEvent('pagehide', { persisted: true }));
      localStorage.removeItem('rawrarr.browser.age-id');
      window.dispatchEvent(new PageTransitionEvent('pageshow', { persisted: true }));
    });
    assert.equal(await page.locator('#age-id-entry').isVisible(), true);
    assert.equal(await page.locator('#movies .library-card').count(), 0, 'a page restored after ID removal must not show the old library');
    assert.deepEqual(pageErrors, []);
  });
}
