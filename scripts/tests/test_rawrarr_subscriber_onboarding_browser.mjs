import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const html = await readFile(new URL('../../rust-arr/src/subscriber_onboarding.html', import.meta.url));
const script = await readFile(new URL('../../rust-arr/src/assets/subscriber-onboarding.js', import.meta.url));

test('first-account subscriber setup gates node choices behind age and root backups', async t => {
  const requests = [];
  const server = createServer((request, response) => {
    requests.push(request.url);
    if (request.url === '/setup/node') { response.setHeader('content-type', 'text/html'); response.end(html); }
    else if (request.url === '/assets/subscriber-onboarding.js') { response.setHeader('content-type', 'text/javascript'); response.end(script); }
    else { response.statusCode = 404; response.end(); }
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  t.after(() => new Promise(resolve => server.close(resolve)));
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const context = await browser.newContext();
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  const page = await context.newPage();
  await page.goto(`http://127.0.0.1:${server.address().port}/setup/node`);
  assert.equal(await page.locator('#root-step').isVisible(), false);
  assert.equal(await page.locator('#deploy-step').isVisible(), false);
  await page.getByRole('button', { name: 'Create age ID' }).click();
  await page.locator('#age-recipient').getByText(/Copy and save/).waitFor();
  assert.equal(await page.locator('#root-step').isVisible(), false);
  await page.getByRole('button', { name: 'Copy new private ID' }).click();
  const identity = await page.evaluate(() => navigator.clipboard.readText());
  assert.match(identity, /^AGE-SECRET-KEY-PQ-1/);
  assert.equal((await page.locator('body').innerText()).includes(identity), false);
  await page.locator('#age-backed-up').check();
  await page.locator('#root-step').waitFor({ state: 'visible' });
  await page.getByRole('button', { name: 'Create encrypted root backup' }).click();
  await page.locator('#root-fingerprint').getByText(/Root fingerprint:/).waitFor();
  assert.equal(await page.locator('#deploy-step').isVisible(), false);
  await page.getByRole('button', { name: 'Copy encrypted backup' }).click();
  const backup = await page.evaluate(() => navigator.clipboard.readText());
  assert.match(backup, /^-----BEGIN AGE ENCRYPTED FILE-----/);
  assert.equal((await page.locator('body').innerText()).includes(backup), false);
  await page.locator('#root-backed-up').check();
  await page.locator('#deploy-step').waitFor({ state: 'visible' });
  assert.deepEqual(await page.locator('#node-role option').allTextContents(), ['Metadata', 'Apex (storage)']);
  assert.deepEqual(requests.filter(path => path !== '/favicon.ico'), ['/setup/node', '/assets/subscriber-onboarding.js'], 'private material never leaves the browser in the setup flow');
  await context.close();
});
