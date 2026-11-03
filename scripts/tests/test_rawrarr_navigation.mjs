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
    if (path === '/api/settings/radarr') return route.fulfill({ json: { configured: true, persisted: true, url: 'https://radarr.test/' } });
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
  await page.locator('#edit-radarr-connection').click();
  await page.locator('#radarr-settings-panel').waitFor({ state: 'visible' });
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
  await expectView('Configuration', 'configuration');
  await page.goto('http://rawrarr.test/#movies');
  await expectView('Movies', 'movies');
  await page.goto('http://rawrarr.test/#unknown');
  await expectView('Overview', 'overview');
});

test('mobile navigation opens from the bottom-right menu and remains usable at desktop widths', async t => {
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const page = await browser.newPage({ viewport: { width: 375, height: 812 } });
  await page.route('http://rawrarr.test/**', async route => {
    const path = new URL(route.request().url()).pathname;
    if (path === '/') return route.fulfill({ contentType: 'text/html', body: html });
    if (path === '/api/settings/radarr') return route.fulfill({ json: { configured: true, persisted: true, url: 'https://radarr.test/' } });
    if (path === '/api/radarr/movies') return route.fulfill({ json: [] });
    return route.fulfill({ status: 503, json: { error: 'Test integration unavailable' } });
  });

  const menuToggle = page.getByRole('button', { name: 'Open navigation menu' });
  const navigation = page.locator('#primary-navigation');
  await page.goto('http://rawrarr.test/');
  await page.waitForFunction(() => document.getElementById('page-title').textContent === 'Overview');
  assert.equal(await menuToggle.isVisible(), true);
  assert.equal(await page.evaluate(() => document.documentElement.scrollWidth === window.innerWidth), true);
  const menuBox = await menuToggle.boundingBox();
  assert.ok(menuBox);
  assert.ok(menuBox.x > 375 - 100);
  assert.ok(menuBox.y > 812 - 100);
  assert.equal(await navigation.isVisible(), false);
  assert.equal(await navigation.getAttribute('aria-hidden'), 'true');

  await menuToggle.click();
  assert.equal(await page.getByRole('navigation', { name: 'Main navigation' }).isVisible(), true);
  assert.deepEqual(await navigation.locator('.nav-text').allTextContents(), [
    'Overview', 'Movies', 'TV', 'Configuration', 'Settings', 'About',
  ]);
  assert.equal(await page.getByRole('button', { name: 'Close navigation menu' }).getAttribute('aria-expanded'), 'true');
  await page.locator('#movies-nav').click();
  await page.locator('#movies-content').waitFor({ state: 'visible' });
  assert.equal(await page.locator('#movies-nav').getAttribute('aria-current'), 'page');
  assert.equal(await navigation.isVisible(), false);

  await page.getByRole('button', { name: 'Open navigation menu' }).click();
  await page.keyboard.press('Escape');
  assert.equal(await navigation.isVisible(), false);
  assert.equal(await page.evaluate(() => document.activeElement.id), 'mobile-nav-toggle');

  await page.getByRole('button', { name: 'Open navigation menu' }).click();
  await page.locator('#page-title').click();
  assert.equal(await navigation.isVisible(), false);

  await page.getByRole('button', { name: 'Open navigation menu' }).click();
  await page.locator('#about-nav').click();
  await page.locator('#about-content').waitFor({ state: 'visible' });
  assert.equal(new URL(page.url()).hash, '#about');
  assert.equal(await page.locator('#about-nav').getAttribute('aria-current'), 'page');

  await page.setViewportSize({ width: 1280, height: 800 });
  assert.equal(await page.locator('#mobile-nav-toggle').isVisible(), false);
  assert.equal(await page.locator('#about-nav').isVisible(), true);
  assert.equal(await page.locator('#primary-navigation').getAttribute('aria-hidden'), 'false');
  assert.equal(await page.evaluate(() => document.documentElement.scrollWidth === window.innerWidth), true);
});
