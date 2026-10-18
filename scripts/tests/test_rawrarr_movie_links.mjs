import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const html = await readFile(new URL('../../rust-arr/src/ui.html', import.meta.url), 'utf8');
const source = await readFile(new URL('../../rust-arr/src/main.rs', import.meta.url), 'utf8');

test('movie API preserves Radarr titleSlug for detail links', () => {
  assert.match(source, /title_slug:\s*movie[\s\S]{0,200}?\.get\("titleSlug"\)/);
});

for (const scenario of ['reachable', 'blocked', 'timeout', 'missing-slug']) {
  test(`movie links: ${scenario}`, async t => {
    const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
    t.after(() => browser.close());
    const context = await browser.newContext();
    const page = await context.newPage();
    const probes = [];
    await context.route('https://radarr.test/**', async route => {
      if (route.request().isNavigationRequest()) return route.fulfill({ body: 'Movie details' });
      probes.push(route.request());
      if (scenario === 'blocked') return route.abort('connectionrefused');
      if (scenario === 'timeout') return; // Let the actual browser AbortSignal expire.
      return route.fulfill({ body: 'Radarr', contentType: 'text/html' });
    });
    await page.route('http://rawrarr.test/**', async route => {
      const path = new URL(route.request().url()).pathname;
      if (path === '/') return route.fulfill({ body: html, contentType: 'text/html' });
      if (path === '/api/settings/radarr') return route.fulfill({ json: { url: 'https://radarr.test/radarr/' } });
      if (path === '/api/radarr/movies') return route.fulfill({ json: [{
        title: 'Test Movie', titleSlug: scenario === 'missing-slug' ? null : 'test movie/#1',
        posterUrl: '/poster.svg', tmdbId: 42
      }] });
      if (path === '/poster.svg') return route.fulfill({ contentType: 'image/svg+xml', body: '<svg xmlns="http://www.w3.org/2000/svg" width="48" height="72"><rect width="48" height="72" fill="blue"/></svg>' });
      return route.fulfill({ status: 503, json: { error: 'Fixture unavailable' } });
    });
    await page.goto('http://rawrarr.test/#movies');
    await page.waitForFunction(() => !document.getElementById('refresh-movies').disabled);
    const row = page.locator('#radarr-movie-rows tr');
    assert.equal(await row.count(), 1);
    if (scenario === 'reachable') await row.locator('a:not(.arr-external-link)').waitFor();
    await page.waitForFunction(() => performance.getEntriesByType('resource').some(entry => entry.name.includes('/api/settings/radarr')));
    // The probe is asynchronous; wait for its dispatch, not for movie rendering.
    await new Promise(resolve => setTimeout(resolve, 100));
    assert.equal(probes.length, 1);
    assert.equal(probes[0].url(), 'https://radarr.test/radarr/');
    assert.equal(probes[0].headers()['x-api-key'], undefined);
    assert.equal(probes[0].headers().cookie, undefined);
    if (scenario !== 'reachable') {
      assert.equal(await row.locator('a').count(), 0);
      assert.equal(await row.locator('.arr-external-link').count(), 0);
      assert.equal(await row.locator('img').count(), 1);
      assert.match(await row.textContent(), /Test Movie/);
      return;
    }
    const link = row.locator('a:not(.arr-external-link)');
    const expected = 'https://radarr.test/radarr/movie/test%20movie%2F%231';
    const yearLink = row.locator('.arr-external-link');
    assert.equal(await yearLink.getAttribute('href'), expected);
    assert.equal(await yearLink.locator('svg').count(), 1);
    assert.equal(await link.getAttribute('href'), expected);
    assert.equal(await link.getAttribute('rel'), 'noopener noreferrer');
    for (const child of ['img', 'span']) {
      const popupPromise = context.waitForEvent('page');
      await link.locator(child).click();
      const popup = await popupPromise;
      await popup.waitForLoadState();
      assert.equal(popup.url(), expected);
      assert.equal(await popup.evaluate(() => window.opener), null);
      await popup.close();
    }
    assert.equal(page.url(), 'http://rawrarr.test/#movies');
    await page.locator('#refresh-movies').click();
    await page.waitForFunction(() => !document.getElementById('refresh-movies').disabled);
    await row.locator('a:not(.arr-external-link)').waitFor();
    assert.equal(probes.length, 2);
  });
}
