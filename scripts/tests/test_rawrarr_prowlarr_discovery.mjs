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

test('Prowlarr settings stay on the selected node and discovery is read-only and responsive', async t => {
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  const identity = 'AGE-SECRET-KEY-PQ-1browser-only-test-fixture';
  const apiKey = 'prowlarr-secret-api-key';
  const workspaceRequests = [];
  const coordinatorPaths = [];
  const directRequests = [];
  let prowlarr = null;
  let failSearch = false;
  const searchResults = [{
    id: 'stable-result-id', title: '<img src=x onerror=alert(1)>', indexerId: 7,
    indexerName: 'Fixture indexer', protocol: 'torrent', sizeBytes: 1234,
    publishedAt: '2026-11-03T00:00:00Z', categories: ['Movies'],
  }];

  await page.addInitScript(value => localStorage.setItem('rawrarr.browser.age-id', value), identity);
  const pageErrors = [];
  page.on('pageerror', error => pageErrors.push(error.message));
  await page.route('http://localhost:3000/**', async route => {
    const request = route.request();
    const url = new URL(request.url());
    if (!url.pathname.startsWith('/direct-api/')) coordinatorPaths.push(url.pathname);
    if (url.pathname === '/health') return route.fulfill({ status: 200, body: 'ok' });
    if (url.pathname === '/') return route.fulfill({ body: html, contentType: 'text/html' });
    if (url.pathname === '/assets/coordinator-workspace.mjs') return route.fulfill({ body: script, contentType: 'text/javascript' });
    if (url.pathname === '/assets/coordinator-availability.mjs') return route.fulfill({ body: availabilityModule, contentType: 'text/javascript' });
    if (url.pathname === '/assets/coordinator-device-routes.mjs') return route.fulfill({ body: deviceRoutes, contentType: 'text/javascript' });
    if (url.pathname === '/assets/coordinator-node-avatar.mjs') return route.fulfill({ body: nodeAvatar, contentType: 'text/javascript' });
    if (url.pathname === '/assets/profile-color.mjs') return route.fulfill({ body: profileColor, contentType: 'text/javascript' });
    if (url.pathname === '/assets/coordinator-ai.mjs') return route.fulfill({ body: aiModule, contentType: 'text/javascript' });
    if (url.pathname === '/assets/metadata-export.js') return route.fulfill({
      contentType: 'text/javascript',
      body: "window.rawrarrMiniExport = { prepareGroupIdentityForDevice: async () => false, encryptIdentityForDevice: async identity => `age-ciphertext:${identity}` };",
    });
    if (url.pathname === '/api/workspace/profile') return route.fulfill({ json: { displayName: 'Test user' } });
    if (url.pathname === '/api/workspace/nodes') return route.fulfill({ json: [{ nodeId: 'metadata-one', role: 'metadata', online: true, relayAvailable: true, directUrl: 'http://localhost:3000/direct-api/' }] });
    if (url.pathname === '/api/workspace/pending') return route.fulfill({ json: [] });
    if (url.pathname === '/api/workspace/direct-session') {
      workspaceRequests.push({ path: url.pathname, body: request.postData() || '' });
      return route.fulfill({ status: 204, headers: { 'set-cookie': `rawrarr_direct_session=${'a'.repeat(64)}; Path=/direct-api/; SameSite=Strict` } });
    }
    if (url.pathname === '/direct-api/health') return route.fulfill({ status: 204 });
    if (url.pathname.startsWith('/direct-api/api/')) {
      directRequests.push({ path: url.pathname, method: request.method(), body: request.postData() || '' });
      if (url.pathname === '/direct-api/api/settings/age' && request.method() === 'GET') {
        return route.fulfill({ json: { unlocked: true, encryptedConfigExists: true } });
      }
      if (url.pathname.endsWith('/settings/radarr') || url.pathname.endsWith('/settings/sonarr')) {
        return route.fulfill({ json: { configured: false, persisted: false, url: null, hasUiAuth: false } });
      }
      if (url.pathname === '/direct-api/api/settings/prowlarr' && request.method() === 'GET') {
        return route.fulfill({ json: { configured: Boolean(prowlarr), persisted: Boolean(prowlarr), url: prowlarr?.url || null, hasUiAuth: false } });
      }
      if (url.pathname === '/direct-api/api/settings/prowlarr' && request.method() === 'POST') {
        return route.fulfill({ json: { ok: true, version: '2.1.0' } });
      }
      if (url.pathname === '/direct-api/api/settings/prowlarr' && request.method() === 'PUT') {
        prowlarr = request.postDataJSON();
        return route.fulfill({ json: { configured: true, persisted: true, url: prowlarr.url, hasUiAuth: false } });
      }
      if (url.pathname === '/direct-api/api/settings/prowlarr' && request.method() === 'DELETE') {
        prowlarr = null;
        return route.fulfill({ json: { configured: false, persisted: false, url: null, hasUiAuth: false } });
      }
      if (url.pathname === '/direct-api/api/prowlarr/indexers') {
        return route.fulfill({ json: [{ id: 7, name: 'Fixture indexer', enabled: true, status: 'enabled', protocol: 'torrent', supportsSearch: true }] });
      }
      if (url.pathname === '/direct-api/api/prowlarr/search' && request.method() === 'POST') {
        if (failSearch) return route.fulfill({ status: 502, json: { error: 'Prowlarr search request failed' } });
        return route.fulfill({ json: searchResults });
      }
    }
    if (url.pathname.startsWith('/api/workspace/')) {
      workspaceRequests.push({ path: url.pathname, body: request.postData() || '' });
      return route.fulfill({ json: {} });
    }
    return route.fulfill({ status: 404, body: 'not found' });
  });

  await page.goto('http://localhost:3000/#discovery');
  await page.waitForFunction(() => document.querySelector('[data-node-id="metadata-one"]')?.dataset.selected === 'true');
  await page.waitForFunction(() => document.getElementById('prowlarr-not-configured').hidden === false);
  await page.click('[data-view="settings"]');
  await page.locator('#settings-prowlarr-node-note').getByText('Not configured on fossil.').waitFor();
  const [composeDownload] = await Promise.all([
    page.waitForEvent('download'),
    page.click('#download-prowlarr-compose'),
  ]);
  assert.equal(composeDownload.suggestedFilename(), 'prowlarr-compose.yaml');
  const compose = await readFile(await composeDownload.path(), 'utf8');
  assert.match(compose, /127\.0\.0\.1:9696:9696/);
  assert.match(compose, /network_mode: service:tailscale/);
  assert.match(compose, /prowlarr-config:\/config/);
  assert.match(compose, /lscr\.io\/linuxserver\/prowlarr:latest/);
  assert.doesNotMatch(compose, /apiKey|api_key|secret|docker\.sock|\/shared|\/data/i);
  await page.locator('.managed-prowlarr-setup details summary').click();
  assert.match(await page.locator('.managed-prowlarr-setup details').textContent(), /restore.*fresh managed instance/i);
  assert.match(await page.locator('.managed-prowlarr-setup details').textContent(), /roll back/i);
  for (const viewport of [{ width: 1280, height: 800 }, { width: 375, height: 812 }]) {
    await page.setViewportSize(viewport);
    assert.ok(await page.locator('#download-prowlarr-compose').isVisible());
    assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth));
    const composeBounds = await page.locator('#download-prowlarr-compose').boundingBox();
    assert.ok(composeBounds && composeBounds.x >= 0 && composeBounds.x + composeBounds.width <= viewport.width);
  }
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.click('#settings-configure-prowlarr');
  await page.locator('#settings-prowlarr-dialog').waitFor({ state: 'visible' });
  await page.locator('#settings-prowlarr-url').fill('https://prowlarr.example.test');
  await page.locator('#settings-prowlarr-key').fill(apiKey);
  await page.click('#settings-test-prowlarr');
  await page.getByText('Prowlarr connection succeeded (version 2.1.0).').waitFor();
  await page.click('#settings-save-prowlarr');
  await page.locator('#settings-prowlarr-saved').waitFor({ state: 'visible' });
  assert.equal(prowlarr.url, 'https://prowlarr.example.test');
  assert.equal(prowlarr.apiKey, apiKey);
  assert.equal(await page.locator('#settings-prowlarr-key').inputValue(), '');
  assert.doesNotMatch(await page.locator('#settings-prowlarr-saved').textContent(), /prowlarr-secret-api-key/);
  assert.ok(directRequests.some(request => request.path === '/direct-api/api/settings/prowlarr' && request.method === 'POST' && request.body.includes(apiKey)));
  assert.ok(directRequests.some(request => request.path === '/direct-api/api/settings/prowlarr' && request.method === 'PUT' && request.body.includes(apiKey)));
  assert.ok(workspaceRequests.every(request => !request.body.includes(apiKey) && !request.body.includes(identity)));

  await page.click('[data-view="discovery"]');
  await page.locator('#prowlarr-indexers .discovery-indexer').waitFor();
  assert.match(await page.locator('#prowlarr-discovery-status').textContent(), /Connected to Prowlarr on fossil/);
  assert.match(await page.locator('#prowlarr-indexers').textContent(), /Fixture indexer.*Enabled.*Search supported/);
  await page.locator('#prowlarr-search-query').fill('example title');
  await page.click('#prowlarr-search-button');
  await page.getByText('1 result returned.').waitFor();
  assert.equal(await page.locator('.discovery-result h3').textContent(), '<img src=x onerror=alert(1)>');
  assert.equal(await page.locator('.discovery-result img').count(), 0);
  assert.match(await page.locator('.discovery-result').textContent(), /Fixture indexer.*torrent.*Result ID: stable-result-id.*Categories: Movies/);
  assert.ok(directRequests.some(request => request.path === '/direct-api/api/prowlarr/search' && request.method === 'POST' && request.body.includes('example title')));
  assert.ok(directRequests.every(request => !request.path.startsWith('/api/prowlarr/') || !request.body.includes(apiKey)));
  assert.ok(workspaceRequests.every(request => !request.body.includes(apiKey)));
  assert.ok(coordinatorPaths.every(path => !path.includes('prowlarr')));

  failSearch = true;
  await page.locator('#prowlarr-search-query').fill('upstream failure');
  await page.click('#prowlarr-search-button');
  await page.getByText('Prowlarr search request failed').waitFor();
  for (const viewport of [{ width: 1280, height: 800 }, { width: 375, height: 812 }]) {
    await page.setViewportSize(viewport);
    assert.ok(await page.locator('#prowlarr-search-query').isVisible());
    assert.ok(await page.locator('#prowlarr-search-button').isVisible());
    assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= document.documentElement.clientWidth));
    const buttonBounds = await page.locator('#prowlarr-search-button').boundingBox();
    assert.ok(buttonBounds && buttonBounds.x >= 0 && buttonBounds.x + buttonBounds.width <= viewport.width);
  }
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.click('[data-view="settings"]');
  await page.click('#settings-configure-prowlarr');
  await page.locator('#settings-prowlarr-dialog').waitFor({ state: 'visible' });
  await page.locator('#settings-clear-prowlarr').waitFor({ state: 'visible' });
  await page.click('#settings-clear-prowlarr');
  await page.locator('#settings-prowlarr-saved').waitFor({ state: 'hidden' });
  assert.equal(prowlarr, null);
  assert.ok(directRequests.some(request => request.path === '/direct-api/api/settings/prowlarr' && request.method === 'DELETE'));
  assert.deepEqual(pageErrors, []);
});
