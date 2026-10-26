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

test('coordinator saves Radarr and Sonarr settings over the direct metadata route only', async t => {
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const page = await browser.newPage();
  const identity = 'AGE-SECRET-KEY-PQ-1browser-only-test-fixture';
  const workspaceRequests = [];
  const directRequests = [];
  let vaultUnlocked = false;
  let sonarr = null;
  let radarr = null;

  await page.route('http://rawrarr.test/**', async route => {
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
    if (url.pathname === '/api/workspace/profile') return route.fulfill({ json: { displayName: 'Test user' } });
    if (url.pathname === '/api/workspace/nodes') return route.fulfill({ json: [{ nodeId: 'metadata-one', role: 'metadata', online: true, relayAvailable: true, directUrl: 'http://rawrarr.test/direct-api/' }] });
    if (url.pathname === '/api/workspace/pending') return route.fulfill({ json: [] });
    if (url.pathname === '/api/workspace/direct-session') {
      workspaceRequests.push({ path: url.pathname, body: request.postData() || '' });
      return route.fulfill({ status: 204, headers: { 'set-cookie': `rawrarr_direct_session=${'a'.repeat(64)}; Path=/direct-api/; SameSite=Strict` } });
    }
    if (url.pathname === '/direct-api/api/direct/health') return route.fulfill({ status: 204 });
    if (url.pathname.startsWith('/direct-api/api/')) {
      directRequests.push({ path: url.pathname, method: request.method(), body: request.postData() || '' });
      if (url.pathname === '/direct-api/api/settings/age' && request.method() === 'GET') {
        return route.fulfill({ json: { unlocked: vaultUnlocked, encryptedConfigExists: true } });
      }
      if (url.pathname === '/direct-api/api/settings/age' && request.method() === 'POST') {
        const body = request.postDataJSON();
        assert.equal(body.identity, identity);
        assert.equal(body.rememberOnDevice, false);
        vaultUnlocked = true;
        return route.fulfill({ json: { unlocked: true, encryptedConfigExists: true } });
      }
      if (url.pathname === '/direct-api/api/settings/sonarr' && request.method() === 'GET') {
        return route.fulfill({ json: { configured: Boolean(sonarr), persisted: Boolean(sonarr), url: sonarr?.url || null } });
      }
      if (url.pathname === '/direct-api/api/settings/sonarr' && request.method() === 'PUT') {
        sonarr = request.postDataJSON();
        return route.fulfill({ json: { configured: true, persisted: true, url: sonarr.url } });
      }
      if (url.pathname === '/direct-api/api/settings/sonarr' && request.method() === 'DELETE') {
        sonarr = null;
        return route.fulfill({ json: { configured: false, persisted: false, url: null } });
      }
      if (url.pathname === '/direct-api/api/sonarr/series') {
        return route.fulfill({ json: [{ id: 7, title: 'Example Show', year: 2024, tvdbId: 7123, monitored: true, episodeFileCount: 3, episodeCount: 8, sizeOnDisk: 1536, qualityProfile: 'HD-1080p' }] });
      }
      if (url.pathname === '/direct-api/api/settings/radarr' && request.method() === 'GET') {
        return route.fulfill({ json: { configured: Boolean(radarr), persisted: Boolean(radarr), hasUiAuth: Boolean(radarr?.uiUsername && radarr?.uiPassword), url: radarr?.url || null } });
      }
      if (url.pathname === '/direct-api/api/settings/radarr' && request.method() === 'PUT') {
        radarr = request.postDataJSON();
        return route.fulfill({ json: { configured: true, persisted: true, hasUiAuth: Boolean(radarr.uiUsername && radarr.uiPassword), url: radarr.url } });
      }
      if (url.pathname === '/direct-api/api/settings/radarr' && request.method() === 'DELETE') {
        radarr = null;
        return route.fulfill({ json: { configured: false, persisted: false, url: null } });
      }
    }
    if (url.pathname.startsWith('/api/workspace/')) {
      workspaceRequests.push({ path: url.pathname, body: request.postData() || '' });
      return route.fulfill({ json: {} });
    }
    return route.fulfill({ status: 404, body: 'not found' });
  });

  await page.goto('http://rawrarr.test/');
  await page.waitForFunction(() => document.querySelector('[data-node-id="metadata-one"]')?.dataset.selected === 'true');
  await page.click('[data-view="movies"]');
  assert.equal(await page.locator('#radarr-connection-card').isVisible(), false, 'Radarr form must start hidden until its saved state is loaded');
  assert.equal(await page.locator('#remember-age-id').count(), 0);
  await page.locator('#age-id').fill(identity);
  await page.locator('#unlock').click();
  await page.waitForFunction(() => document.getElementById('age-id').value === '');
  assert.equal(await page.evaluate(() => localStorage.getItem('rawrarr.browser.age-id')), null);
  await page.locator('#radarr-node-note').getByText('Radarr is not configured on metadata-one.').waitFor();
  await page.click('[data-view="series"]');
  assert.equal(new URL(page.url()).hash, '#tv');
  await page.locator('#sonarr-settings-message').getByText('Enter your Sonarr HTTPS URL and API key.').waitFor();
  await page.locator('#sonarr-node-note').getByText('Sonarr is not configured on metadata-one.').waitFor();
  await page.locator('#sonarr-url').fill('https://sonarr.example.test');
  await page.locator('#sonarr-key').fill('sonarr-secret-api-key');
  await page.click('#save-sonarr');
  await page.locator('#sonarr-connection-card').waitFor({ state: 'hidden' });
  await page.locator('#sonarr-series .library-card').waitFor();
  assert.equal(await page.locator('#sonarr-series .library-card-title').textContent(), 'Example Show');
  assert.match(await page.locator('#sonarr-series .library-card').textContent(), /2024.*3 \/ 8 episodes.*HD-1080p/);
  assert.equal(await page.locator('#tv-count').textContent(), '1 of 1 series');
  await page.locator('#tv-search').fill('absent');
  assert.equal(await page.locator('#sonarr-series .library-empty').textContent(), 'No titles match this search or filter.');
  assert.equal(await page.locator('#tv-series-count').textContent(), '1');
  assert.equal(await page.locator('#tv-episodes-present').textContent(), '3');
  assert.ok(directRequests.some(request => request.path === '/direct-api/api/sonarr/series' && request.method === 'GET' && !request.body.includes('sonarr-secret-api-key')));

  assert.equal(sonarr.url, 'https://sonarr.example.test');
  assert.equal(sonarr.apiKey, 'sonarr-secret-api-key');
  assert.equal(await page.locator('#sonarr-key').inputValue(), '');
  assert.ok(directRequests.some(request => request.path === '/direct-api/api/settings/sonarr' && request.method === 'PUT' && request.body.includes('sonarr-secret-api-key')));
  assert.ok(directRequests.some(request => request.path === '/direct-api/api/settings/age' && request.method === 'POST'));
  assert.ok(workspaceRequests.every(request => !request.body.includes(identity) && !request.body.includes('sonarr-secret-api-key')));

  await page.click('[data-view="settings"]');
  await page.locator('#settings-sonarr-saved').waitFor({ state: 'visible' });
  await page.locator('#sonarr-connection-card').waitFor({ state: 'hidden' });
  assert.equal(await page.locator('#settings-sonarr-saved-url').textContent(), 'https://sonarr.example.test');
  assert.equal(await page.locator('#settings-sonarr-key').inputValue(), '');
  assert.doesNotMatch(await page.locator('#settings-sonarr-saved').textContent(), /sonarr-secret-api-key/);
  await page.click('#settings-clear-sonarr');
  await page.locator('#settings-sonarr-settings-message').getByText('Sonarr settings cleared from the age-encrypted config.').waitFor();
  await page.click('[data-view="series"]');
  await page.locator('#sonarr-connection-card').waitFor({ state: 'visible' });
  assert.equal(await page.locator('#tv-library').isVisible(), false);
  assert.equal(sonarr, null);
  assert.ok(directRequests.some(request => request.path === '/direct-api/api/settings/sonarr' && request.method === 'DELETE'));

  await page.click('[data-view="movies"]');
  await page.locator('#radarr-settings-message').getByText('Enter your Radarr HTTPS URL and API key.').waitFor();
  await page.locator('#radarr-url').fill('https://radarr.example.test');
  await page.locator('#radarr-key').fill('radarr-secret-api-key');
  await page.locator('#radarr-ui-username').fill('radarr-ui-user');
  await page.locator('#radarr-ui-password').fill('radarr-ui-password-secret');
  await page.click('#save-radarr');
  await page.locator('#radarr-connection-card').waitFor({ state: 'hidden' });
  assert.equal(radarr.url, 'https://radarr.example.test');
  assert.equal(radarr.apiKey, 'radarr-secret-api-key');
  assert.equal(radarr.uiUsername, 'radarr-ui-user');
  assert.equal(radarr.uiPassword, 'radarr-ui-password-secret');
  assert.equal(await page.locator('#radarr-key').inputValue(), '');
  assert.equal(await page.locator('#radarr-ui-username').inputValue(), '');
  assert.equal(await page.locator('#radarr-ui-password').inputValue(), '');
  assert.ok(directRequests.some(request => request.path === '/direct-api/api/settings/radarr' && request.method === 'PUT' && request.body.includes('radarr-secret-api-key')));
  assert.ok(directRequests.some(request => request.path === '/direct-api/api/settings/radarr' && request.method === 'PUT' && request.body.includes('radarr-ui-password-secret')));
  assert.ok(directRequests.filter(request => request.path === '/direct-api/api/settings/radarr' && request.method === 'GET').every(request => !request.body.includes('radarr-ui-password-secret')));
  assert.ok(workspaceRequests.every(request => !request.body.includes('radarr-ui-user') && !request.body.includes('radarr-ui-password-secret')));
  await page.click('[data-view="settings"]');
  await page.locator('#settings-radarr-saved').waitFor({ state: 'visible' });
  await page.locator('#radarr-connection-card').waitFor({ state: 'hidden' });
  assert.equal(await page.locator('#settings-radarr-saved-url').textContent(), 'https://radarr.example.test');
  assert.equal(await page.locator('#settings-radarr-key').inputValue(), '');
  assert.equal(await page.locator('#settings-radarr-ui-username').inputValue(), '');
  assert.equal(await page.locator('#settings-radarr-ui-password').inputValue(), '');
  assert.doesNotMatch(await page.locator('#settings-radarr-saved').textContent(), /radarr-secret-api-key/);
  await page.click('#settings-clear-radarr');
  await page.locator('#settings-radarr-settings-message').getByText('Radarr settings cleared from the age-encrypted config.').waitFor();
  await page.click('[data-view="movies"]');
  await page.locator('#radarr-connection-card').waitFor({ state: 'visible' });
  assert.equal(radarr, null);
  assert.ok(directRequests.some(request => request.path === '/direct-api/api/settings/radarr' && request.method === 'DELETE'));
  assert.ok(workspaceRequests.every(request => !request.body.includes('radarr-secret-api-key')));
});

test('coordinator opens TV from #tv and upgrades the old #series link', async t => {
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const page = await browser.newPage();
  await page.route('http://rawrarr.test/**', async route => {
    const path = new URL(route.request().url()).pathname;
    if (path === '/health') return route.fulfill({ status: 200, body: 'ok' });
    if (path === '/') return route.fulfill({ body: html, contentType: 'text/html' });
    if (path === '/assets/coordinator-workspace.mjs') return route.fulfill({ body: script, contentType: 'text/javascript' });
    if (path === '/assets/coordinator-availability.mjs') return route.fulfill({ body: availabilityModule, contentType: 'text/javascript' });
    if (path === '/assets/coordinator-device-routes.mjs') return route.fulfill({ body: deviceRoutes, contentType: 'text/javascript' });
    if (path === '/assets/coordinator-node-avatar.mjs') return route.fulfill({ body: nodeAvatar, contentType: 'text/javascript' });
    if (path === '/assets/profile-color.mjs') return route.fulfill({ body: profileColor, contentType: 'text/javascript' });
    if (path === '/assets/coordinator-ai.mjs') return route.fulfill({ body: aiModule, contentType: 'text/javascript' });
    if (path === '/api/workspace/profile') return route.fulfill({ json: { displayName: 'Test user' } });
    if (path === '/api/workspace/nodes') return route.fulfill({ json: [] });
    if (path === '/api/workspace/pending') return route.fulfill({ json: [] });
    return route.fulfill({ status: 404, body: 'not found' });
  });
  await page.goto('http://rawrarr.test/#tv');
  assert.equal(await page.locator('#series-view').isVisible(), true);
  assert.equal(new URL(page.url()).hash, '#tv');
  await page.goto('http://rawrarr.test/?legacy=1#series');
  assert.equal(await page.locator('#series-view').isVisible(), true);
  assert.equal(new URL(page.url()).hash, '#tv');
});

test('Movies and TV do not display cached library data without an age ID', async t => {
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const page = await browser.newPage();
  const moviesCache = [{ title: 'Private Movie Cache Fixture', year: 2025, tmdbId: 99901, monitored: true, hasFile: true }];
  const tvCache = [{ title: 'Private TV Cache Fixture', year: 2025, tvdbId: 99902, monitored: true, episodeFileCount: 1, episodeCount: 1 }];
  await page.addInitScript(({ moviesCache, tvCache }) => {
    localStorage.setItem('rawrarr.library-text.v1:metadata-one:movies', JSON.stringify({ savedAt: Date.now(), items: moviesCache }));
    localStorage.setItem('rawrarr.library-text.v1:metadata-one:tv', JSON.stringify({ savedAt: Date.now(), items: tvCache }));
  }, { moviesCache, tvCache });
  await page.route('http://rawrarr.test/**', async route => {
    const url = new URL(route.request().url());
    if (url.pathname === '/health') return route.fulfill({ status: 200, body: 'ok' });
    if (url.pathname === '/') return route.fulfill({ body: html, contentType: 'text/html' });
    if (url.pathname === '/assets/coordinator-workspace.mjs') return route.fulfill({ body: script, contentType: 'text/javascript' });
    if (url.pathname === '/assets/coordinator-availability.mjs') return route.fulfill({ body: availabilityModule, contentType: 'text/javascript' });
    if (url.pathname === '/assets/coordinator-device-routes.mjs') return route.fulfill({ body: deviceRoutes, contentType: 'text/javascript' });
    if (url.pathname === '/assets/coordinator-node-avatar.mjs') return route.fulfill({ body: nodeAvatar, contentType: 'text/javascript' });
    if (url.pathname === '/assets/profile-color.mjs') return route.fulfill({ body: profileColor, contentType: 'text/javascript' });
    if (url.pathname === '/assets/coordinator-ai.mjs') return route.fulfill({ body: aiModule, contentType: 'text/javascript' });
    if (url.pathname === '/api/workspace/profile') return route.fulfill({ json: { displayName: 'Test user' } });
    if (url.pathname === '/api/workspace/nodes') return route.fulfill({ json: [{ nodeId: 'metadata-one', role: 'metadata', online: true, relayAvailable: true, directUrl: 'http://rawrarr.test/direct-api/' }] });
    if (url.pathname === '/api/workspace/pending') return route.fulfill({ json: [] });
    if (url.pathname === '/direct-api/api/direct/health') return route.fulfill({ status: 204 });
    return route.fulfill({ status: 404, body: 'not found' });
  });

  await page.goto('http://rawrarr.test/');
  await page.waitForFunction(() => document.querySelector('[data-node-id="metadata-one"]')?.dataset.selected === 'true');
  assert.equal(await page.evaluate(() => localStorage.getItem('rawrarr.browser.age-id')), null);

  await page.click('[data-view="movies"]');
  assert.equal(await page.locator('#movies .library-card').count(), 0);
  assert.doesNotMatch(await page.locator('#movies-view').textContent(), /Private Movie Cache Fixture/);

  await page.click('[data-view="series"]');
  assert.equal(await page.locator('#sonarr-series .library-card').count(), 0);
  assert.doesNotMatch(await page.locator('#series-view').textContent(), /Private TV Cache Fixture/);
});

test('age ID is remembered after validation, restored on reload, and cleared on sign-out', async t => {
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const page = await browser.newPage();
  const identity = 'AGE-SECRET-KEY-PQ-1remembered-test-fixture';
  let vaultUnlocked = false;
  await page.route('http://rawrarr.test/**', async route => {
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
    if (url.pathname === '/assets/metadata-export.js') return route.fulfill({ contentType: 'text/javascript', body: "window.rawrarrMiniExport = { loadRadarrLibrary: async () => [], deriveProfileRecipient: async identity => { if (!identity.startsWith('AGE-SECRET-KEY-PQ-1')) throw new Error('invalid identity'); return 'age1pq1validrecipient'; } };" });
    if (url.pathname === '/api/workspace/profile') return route.fulfill({ json: { displayName: 'Test user' } });
    if (url.pathname === '/api/workspace/nodes') return route.fulfill({ json: [{ nodeId: 'metadata-one', role: 'metadata', online: true, relayAvailable: true, directUrl: 'http://rawrarr.test/direct-api/' }] });
    if (url.pathname === '/api/workspace/pending') return route.fulfill({ json: [] });
    if (url.pathname === '/api/workspace/direct-session') return route.fulfill({ status: 204, headers: { 'set-cookie': `rawrarr_direct_session=${'b'.repeat(64)}; Path=/direct-api/; SameSite=Strict` } });
    if (url.pathname === '/direct-api/api/direct/health') return route.fulfill({ status: 204 });
    if (url.pathname === '/direct-api/api/settings/age' && request.method() === 'GET') return route.fulfill({ json: { unlocked: vaultUnlocked, encryptedConfigExists: true } });
    if (url.pathname === '/direct-api/api/settings/age' && request.method() === 'POST') {
      const body = request.postDataJSON();
      assert.equal(body.identity, identity);
      vaultUnlocked = true;
      return route.fulfill({ json: { unlocked: true, encryptedConfigExists: true } });
    }
    if (url.pathname === '/direct-api/api/settings/radarr') return route.fulfill({ json: { configured: false, persisted: false, url: null } });
    if (url.pathname.startsWith('/direct-api/api/')) return route.fulfill({ json: {} });
    if (url.pathname.startsWith('/api/workspace/')) return route.fulfill({ json: {} });
    return route.fulfill({ status: 404, body: 'not found' });
  });

  await page.goto('http://rawrarr.test/#movies');
  await page.waitForFunction(() => document.querySelector('[data-node-id="metadata-one"]')?.dataset.selected === 'true');
  await page.locator('#age-id').fill(identity);
  await page.click('#unlock');
  await page.waitForFunction(() => document.getElementById('age-id').value === '');
  await page.waitForFunction(key => localStorage.getItem('rawrarr.browser.age-id') === key, identity);
  assert.equal(await page.evaluate(key => localStorage.getItem('rawrarr.browser.age-id') === key, identity), true);
  await page.reload();
  await page.waitForFunction(() => document.querySelector('[data-node-id="metadata-one"]')?.dataset.selected === 'true');
  assert.equal(await page.evaluate(key => localStorage.getItem('rawrarr.browser.age-id') === key, identity), true);
  assert.equal(await page.locator('#age-id-entry').isVisible(), false);
  await page.click('#sign-out');
  await page.waitForURL('**/auth');
  assert.equal(await page.evaluate(() => localStorage.getItem('rawrarr.browser.age-id')), null);
});

test('Settings imports an age ID locally without an online node and clears it manually', async t => {
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const page = await browser.newPage();
  const identity = 'AGE-SECRET-KEY-PQ-1settings-test-fixture';
  let unlocked = false;
  let directAgeValidationRequests = 0;
  await page.route('http://rawrarr.test/**', async route => {
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
    if (url.pathname === '/assets/metadata-export.js') return route.fulfill({ contentType: 'text/javascript', body: "window.rawrarrMiniExport = { loadRadarrLibrary: async () => [], deriveProfileRecipient: async identity => { if (!identity.startsWith('AGE-SECRET-KEY-PQ-1')) throw new Error('invalid identity'); return 'age1pq1validrecipient'; } };" });
    if (url.pathname === '/api/workspace/profile') return route.fulfill({ json: { displayName: 'Test user' } });
    if (url.pathname === '/api/workspace/nodes') return route.fulfill({ json: [] });
    if (url.pathname === '/api/workspace/pending') return route.fulfill({ json: [] });
    if (url.pathname === '/api/workspace/direct-session') return route.fulfill({ status: 204, headers: { 'set-cookie': `rawrarr_direct_session=${'c'.repeat(64)}; Path=/direct-api/; SameSite=Strict` } });
    if (url.pathname === '/direct-api/api/direct/health') return route.fulfill({ status: 204 });
    if (url.pathname === '/direct-api/api/settings/age' && request.method() === 'POST') {
      directAgeValidationRequests += 1;
      const body = request.postDataJSON();
      assert.equal(body.identity, identity);
      assert.equal(body.rememberOnDevice, false);
      unlocked = true;
      return route.fulfill({ json: { unlocked: true, encryptedConfigExists: true } });
    }
    if (url.pathname === '/direct-api/api/settings/age' && request.method() === 'GET') return route.fulfill({ json: { unlocked, encryptedConfigExists: true } });
    if (url.pathname === '/direct-api/api/radarr/live-library') return route.fulfill({ json: [] });
    if (url.pathname.startsWith('/direct-api/api/')) return route.fulfill({ json: {} });
    if (url.pathname.startsWith('/api/workspace/')) return route.fulfill({ json: {} });
    return route.fulfill({ status: 404, body: 'not found' });
  });

  await page.goto('http://rawrarr.test/#settings');
  await page.getByText('Age private ID is not stored in this browser.').waitFor();
  await page.locator('#settings-age-id').fill(identity);
  await page.click('#save-age-id');
  await page.locator('#age-storage-message').getByText('Age private ID checked and saved in this browser.').waitFor();
  assert.equal(await page.evaluate(key => localStorage.getItem('rawrarr.browser.age-id') === key, identity), true);
  assert.equal(directAgeValidationRequests, 0);
  assert.equal(await page.locator('#settings-age-id').inputValue(), '');
  assert.equal(await page.locator('#remember-age-id').count(), 0);

  await page.click('#remove-age-id');
  await page.getByText('Age private ID removed from this browser.').waitFor();
  assert.equal(await page.evaluate(() => localStorage.getItem('rawrarr.browser.age-id')), null);
  assert.equal(await page.locator('#remember-age-id').count(), 0);
});

test('Settings rejects an invalid age ID without persisting it', async t => {
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const page = await browser.newPage();
  await page.route('http://rawrarr.test/**', async route => {
    const url = new URL(route.request().url());
    if (url.pathname === '/health') return route.fulfill({ status: 200, body: 'ok' });
    if (url.pathname === '/') return route.fulfill({ body: html, contentType: 'text/html' });
    if (url.pathname === '/assets/coordinator-workspace.mjs') return route.fulfill({ body: script, contentType: 'text/javascript' });
    if (url.pathname === '/assets/coordinator-availability.mjs') return route.fulfill({ body: availabilityModule, contentType: 'text/javascript' });
    if (url.pathname === '/assets/coordinator-device-routes.mjs') return route.fulfill({ body: deviceRoutes, contentType: 'text/javascript' });
    if (url.pathname === '/assets/coordinator-node-avatar.mjs') return route.fulfill({ body: nodeAvatar, contentType: 'text/javascript' });
    if (url.pathname === '/assets/profile-color.mjs') return route.fulfill({ body: profileColor, contentType: 'text/javascript' });
    if (url.pathname === '/assets/coordinator-ai.mjs') return route.fulfill({ body: aiModule, contentType: 'text/javascript' });
    if (url.pathname === '/assets/metadata-export.js') return route.fulfill({ contentType: 'text/javascript', body: "window.rawrarrMiniExport = { deriveProfileRecipient: async () => { throw new Error('invalid identity'); } };" });
    if (url.pathname === '/api/workspace/profile') return route.fulfill({ json: { displayName: 'Test user' } });
    if (url.pathname === '/api/workspace/nodes') return route.fulfill({ json: [] });
    if (url.pathname === '/api/workspace/pending') return route.fulfill({ json: [] });
    if (url.pathname.startsWith('/api/workspace/')) return route.fulfill({ json: {} });
    return route.fulfill({ status: 404, body: 'not found' });
  });
  await page.goto('http://rawrarr.test/#settings');
  await page.locator('#settings-age-id').fill('not-an-age-identity');
  await page.click('#save-age-id');
  await page.getByText('invalid identity').waitFor();
  assert.equal(await page.evaluate(() => localStorage.getItem('rawrarr.browser.age-id')), null);
  assert.equal(await page.locator('#settings-age-id').inputValue(), '');
});
