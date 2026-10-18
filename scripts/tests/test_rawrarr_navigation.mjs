import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const html = await readFile(new URL('../../rust-arr/src/ui.html', import.meta.url), 'utf8');

test('workspace navigation survives refresh and browser history', async t => {
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const page = await browser.newPage();
  let movieRequests = 0;
  await page.route('http://rawrarr.test/**', async route => {
    const path = new URL(route.request().url()).pathname;
    if (path === '/') return route.fulfill({ contentType: 'text/html', body: html });
    if (path === '/api/radarr/movies') {
      movieRequests += 1;
      return route.fulfill({ json: [] });
    }
    return route.fulfill({ status: 503, json: { error: 'Test integration unavailable' } });
  });
  async function expectView(title, id) {
    await page.waitForFunction(expected => document.getElementById('page-title').textContent === expected, title);
    assert.equal(await page.locator(`#${id}-content`).isVisible(), true);
    if (id !== 'overview') assert.equal(await page.locator(`#${id}-nav`).getAttribute('aria-current'), 'page');
  }
  await page.goto('http://rawrarr.test/');
  await expectView('Overview', 'overview');
  await page.locator('#movies-nav').click();
  await expectView('Movies', 'movies');
  assert.equal(await page.locator('#series-nav .nav-text').textContent(), 'TV');
  await page.locator('#series-nav').click();
  await expectView('TV', 'series');
  assert.equal(new URL(page.url()).hash, '#series');
  await page.reload();
  await expectView('TV', 'series');
  await page.locator('#movies-nav').click();
  await expectView('Movies', 'movies');
  assert.equal(new URL(page.url()).hash, '#movies');
  await page.reload();
  await expectView('Movies', 'movies');
  await page.waitForFunction(() => !document.getElementById('refresh-movies').disabled);
  const beforeRefresh = movieRequests;
  await page.locator('#refresh-movies').click();
  await page.waitForFunction(() => !document.getElementById('refresh-movies').disabled);
  assert.equal(movieRequests, beforeRefresh + 1);
  await expectView('Movies', 'movies');
  await page.locator('#configuration-nav').click();
  await expectView('Configuration', 'configuration');
  await page.reload();
  await expectView('Configuration', 'configuration');
  await page.goBack();
  await expectView('Movies', 'movies');
  await page.goForward();
  await expectView('Configuration', 'configuration');
  await page.locator('#settings-nav').click();
  await expectView('Overview', 'overview');
  assert.equal(await page.locator('#radarr-url').evaluate(el => el === document.activeElement), true);
  await page.goto('http://rawrarr.test/#movies');
  await expectView('Movies', 'movies');
  await page.goto('http://rawrarr.test/#unknown');
  await expectView('Overview', 'overview');
});
