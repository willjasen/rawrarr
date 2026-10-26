import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';
const html = await readFile(new URL('../../rust-arr/src/ui.html', import.meta.url), 'utf8');
test('one column schema controls both libraries, including reorder and provider semantics', async t => {
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const page = await browser.newPage();
  const errors = [];
  page.on('pageerror', e => errors.push(e.message));
  await page.route('http://rawrarr.test/**', route => {
    const path = new URL(route.request().url()).pathname;
    if (path === '/') return route.fulfill({ body: html, contentType: 'text/html' });
    if (path === '/api/sonarr/series') return route.fulfill({ json: [{ title: '<Series>', titleSlug: 'series-slug', tvdbId: 99, year: 2020, monitored: true, episodeFileCount: 3, episodeCount: 8, hasFile: true, sizeOnDisk: 2048, qualityProfile: 'HD' }] });
    if (path === '/api/radarr/movies') return route.fulfill({ json: [{ title: 'Movie', titleSlug: 'movie-slug', year: 2021, tmdbId: 42, hasFile: true }] });
    if (path === '/api/settings/sonarr') return route.fulfill({ json: { url: 'https://sonarr.test/' } });
    if (path === '/api/settings/radarr') return route.fulfill({ json: { url: 'https://radarr.test/' } });
    return route.fulfill({ status: 503, json: { error: 'Fixture unavailable' } });
  });
  await page.route('https://sonarr.test/**', route => route.fulfill({ body: 'Sonarr', contentType: 'text/html' }));
  await page.route('https://radarr.test/**', route => route.fulfill({ body: 'Radarr', contentType: 'text/html' }));
  await page.goto('http://rawrarr.test/#series');
  await page.waitForFunction(() => !document.getElementById('refresh-series').disabled);
  const cells = await page.locator('#sonarr-series-rows tr td').allTextContents();
  cells[4] = cells[4].trim();
  cells[4] = cells[4].trim();
  assert.deepEqual(cells, ['<Series> (2020)', '—', 'Unavailable', '99', '2020', '3 / 8', 'Yes', 'HD', '2 KB']);
  await page.evaluate(() => { libraryColumns.unshift(libraryColumns.pop()); renderArrLibrary([{title:'Series',sizeOnDisk:1024}], 0, null, libraryApps.sonarr); renderArrLibrary([{title:'Movie',sizeOnDisk:2048}], 0, null, libraryApps.radarr); });
  for (const id of ['sonarr-series-rows', 'radarr-movie-rows']) {
    assert.equal(await page.locator(`#${id}`).evaluate(el => el.closest('table').querySelector('th').textContent), 'Size');
  }
  assert.equal(await page.locator('#sonarr-series-rows td').first().textContent(), '1 KB');
  assert.equal(await page.locator('#radarr-movie-rows td').first().textContent(), '2 KB');
  await page.click('#movies-nav');
  await page.waitForFunction(() => !document.getElementById('refresh-movies').disabled);
  assert.equal(await page.locator('#series-content').isVisible(), false);
  assert.equal(await page.locator('#movies-content').isVisible(), true);
  assert.deepEqual(errors, []);
});
