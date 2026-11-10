import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const root = new URL('../../', import.meta.url);
const workspaceHtml = await readFile(new URL('rust-arr/src/coordinator_workspace.html', root), 'utf8');
const moduleFiles = new Map(await Promise.all([
  'coordinator-workspace.mjs',
  'coordinator-device-routes.mjs',
  'coordinator-availability.mjs',
  'coordinator-node-avatar.mjs',
  'profile-color.mjs',
  'coordinator-ai.mjs',
].map(async name => [name, await readFile(new URL(`rust-arr/src/${name}`, root), 'utf8')])));
const metadataExportStub = `window.rawrarrMiniExport = {
  loadDeviceNames: async () => ({ 'fossil-1': 'Will’s MacBook fossil' }),
  deriveProfileRecipient: async () => 'age1pq1fixture-recipient',
  prepareGroupIdentityForDevice: async () => false,
  encryptIdentityForDevice: async () => 'fixture-envelope',
};`;

async function openSettings(t) {
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const context = await browser.newContext();
  const page = await context.newPage();
  const connections = {
    radarr: { configured: true, persisted: true, url: 'https://radarr.example.test' },
    sonarr: { configured: false, persisted: false, url: null },
    lidarr: { configured: false, persisted: false, url: null },
  };
  const requests = [];
  const externalRequests = [];
  page.on('request', request => {
    if (request.url().startsWith('https://radarr.example.test') || request.url().startsWith('https://sonarr.example.test') || request.url().startsWith('https://lidarr.example.test')) {
      externalRequests.push({ url: request.url(), method: request.method(), headers: request.headers(), body: request.postData() });
    }
  });
  await page.addInitScript(() => localStorage.setItem('rawrarr.browser.age-id', 'AGE-SECRET-KEY-PQ-fixture'));
  await page.route('**/*', async route => {
    const request = route.request();
    const url = new URL(request.url());
    const path = url.pathname;
    const json = value => route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(value) });
    if (url.hostname === 'coordinator.test' && path === '/') return route.fulfill({ contentType: 'text/html', body: workspaceHtml });
    if (url.hostname === 'coordinator.test' && path.startsWith('/assets/')) {
      const name = path.slice('/assets/'.length);
      if (name === 'coordinator-workspace.mjs') return route.fulfill({ contentType: 'text/javascript', body: moduleFiles.get(name) });
      if (name === 'metadata-export.js') return route.fulfill({ contentType: 'text/javascript', body: metadataExportStub });
      if (moduleFiles.has(name)) return route.fulfill({ contentType: 'text/javascript', body: moduleFiles.get(name) });
      return route.fulfill({ status: 404, body: '' });
    }
    if (url.hostname === 'coordinator.test' && path === '/health') return json({ ok: true });
    if (url.hostname === 'coordinator.test' && path === '/api/workspace/nodes') return json([{
      nodeId: 'fossil-1', role: 'metadata', online: true, relayAvailable: true,
      directUrl: 'http://coordinator.test/node-direct', displayName: 'Test fossil',
    }]);
    if (url.hostname === 'coordinator.test' && path === '/api/workspace/pending') return json([]);
    if (url.hostname === 'coordinator.test' && path === '/api/workspace/direct-session') return json({ token: 'fixture-session' });
    if (url.hostname === 'coordinator.test' && path === '/api/workspace/profile') return json({ accountId: 'owner', displayName: 'Test owner' });
    if (url.hostname === 'coordinator.test' && path === '/api/crypto/access') return json({ tier: 'free', fossilNodes: 1, fossilNodeLimit: 5 });
    if (url.hostname === 'coordinator.test' && path === '/api/auth/eth/link') return json({ wallets: [] });
    if (url.hostname === 'coordinator.test' && path === '/api/workspace/billing') return json({ checkoutEnabled: false });
    if (url.hostname === 'coordinator.test' && path === '/nodes/fossil-1/api/metadata/export/node-recipient') {
      return route.fulfill({ status: 200, contentType: 'text/plain', body: 'age1pq1fixture-recipient' });
    }
    if (url.hostname === 'coordinator.test' && path === '/node-direct/health') return json({ artworkCacheConfigured: false, artworkCacheReachable: false });
    if (url.hostname === 'coordinator.test' && path.startsWith('/node-direct/api/')) {
      const apiPath = path.slice('/node-direct/api'.length);
      const method = request.method();
      let body = {};
      try { body = request.postDataJSON() || {}; } catch { /* GET has no JSON body. */ }
      if (apiPath.startsWith('/settings/')) requests.push({ path: apiPath, method, body });
      if (apiPath === '/settings/age' && method === 'GET') return json({ unlocked: true, encryptedConfigExists: true });
      if (apiPath === '/settings/age' && method === 'POST') return json({ unlocked: true });
      if (apiPath === '/settings/radarr' && method === 'GET') return json(connections.radarr);
      if (apiPath === '/settings/sonarr' && method === 'GET') return json(connections.sonarr);
      if (apiPath === '/settings/lidarr' && method === 'GET') return json(connections.lidarr);
      if (apiPath === '/settings/radarr' && method === 'POST') return json({ ok: true, version: '5.0.0' });
      if (apiPath === '/settings/sonarr' && method === 'POST') return json({ ok: true, version: '4.0.0' });
      if (apiPath === '/settings/lidarr' && method === 'POST') return json({ ok: true, version: '2.0.0' });
      if (apiPath === '/settings/radarr' && method === 'PUT') {
        connections.radarr = { configured: true, persisted: true, url: body.url, hasUiAuth: Boolean(body.uiUsername) };
        return json(connections.radarr);
      }
      if (apiPath === '/settings/sonarr' && method === 'PUT') {
        connections.sonarr = { configured: true, persisted: true, url: body.url };
        return json(connections.sonarr);
      }
      if (apiPath === '/settings/lidarr' && method === 'PUT') {
        connections.lidarr = { configured: true, persisted: true, url: body.url };
        return json(connections.lidarr);
      }
      if (apiPath === '/settings/radarr' && method === 'DELETE') {
        connections.radarr = { configured: false, persisted: false, url: null };
        return json(connections.radarr);
      }
      if (apiPath === '/settings/sonarr' && method === 'DELETE') {
        connections.sonarr = { configured: false, persisted: false, url: null };
        return json(connections.sonarr);
      }
      if (apiPath === '/settings/lidarr' && method === 'DELETE') {
        connections.lidarr = { configured: false, persisted: false, url: null };
        return json(connections.lidarr);
      }
      if (apiPath === '/radarr/live-library') return json([]);
      if (apiPath === '/lidarr/library') return json({
        source: 'lidarr',
        artists: [{ provider: 'lidarr', id: 1, name: 'Fixture Artist', foreignArtistId: 'musicbrainz-artist', albumCount: 1, trackFileCount: 3 }],
        releases: [{ provider: 'lidarr', id: 2, artistId: 1, artistName: 'Fixture Artist', title: 'Fixture Release', foreignAlbumId: 'musicbrainz-release', releaseDate: '2024-01-02', trackCount: 3, trackFileCount: 3 }],
      });
      if (apiPath === '/metadata/storage') return json({ totalBytes: 0, files: 0 });
      return json({});
    }
    if (url.hostname === 'coordinator.test' && path.startsWith('/api/workspace/device-picture/')) return route.fulfill({ status: 404, body: '' });
    return route.fulfill({ status: 404, contentType: 'application/json', body: JSON.stringify({ error: `unmocked ${request.method()} ${path}` }) });
  });
  await page.goto('http://coordinator.test/#settings');
  await page.getByRole('button', { name: 'Configure Sonarr' }).waitFor({ state: 'visible' });
  await page.getByText('Radarr details saved').waitFor({ state: 'visible' });
  return { page, context, connections, requests, externalRequests };
}

test('media service cards show configured and unconfigured states in compact dialogs', async t => {
  const { page, context } = await openSettings(t);
  assert.equal(await page.locator('#settings-radarr-saved').isVisible(), true);
  assert.equal(await page.locator('#settings-radarr-saved-url').textContent(), 'https://radarr.example.test');
  assert.equal(await page.locator('#settings-sonarr-saved').isVisible(), false);
  assert.equal(await page.locator('#settings-sonarr-node-note').textContent(), 'Not configured on Will’s MacBook fossil. Add the connection when ready.');
  assert.match(await page.locator('#settings-radarr-node-note').textContent(), /Connected on Will’s MacBook fossil/);
  assert.equal(await page.locator('#settings-radarr-settings').isVisible(), false);
  assert.equal(await page.locator('#settings-sonarr-settings').isVisible(), false);
  assert.equal(await page.locator('#settings-lidarr-saved').isVisible(), false);
  assert.equal(await page.locator('#settings-lidarr-settings').isVisible(), false);

  await page.getByRole('button', { name: 'Configure Sonarr' }).click();
  const dialog = page.getByRole('dialog', { name: 'Configure Sonarr' });
  await dialog.waitFor({ state: 'visible' });
  await assert.doesNotReject(() => dialog.getByLabel('Sonarr URL').waitFor({ state: 'visible' }));
  await assert.doesNotReject(() => dialog.getByLabel('Sonarr API key').waitFor({ state: 'visible' }));
  await context.close();
});

test('Lidarr saves on the selected node and Music renders its artist and release library responsively', async t => {
  const { page, context, requests, externalRequests } = await openSettings(t);
  await page.getByRole('button', { name: 'Configure Lidarr' }).click();
  const dialog = page.getByRole('dialog', { name: 'Configure Lidarr' });
  await dialog.getByLabel('Lidarr URL').fill('https://lidarr.example.test');
  await dialog.getByLabel('Lidarr API key').fill('lidarr-secret-fixture');
  await dialog.getByRole('button', { name: 'Test connection' }).click();
  await page.getByText('Lidarr connection succeeded (version 2.0.0).').waitFor();
  await dialog.getByRole('button', { name: 'Save and connect' }).click();
  await page.getByText('Lidarr details saved').waitFor({ state: 'visible' });
  assert.equal(await page.locator('#settings-lidarr-key').inputValue(), '');
  assert.equal(await page.locator('body').innerText().then(text => text.includes('lidarr-secret-fixture')), false);
  assert.ok(requests.some(item => item.path === '/settings/lidarr' && item.method === 'POST' && item.body.apiKey === 'lidarr-secret-fixture'));
  assert.ok(requests.some(item => item.path === '/settings/lidarr' && item.method === 'PUT' && item.body.apiKey === 'lidarr-secret-fixture'));
  assert.ok(externalRequests.every(request => request.method === 'GET' && new URL(request.url).pathname === '/'));
  assert.ok(externalRequests.every(request => !request.headers['x-api-key'] && !request.body));

  await page.locator('.nav-item[data-view="music"]').click();
  await page.getByText('Loaded 1 artists and 1 releases from Lidarr').waitFor();
  await page.getByRole('heading', { name: 'Fixture Artist' }).waitFor();
  await page.getByRole('heading', { name: 'Fixture Release' }).waitFor();
  for (const viewport of [{ width: 1280, height: 800 }, { width: 375, height: 812 }]) {
    await page.setViewportSize(viewport);
    await page.waitForTimeout(100);
    const dimensions = await page.evaluate(() => ({
      scrollWidth: document.documentElement.scrollWidth,
      clientWidth: document.documentElement.clientWidth,
      refresh: document.getElementById('refresh-lidarr-library').getBoundingClientRect().toJSON(),
      configure: document.getElementById('configure-lidarr-from-music').getBoundingClientRect().toJSON(),
    }));
    assert.ok(dimensions.scrollWidth <= dimensions.clientWidth, `Music view overflows at ${viewport.width}px`);
    assert.ok(dimensions.refresh.width > 0 && dimensions.refresh.right <= viewport.width, `Music refresh action is not visible at ${viewport.width}px`);
    assert.ok(dimensions.configure.width > 0 && dimensions.configure.right <= viewport.width, `Music configure action is not visible at ${viewport.width}px`);
  }
  await page.locator('.nav-item[data-view="settings"]').click();
  await page.getByRole('button', { name: 'Change Lidarr settings' }).click();
  await page.getByRole('dialog', { name: 'Configure Lidarr' }).getByRole('button', { name: 'Clear connection' }).click();
  await page.locator('#settings-lidarr-saved').waitFor({ state: 'hidden' });
  assert.ok(requests.some(item => item.path === '/settings/lidarr' && item.method === 'DELETE'));
  await context.close();
});

test('Test validates from the selected node; Save persists and clears the secret fields', async t => {
  const { page, context, requests, externalRequests } = await openSettings(t);
  const dialog = page.getByRole('dialog', { name: 'Configure Sonarr' });
  await page.getByRole('button', { name: 'Configure Sonarr' }).click();
  await dialog.getByLabel('Sonarr URL').fill('https://sonarr.example.test');
  await dialog.getByLabel('Sonarr API key').fill('sonarr-secret-fixture');
  await dialog.getByRole('button', { name: 'Test connection' }).click();
  await page.getByText('Sonarr connection succeeded (version 4.0.0).').waitFor();
  const testRequest = requests.find(item => item.path === '/settings/sonarr' && item.method === 'POST');
  assert.deepEqual(testRequest?.body, { url: 'https://sonarr.example.test', apiKey: 'sonarr-secret-fixture' });
  assert.ok(externalRequests.every(request => request.method === 'GET' && new URL(request.url).pathname === '/'),
    'the browser only performs the existing credential-free reachability probe, not the service API test');
  assert.ok(externalRequests.every(request => !request.headers['x-api-key'] && !request.body),
    'the browser sends no credentials to Radarr or Sonarr');

  await dialog.getByRole('button', { name: 'Save and connect' }).click();
  await page.getByText('Sonarr details saved').waitFor({ state: 'visible' });
  assert.equal(await dialog.isVisible(), false);
  assert.equal(await page.locator('#settings-sonarr-key').inputValue(), '');
  assert.equal(await page.locator('#settings-sonarr-url').inputValue(), 'https://sonarr.example.test');
  assert.equal(await page.locator('body').innerText().then(text => text.includes('sonarr-secret-fixture')), false);
  const saveRequest = requests.find(item => item.path === '/settings/sonarr' && item.method === 'PUT');
  assert.equal(saveRequest?.body.apiKey, 'sonarr-secret-fixture');
  await context.close();
});

test('Radarr test includes optional UI credentials and Clear removes its connection', async t => {
  const { page, context, requests } = await openSettings(t);
  await page.getByRole('button', { name: 'Change Radarr settings' }).click();
  const dialog = page.getByRole('dialog', { name: 'Configure Radarr' });
  await dialog.getByLabel('Radarr URL').fill('https://radarr-new.example.test');
  await dialog.getByLabel('Radarr API key').fill('radarr-secret-fixture');
  await dialog.getByLabel('Radarr UI username (optional)').fill('backup-user');
  await dialog.getByLabel('Radarr UI password (optional)').fill('backup-password');
  await dialog.getByRole('button', { name: 'Test connection' }).click();
  await page.getByText('Radarr connection succeeded (version 5.0.0).').waitFor();
  const testRequest = requests.find(item => item.path === '/settings/radarr' && item.method === 'POST');
  assert.deepEqual(testRequest?.body, {
    url: 'https://radarr-new.example.test', apiKey: 'radarr-secret-fixture',
    uiUsername: 'backup-user', uiPassword: 'backup-password',
  });

  await dialog.getByRole('button', { name: 'Clear connection' }).click();
  await page.locator('#settings-radarr-saved').waitFor({ state: 'hidden' });
  assert.equal(await page.locator('#settings-configure-radarr').textContent(), 'Configure Radarr');
  assert.ok(requests.some(item => item.path === '/settings/radarr' && item.method === 'DELETE'));
  await context.close();
});
