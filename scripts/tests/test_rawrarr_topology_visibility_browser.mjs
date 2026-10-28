import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import test from 'node:test';
import { readFile } from 'node:fs/promises';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const workspace = await readFile(new URL('../../rust-arr/src/coordinator_workspace.html', import.meta.url), 'utf8');
const workspaceScript = await readFile(new URL('../../rust-arr/src/coordinator-workspace.mjs', import.meta.url), 'utf8');
const routeScript = await readFile(new URL('../../rust-arr/src/coordinator-device-routes.mjs', import.meta.url), 'utf8');
const avatarScript = await readFile(new URL('../../rust-arr/src/coordinator-node-avatar.mjs', import.meta.url), 'utf8');
const colorScript = await readFile(new URL('../../rust-arr/src/profile-color.mjs', import.meta.url), 'utf8');
const aiScript = await readFile(new URL('../../rust-arr/src/coordinator-ai.mjs', import.meta.url), 'utf8');
const availabilityScript = await readFile(new URL('../../rust-arr/src/coordinator-availability.mjs', import.meta.url), 'utf8');
const sampleWorkspace = workspace.replace('<script type="module" src="/assets/coordinator-workspace.mjs"></script>', `<script type="module">
import { checkDeviceRoutes } from './coordinator-device-routes.mjs';
import { deriveNodeAvatarSeed } from './coordinator-node-avatar.mjs';
import { dinosaurMainColor, dinosaurWingWord } from './profile-color.mjs';
import { initAiChat } from './coordinator-ai.mjs';
import { createCoordinatorAvailabilityMonitor } from './coordinator-availability.mjs';
${workspaceScript.replace(/^import .*;\n/gm, '')}
</script>`);

async function serveWorkspace(devices) {
  let coordinatorOnline = true;
  let healthChecks = 0;
  const server = createServer((request, response) => {
    response.setHeader('content-type', 'application/json; charset=utf-8');
    if (request.url === '/health') {
      healthChecks += 1;
      response.statusCode = coordinatorOnline ? 200 : 503;
      response.end('health');
    } else if (request.url === '/' || request.url.startsWith('/#')) {
      response.setHeader('content-type', 'text/html; charset=utf-8');
      response.end(sampleWorkspace);
    } else if (request.url === '/coordinator-device-routes.mjs') {
      response.setHeader('content-type', 'text/javascript; charset=utf-8'); response.end(routeScript);
    } else if (request.url === '/coordinator-node-avatar.mjs') {
      response.setHeader('content-type', 'text/javascript; charset=utf-8'); response.end(avatarScript);
    } else if (request.url === '/profile-color.mjs') {
      response.setHeader('content-type', 'text/javascript; charset=utf-8'); response.end(colorScript);
    } else if (request.url === '/coordinator-ai.mjs') {
      response.setHeader('content-type', 'text/javascript; charset=utf-8'); response.end(aiScript);
    } else if (request.url === '/coordinator-availability.mjs') {
      response.setHeader('content-type', 'text/javascript; charset=utf-8'); response.end(availabilityScript);
    } else if (request.url === '/api/workspace/profile') {
      response.end(JSON.stringify({ displayName: 'Test User' }));
    } else if (request.url === '/api/workspace/nodes') {
      response.end(JSON.stringify(devices));
    } else if (request.url === '/api/workspace/pending') {
      response.end('[]');
    } else {
      response.statusCode = 404;
      response.end(JSON.stringify({ error: 'not found' }));
    }
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  server.setCoordinatorOnline = value => { coordinatorOnline = value; };
  server.healthChecks = () => healthChecks;
  return server;
}

async function waitForHealthChecks(server, count, timeoutMs = 12000) {
  const deadline = Date.now() + timeoutMs;
  while (server.healthChecks() < count) {
    if (Date.now() >= deadline) throw new Error(`expected ${count} coordinator health checks, observed ${server.healthChecks()}`);
    await new Promise(resolve => setTimeout(resolve, 25));
  }
}

for (const [name, devices, expectedVisible] of [
  ['no devices', [], false],
  ['only offline devices', [{ nodeId: 'offline-1', role: 'metadata', online: false, relayAvailable: false }], false],
  ['an online device', [{ nodeId: 'online-1', role: 'metadata', online: true, relayAvailable: false }], true],
]) {
  test(`Devices topology visibility: ${name}`, async t => {
    const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
    t.after(() => browser.close());
    const page = await browser.newPage();
    const server = await serveWorkspace(devices);
    t.after(() => new Promise(resolve => server.close(resolve)));
    await page.goto(`http://127.0.0.1:${server.address().port}/#devices`);
    await page.waitForFunction(() => document.querySelector('#status')?.textContent?.startsWith('No nodes yet.') || document.querySelector('#status')?.textContent?.startsWith('Choose a connected'));
    assert.equal(await page.locator('#device-topology').isVisible(), expectedVisible);
    assert.equal(await page.locator('#device-topology-content .topology-device').count(), expectedVisible ? devices.length : 0);
    if (name === 'only offline devices') {
      devices[0].online = true;
      await page.waitForTimeout(10000);
      assert.equal(await page.locator('#device-topology').isVisible(), true, 'the next node poll reveals topology after the device comes online');
    }
  });
}

test('an already-open workspace waits through transient loss and resumes after coordinator recovery', async t => {
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const page = await browser.newPage();
  const server = await serveWorkspace([]);
  t.after(() => new Promise(resolve => server.close(resolve)));
  await page.goto(`http://127.0.0.1:${server.address().port}/#fossils`);
  await page.waitForFunction(() => document.querySelector('#status')?.textContent?.startsWith('No nodes yet.'));
  await waitForHealthChecks(server, 1);
  const beforeOffline = server.healthChecks();
  server.setCoordinatorOnline(false);
  await waitForHealthChecks(server, beforeOffline + 1);
  assert.equal(await page.locator('#coordinator-pending-dialog').getAttribute('open'), null, 'one failed health check leaves the workspace visible');
  await waitForHealthChecks(server, beforeOffline + 2);
  await page.waitForFunction(() => document.querySelector('#coordinator-pending-dialog')?.open === true, { timeout: 7000 });
  assert.match(await page.locator('#coordinator-pending-title').textContent(), /Reconnecting to roarlay/);
  const beforeRecovery = server.healthChecks();
  server.setCoordinatorOnline(true);
  await waitForHealthChecks(server, beforeRecovery + 2);
  await page.waitForFunction(() => document.querySelector('#coordinator-pending-dialog')?.open === false, { timeout: 8000 });
  assert.equal(new URL(page.url()).hash, '#fossils', 'recovery preserves the current view route');
});

test('Roarlay page shows the coordinator health state separately from Fossils', async t => {
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const page = await browser.newPage();
  const server = await serveWorkspace([]);
  t.after(() => new Promise(resolve => server.close(resolve)));
  await page.goto(`http://127.0.0.1:${server.address().port}/#roarlay`);
  await page.waitForFunction(() => document.querySelector('#roarlay-health-label')?.textContent === 'Responding');
  assert.equal(new URL(page.url()).hash, '#roarlay');
  assert.equal(await page.locator('#roarlay-view').isVisible(), true);
  assert.equal(await page.locator('#fossils-view').isVisible(), false);
  assert.match(await page.locator('#roarlay-address').textContent(), /^http:\/\/127\.0\.0\.1:/);
  server.setCoordinatorOnline(false);
  await page.click('#roarlay-health-refresh');
  await page.waitForFunction(() => document.querySelector('#roarlay-health-label')?.textContent === 'Unavailable');
  assert.match(await page.locator('#roarlay-health-detail').textContent(), /HTTP 503/);
});
