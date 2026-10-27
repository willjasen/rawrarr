import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const html = await readFile(new URL('../../rust-arr/src/ui.html', import.meta.url), 'utf8');
test('movie titles and TMDb identifiers occupy separate aligned columns', async t => {
  const browser = await chromium.launch({ executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const page = await browser.newPage();
  await page.route('http://rawrarr.test/**', route => {
    const path = new URL(route.request().url()).pathname;
    if (path === '/') return route.fulfill({ body: html, contentType: 'text/html' });
    if (path === '/api/settings/radarr') return route.fulfill({ json: { configured: true, persisted: true, url: 'https://radarr.test/' } });
    if (path === '/api/radarr/movies') return route.fulfill({ json: [
      { title: 'A <Movie>', tmdbId: 42, year: 2020, path: '/media/example' },
      { title: 'No identifier', year: 2021 }
    ] });
    return route.fulfill({ status: 503, json: {} });
  });
  await page.goto('http://rawrarr.test/#movies');
  await page.waitForFunction(() => !document.getElementById('refresh-movies').disabled);
  const rows = page.locator('#radarr-movie-rows tr');
  assert.equal(await rows.count(), 2);
  const headers = await page.locator('table:has(#radarr-movie-rows) th').allTextContents();
  const idIndex = headers.indexOf('TMDb ID');
  assert.ok(idIndex > 0);
  for (const [index, title, id, year] of [[0, 'A <Movie> (2020)', '42', '2020'], [1, 'No identifier (2021)', '—', '2021']]) {
    const cells = rows.nth(index).locator('td');
    assert.equal(await cells.count(), headers.length);
    assert.equal(await cells.nth(0).textContent(), title);
    assert.equal(await cells.nth(idIndex).textContent(), id);
    assert.equal(await cells.nth(headers.indexOf('Year')).textContent(), year);
  }
  assert.ok(!headers.includes('Path'));
  assert.equal(await rows.locator('td.path').count(), 0);
  assert.ok(!(await rows.allTextContents()).some(text => text.includes('/media/example')));
});
