import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const pageHtml = await readFile(new URL('../../rust-arr/src/age_setup.html', import.meta.url), 'utf8');
const metadataExport = "window.rawrarrMiniExport = { prepareGroupIdentityForDevice: async () => false, encryptIdentityForDevice: async value => `age-ciphertext:${value}` };";
const privateKey = 'AGE-SECRET-KEY-PQ-1ABCDXYZ0123456789';
const recipient = 'age1pq1browser-test-recipient';
const themeSeed = '0123456789abcdef'.repeat(4);
let unlockRequests = 0;
let generateRequests = 0;
const unlockBodies = [];

const server = createServer((request, response) => {
  if (request.method === 'GET' && (request.url === '/age-setup' || request.url === '/')) {
    response.setHeader('content-type', 'text/html; charset=utf-8');
    response.end(pageHtml);
    return;
  }
  if (request.method === 'GET' && request.url === '/assets/metadata-export.js') {
    response.setHeader('content-type', 'text/javascript; charset=utf-8');
    response.end(metadataExport);
    return;
  }
  response.setHeader('content-type', 'application/json; charset=utf-8');
  if (request.url === '/api/settings/age' && request.method === 'GET') {
    response.end(JSON.stringify({ unlocked: false, encryptedConfigExists: false }));
  } else if (request.url === '/api/settings/age/generate' && request.method === 'POST') {
    generateRequests += 1;
    response.end(JSON.stringify({ identity: `# public key: ${recipient}\n${privateKey}`, recipient, themeSeed }));
  } else if (request.url === '/api/settings/age' && request.method === 'POST') {
    unlockRequests += 1;
    let body = '';
    request.on('data', chunk => { body += chunk; });
    request.on('end', () => { unlockBodies.push(JSON.parse(body)); });
    response.end(JSON.stringify({ unlocked: true, themeSeed }));
  } else {
    response.statusCode = 404;
    response.end(JSON.stringify({ error: 'not found' }));
  }
});

let browser;
test('age identity creation and backup acknowledgement work in a real browser', async t => {
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  t.after(() => new Promise(resolve => server.close(resolve)));
  browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const context = await browser.newContext();
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  const page = await context.newPage();
  const address = server.address();
  await page.goto(`http://127.0.0.1:${address.port}/age-setup`);
  for (const viewport of [{ width: 1280, height: 800 }, { width: 375, height: 812 }]) {
    await page.setViewportSize(viewport);
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true, 'setup privacy text fits the viewport');
    for (const selector of ['.privacy-note', '#remember-on-device', '#generate', '#import-id']) {
      await page.locator(selector).scrollIntoViewIfNeeded();
      const rect = await page.locator(selector).boundingBox();
      assert.ok(rect && rect.width > 0 && rect.x >= 0 && rect.x + rect.width <= viewport.width, `${selector} fits at ${viewport.width}px`);
    }
  }

  await page.getByRole('button', { name: 'Create ID' }).click();
  await page.getByRole('dialog').waitFor({ state: 'visible' });
  assert.equal(generateRequests, 1, 'Create ID calls the generation endpoint');
  assert.equal(await page.locator('#private-preview').textContent(), 'AGE-SECRET-K…56789');
  assert.equal(await page.locator('body').textContent().then(text => text.includes(privateKey)), false, 'the complete private key is not displayed');
  const copyButton = page.getByRole('button', { name: 'Copy private identity' });
  assert.equal(await copyButton.isEnabled(), true, 'the generated ID can be copied');
  const backupCheckbox = page.locator('#backup-confirmed');
  assert.equal(await backupCheckbox.isVisible(), false, 'the acknowledgement checkbox appears after copying');
  const continueButton = page.locator('#unlock');
  assert.equal(await continueButton.textContent(), 'Continue');
  assert.equal(await continueButton.isDisabled(), true, 'Continue waits for the identity copy and backup acknowledgement');

  await copyButton.click();
  await backupCheckbox.waitFor({ state: 'visible' });
  assert.equal(await backupCheckbox.isVisible(), true, 'copying the ID reveals the acknowledgement checkbox');
  assert.equal(await continueButton.isDisabled(), true, 'copying alone does not enable Continue');
  await backupCheckbox.check();
  assert.equal(await backupCheckbox.isChecked(), true, 'backup acknowledgement is checked');
  assert.equal(await continueButton.isEnabled(), true, 'the backup acknowledgement enables Continue');
  assert.equal(await page.evaluate(() => navigator.clipboard.readText()), privateKey, 'copy action places only the private key on the clipboard');
  await continueButton.click();
  await page.waitForURL(url => url.pathname === '/', { timeout: 5000 });
  assert.equal(unlockRequests, 1, 'Continue submits the identity after backup acknowledgement');
  assert.equal(await page.evaluate(() => localStorage.getItem('rawrarr.browser.age-id')), `# public key: ${recipient}\n${privateKey}`);
  assert.equal(unlockBodies[0].rememberOnDevice, false, 'browser persistence does not require the separate node-local opt-in');
  await page.reload();
  assert.equal(await page.evaluate(() => localStorage.getItem('rawrarr.browser.age-id')), `# public key: ${recipient}\n${privateKey}`, 'setup ID survives reload');

  await page.goto(`http://127.0.0.1:${address.port}/age-setup`);
  await page.getByRole('button', { name: 'Import ID', exact: true }).click();
  await page.locator('#identity').fill(privateKey);
  await page.locator('#import-submit').click();
  await page.waitForURL(url => url.pathname === '/', { timeout: 5000 });
  assert.equal(await page.evaluate(() => localStorage.getItem('rawrarr.browser.age-id')), privateKey);
  assert.equal(unlockBodies[1].rememberOnDevice, false);
  await context.close();
});
