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
const sampleWorkspace = workspace.replace('<script type="module" src="/assets/coordinator-workspace.mjs"></script>', `<script type="module">
import { checkDeviceRoutes } from './coordinator-device-routes.mjs';
import { deriveNodeAvatarSeed } from './coordinator-node-avatar.mjs';
import { dinosaurMainColor, dinosaurWingWord } from './profile-color.mjs';
import { initAiChat } from './coordinator-ai.mjs';
${workspaceScript.replace(/^import .*;\n/gm, '')}
</script>`);

async function serveWorkspace(devices) {
  const server = createServer((request, response) => {
    response.setHeader('content-type', 'application/json; charset=utf-8');
    if (request.url === '/' || request.url.startsWith('/#')) {
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
    } else if (request.url === '/api/workspace/profile') {
      response.end(JSON.stringify({ displayName: 'Test User' }));
    } else if (request.url === '/api/workspace/nodes') {
      response.end(JSON.stringify(devices));
    } else if (request.url === '/api/workspace/pending') {
      response.end('[]');
    } else if (request.url === '/api/workspace/ui-signature') {
      response.end(JSON.stringify({ status: 'unverified' }));
    } else {
      response.statusCode = 404;
      response.end(JSON.stringify({ error: 'not found' }));
    }
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  return server;
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
