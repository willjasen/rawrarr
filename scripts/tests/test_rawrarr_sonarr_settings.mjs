import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const html = await readFile(new URL('../../rust-arr/src/ui.html', import.meta.url), 'utf8');

test('Sonarr settings save the entered key, reload only its URL, and clear the connection', async t => {
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const page = await browser.newPage();
  const errors = [];
  const saves = [];
  let configured = false;

  page.on('pageerror', error => errors.push(error.message));
  await page.route('http://rawrarr.test/**', async route => {
    const request = route.request();
    const url = new URL(request.url());
    if (url.pathname === '/') return route.fulfill({ body: html, contentType: 'text/html' });
    if (url.pathname === '/api/settings/sonarr' && request.method() === 'GET') {
      return route.fulfill({ json: { configured, persisted: configured, url: configured ? 'https://sonarr.example.test' : null } });
    }
    if (url.pathname === '/api/settings/sonarr' && request.method() === 'PUT') {
      const body = request.postDataJSON();
      saves.push(body);
      configured = true;
      return route.fulfill({ json: { configured: true, persisted: true, url: body.url } });
    }
    if (url.pathname === '/api/settings/sonarr' && request.method() === 'DELETE') {
      configured = false;
      return route.fulfill({ json: { configured: false, persisted: false, url: null } });
    }
    return route.fulfill({ status: 503, json: { error: 'Fixture unavailable' } });
  });

  await page.goto('http://rawrarr.test/');
  await page.click('#settings-nav');
  await page.locator('#sonarr-settings-panel').scrollIntoViewIfNeeded();
  await page.locator('#sonarr-url').fill('https://sonarr.example.test');
  await page.locator('#sonarr-key').fill('sonarr-test-api-key');
  await page.click('#save-sonarr');
  await page.getByText('Connected to Sonarr. Settings saved encrypted.').waitFor();

  assert.deepEqual(saves, [{ url: 'https://sonarr.example.test', apiKey: 'sonarr-test-api-key' }]);
  assert.equal(await page.locator('#sonarr-key').inputValue(), '');
  assert.equal(await page.locator('#sonarr-url').inputValue(), 'https://sonarr.example.test');
  assert.equal(await page.locator('#sonarr-key').evaluate(element => element.required), false);

  await page.click('#clear-sonarr');
  await page.getByText('Sonarr settings cleared.').waitFor();
  assert.equal(await page.locator('#sonarr-url').inputValue(), '');
  assert.equal(await page.locator('#sonarr-key').evaluate(element => element.required), true);
  assert.deepEqual(errors, []);
});
