import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const pageHtml = await readFile(new URL('../../rust-arr/src/age_setup.html', import.meta.url), 'utf8');
const privateKey = 'AGE-SECRET-KEY-PQ-1ABCDXYZ0123456789';
const recipient = 'age1pq1browser-test-recipient';
const themeSeed = '0123456789abcdef'.repeat(4);
let unlockRequests = 0;

const server = createServer((request, response) => {
  if (request.method === 'GET' && (request.url === '/age-setup' || request.url === '/')) {
    response.setHeader('content-type', 'text/html; charset=utf-8');
    response.end(pageHtml);
    return;
  }
  response.setHeader('content-type', 'application/json; charset=utf-8');
  if (request.url === '/api/settings/age' && request.method === 'GET') {
    response.end(JSON.stringify({ unlocked: false, encryptedConfigExists: false }));
  } else if (request.url === '/api/settings/age/generate' && request.method === 'POST') {
    response.end(JSON.stringify({ identity: `# public key: ${recipient}\n${privateKey}`, recipient, themeSeed }));
  } else if (request.url === '/api/settings/age' && request.method === 'POST') {
    unlockRequests += 1;
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

  await page.getByRole('button', { name: 'Create ID' }).click();
  await page.getByRole('dialog').waitFor({ state: 'visible' });
  assert.equal(await page.locator('#private-preview').textContent(), 'AGE-SECRET-K…56789');
  assert.equal(await page.locator('body').textContent().then(text => text.includes(privateKey)), false, 'the complete private key is not displayed');
  const continueButton = page.getByRole('button', { name: 'Continue' });
  assert.equal(await continueButton.isDisabled(), true, 'Continue waits for the identity copy and backup acknowledgement');

  await page.getByRole('button', { name: 'Copy private identity' }).click();
  const backupCheckbox = page.locator('#backup-confirmed');
  await backupCheckbox.check();
  assert.equal(await backupCheckbox.isChecked(), true, 'backup acknowledgement is checked');
  assert.equal(await continueButton.isEnabled(), true, 'the backup acknowledgement enables Continue');
  assert.equal(await page.evaluate(() => navigator.clipboard.readText()), privateKey, 'copy action places only the private key on the clipboard');
  await continueButton.click();
  await page.waitForURL(url => url.pathname === '/', { timeout: 5000 });
  assert.equal(unlockRequests, 1, 'Continue submits the identity after backup acknowledgement');
  await context.close();
});
