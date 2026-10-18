import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';
const html = await readFile(new URL('../../rust-arr/src/ui.html', import.meta.url), 'utf8');
test('scrolling to a deferred poster measures its request rather than time since page render', async t => {
  const browser = await chromium.launch({ executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const page = await browser.newPage();
  const requested = new Set();
  await page.route('http://rawrarr.test/**', async route => {
    const path = new URL(route.request().url()).pathname;
    if (path === '/') return route.fulfill({ body: html, contentType: 'text/html' });
    if (path === '/api/radarr/movies') return route.fulfill({ json: Array.from({ length: 150 }, (_, i) => ({ title: `Movie ${i}`, posterUrl: `/poster/${i}`, artworkSource: 'shared' })) });
    if (path.startsWith('/poster/')) {
      requested.add(path);
      return route.fulfill({ body: '<svg xmlns="http://www.w3.org/2000/svg" width="48" height="72"><rect width="48" height="72" fill="red"/></svg>', contentType: 'image/svg+xml' });
    }
    return route.fulfill({ status: 503, json: {} });
  });
  await page.goto('http://rawrarr.test/#movies');
  const row = page.locator('#radarr-movie-rows tr').last();
  await row.waitFor();
  assert.equal(requested.has('/poster/149'), false);
  // Deliberately wait to reproduce the old timer's inclusion of user scroll delay.
  await page.waitForTimeout(1200);
  await row.scrollIntoViewIfNeeded();
  await page.waitForFunction(() => document.querySelector('#radarr-movie-rows tr:last-child td:nth-child(2)').textContent.endsWith(' ms'));
  const label = await row.locator('td').nth(1).textContent();
  assert.ok(parseFloat(label) < 1000, label);
  assert.equal(requested.has('/poster/149'), true);
});
