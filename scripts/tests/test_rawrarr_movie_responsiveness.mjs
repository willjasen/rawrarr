import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const html = await readFile(new URL('../../rust-arr/src/ui.html', import.meta.url), 'utf8');

test('movies render while link checks are pending and survive a failed refresh', async t => {
  const browser = await chromium.launch({ executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const page = await browser.newPage();
  let releaseSettings;
  const settingsGate = new Promise(resolve => { releaseSettings = resolve; });
  let releaseRefresh;
  const refreshGate = new Promise(resolve => { releaseRefresh = resolve; });
  t.after(() => { releaseSettings(); releaseRefresh(); });
  let requests = 0;
  await page.route('http://rawrarr.test/**', async route => {
    const path = new URL(route.request().url()).pathname;
    if (path === '/') return route.fulfill({ body: html, contentType: 'text/html' });
    if (path === '/api/settings/radarr') {
      await settingsGate;
      return route.fulfill({ status: 503, json: {} });
    }
    if (path === '/api/radarr/movies') {
      requests++;
      if (requests > 1) {
        await refreshGate;
        return route.fulfill({ status: 502, json: { error: 'Radarr unavailable' } });
      }
      return route.fulfill({ json: [{ title: 'Example Movie', titleSlug: 'example', posterUrl: '/poster.svg', hasFile: true }] });
    }
    if (path === '/poster.svg') return route.fulfill({ contentType: 'image/svg+xml', body: '<svg xmlns="http://www.w3.org/2000/svg" width="48" height="72"/>' });
    return route.fulfill({ status: 503, json: {} });
  });
  await page.goto('http://rawrarr.test/#movies');
  // The settings response remains blocked: the previous implementation cannot render here.
  await page.locator('#radarr-movie-rows tr').waitFor({ timeout: 1500 });
  assert.equal(await page.locator('#refresh-movies').isEnabled(), true);
  await page.waitForFunction(() => document.querySelector('#radarr-movie-rows img')?.naturalWidth > 0);
  await page.locator('#refresh-movies').click();
  assert.match(await page.locator('#movies-message').textContent(), /Updating movies/);
  assert.match(await page.locator('#radarr-movie-rows').textContent(), /Example Movie/);
  releaseRefresh();
  await page.waitForFunction(() => !document.getElementById('refresh-movies').disabled);
  assert.match(await page.locator('#movies-message').textContent(), /Showing the previous list/);
  assert.equal(await page.locator('#radarr-movie-rows tr').count(), 1);
  releaseSettings();
});

test('reload shows server snapshot before slow live results and never lets a late snapshot overwrite them', async t => {
  const browser = await chromium.launch({ executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const page = await browser.newPage();
  let finishLive;
  let liveGate = new Promise(resolve => { finishLive = resolve; });
  let finishSnapshot;
  let snapshotGate = Promise.resolve();
  const snapshot = { movies: [{ title: 'Saved Movie' }], updatedAt: 1700000000 };
  await page.route('http://rawrarr.test/**', async route => {
    const path = new URL(route.request().url()).pathname;
    if (path === '/') return route.fulfill({ body: html, contentType: 'text/html' });
    if (path === '/api/radarr/movies/snapshot') {
      await snapshotGate;
      return route.fulfill({ json: snapshot });
    }
    if (path === '/api/radarr/movies') {
      await liveGate;
      return route.fulfill({ json: [{ title: 'Fresh Movie' }] });
    }
    return route.fulfill({ status: 503, json: {} });
  });
  await page.goto('http://rawrarr.test/#movies');
  await page.getByText('Saved Movie', { exact: true }).waitFor();
  assert.match(await page.locator('#movies-message').textContent(), /last updated.*Updating/);
  assert.equal(await page.locator('#refresh-movies').isEnabled(), false);
  finishLive();
  await page.getByText('Fresh Movie', { exact: true }).waitFor();
  snapshotGate = new Promise(resolve => { finishSnapshot = resolve; });
  await page.reload();
  await page.getByText('Fresh Movie', { exact: true }).waitFor();
  const response = page.waitForResponse('**/api/radarr/movies/snapshot');
  finishSnapshot();
  await response;
  await page.waitForFunction(() => document.getElementById('movies-message').textContent.includes('live data'));
  assert.equal(await page.getByText('Saved Movie', { exact: true }).count(), 0);
});
