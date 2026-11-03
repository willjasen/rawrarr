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

test('Fossils fits desktop and narrow mobile viewports with long labels and reachable actions', async t => {
  const devices = [
    { nodeId: 'device-alpha', role: 'metadata', online: true, relayAvailable: false, displayName: 'Brandon workstation with an exceptionally long display name' },
    { nodeId: 'device-beta', role: 'storage', online: true, relayAvailable: false, displayName: 'Living room media and backup storage node' },
    { nodeId: 'device-gamma', role: 'metadata', online: true, relayAvailable: false, displayName: 'Travel laptop with a deliberately long device label' },
  ];
  const server = await serveWorkspace(devices);
  t.after(() => new Promise(resolve => server.close(resolve)));
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());

  for (const viewport of [
    { name: 'desktop', width: 1280, height: 800 },
    { name: 'narrow mobile', width: 375, height: 812 },
  ]) {
    const page = await browser.newPage({ viewport: { width: viewport.width, height: viewport.height } });
    page.setDefaultTimeout(8000);
    page.setDefaultNavigationTimeout(8000);
    await page.goto(`http://127.0.0.1:${server.address().port}/#fossils`);
    await page.waitForFunction(() => document.querySelectorAll('#nodes .device-card').length === 3, null, { timeout: 10000 });
    await page.waitForFunction(() => document.querySelectorAll('#device-topology-content .topology-device').length === 3, null, { timeout: 10000 });

    const layout = await page.evaluate(() => {
      const rect = element => {
        const { x, y, right, bottom, width, height } = element.getBoundingClientRect();
        return { x, y, right, bottom, width, height };
      };
      return {
        viewportWidth: window.innerWidth,
        documentWidth: document.documentElement.scrollWidth,
        heading: rect(document.querySelector('#fossils-view .devices-page-heading')),
        title: rect(document.querySelector('#fossils-view .devices-page-heading .page-title')),
        addAction: rect(document.querySelector('#fossils-view .devices-page-heading .button-link')),
        cards: [...document.querySelectorAll('#nodes .device-card')].map(rect),
        statuses: [...document.querySelectorAll('#nodes .device-card .node-state')].map(rect),
        graph: rect(document.querySelector('.topology-graph')),
        topologyDevices: [...document.querySelectorAll('.topology-graph .topology-device')].map(rect),
      };
    });

    assert.ok(layout.documentWidth <= layout.viewportWidth, `${viewport.name}: page should not overflow horizontally (${layout.documentWidth}px > ${layout.viewportWidth}px)`);
    assert.ok(layout.title.x >= layout.heading.x && layout.title.right <= layout.heading.right, `${viewport.name}: Fossils heading should stay inside its layout`);
    assert.ok(layout.addAction.x >= layout.heading.x && layout.addAction.right <= layout.heading.right, `${viewport.name}: Add a fossil action should stay inside its layout`);
    if (viewport.name === 'desktop') {
      assert.ok(layout.cards[0].width < layout.viewportWidth / 2, 'desktop: device cards should retain a multi-column layout');
    }
    for (const [index, card] of layout.cards.entries()) {
      assert.ok(card.x >= 0 && card.right <= layout.viewportWidth, `${viewport.name}: device card ${index + 1} should fit the viewport`);
      const status = layout.statuses[index];
      assert.ok(status.x >= card.x && status.right <= card.right, `${viewport.name}: connection status ${index + 1} should remain within its card`);
    }
    for (const [index, node] of layout.topologyDevices.entries()) {
      assert.ok(node.x >= layout.graph.x && node.right <= layout.graph.right, `${viewport.name}: topology node ${index + 1} should fit inside the graph`);
    }

    const edit = page.getByRole('button', { name: 'Edit Brandon workstation with an exceptionally long display name' });
    const editBounds = await edit.boundingBox();
    assert.ok(editBounds && editBounds.x >= 0 && editBounds.x + editBounds.width <= viewport.width && editBounds.y >= 0 && editBounds.y + editBounds.height <= viewport.height, `${viewport.name}: Edit action should be visible in the viewport`);
    await edit.click();
    await page.getByRole('dialog', { name: 'Edit device' }).waitFor({ state: 'visible' });
    const deleteButton = page.getByRole('button', { name: 'Delete device' });
    assert.equal(await deleteButton.isVisible(), true, `${viewport.name}: Delete action should be reachable from Edit`);
    assert.equal(await deleteButton.isEnabled(), true, `${viewport.name}: Delete action should be enabled`);
    const deleteBounds = await deleteButton.boundingBox();
    assert.ok(deleteBounds && deleteBounds.x >= 0 && deleteBounds.x + deleteBounds.width <= viewport.width && deleteBounds.y >= 0 && deleteBounds.y + deleteBounds.height <= viewport.height, `${viewport.name}: Delete action should fit within the viewport`);
    await page.locator('#device-edit-cancel').click();
    await page.close();
  }
});

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

test('Signing key topology fits desktop and mobile with long device labels', async t => {
  const devices = [
    { nodeId: 'device-alpha', role: 'metadata', online: true, relayAvailable: false, displayName: 'Brandon workstation with an exceptionally long display name', signingFingerprint: 'a'.repeat(64), signingRootFingerprint: 'b'.repeat(64) },
    { nodeId: 'device-beta', role: 'storage', online: true, relayAvailable: false, displayName: 'Living room media and backup storage node', signingFingerprint: 'c'.repeat(64), signingRootFingerprint: 'b'.repeat(64) },
  ];
  const server = await serveWorkspace(devices);
  t.after(() => new Promise(resolve => server.close(resolve)));
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  for (const viewport of [{ name: 'desktop', width: 1280, height: 800 }, { name: 'mobile', width: 375, height: 812 }]) {
    const page = await browser.newPage({ viewport: { width: viewport.width, height: viewport.height } });
    await page.goto(`http://127.0.0.1:${server.address().port}/#fossils`);
    await page.waitForSelector('.signing-device-card');
    const layout = await page.evaluate(() => ({
      width: document.documentElement.scrollWidth,
      viewport: window.innerWidth,
      root: document.querySelector('.signing-root-card').getBoundingClientRect().toJSON(),
      cards: [...document.querySelectorAll('.signing-device-card')].map(card => card.getBoundingClientRect().toJSON()),
      title: document.querySelector('#signing-topology-title').getBoundingClientRect().toJSON(),
    }));
    assert.ok(layout.width <= layout.viewport, `${viewport.name} has no horizontal overflow`);
    assert.ok(layout.root.x >= 0 && layout.root.right <= layout.viewport, `${viewport.name} root card fits`);
    assert.ok(layout.title.width > 0 && layout.title.bottom <= viewport.height, `${viewport.name} section title is visible`);
    assert.equal(layout.cards.length, 2);
    assert.ok(layout.cards.every(card => card.x >= 0 && card.right <= layout.viewport), `${viewport.name} device cards fit`);
    assert.match(await page.locator('.signing-device-card').first().innerText(), /root-authorized signing key/);
    await page.close();
  }
});
