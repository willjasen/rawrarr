import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const html = await readFile(new URL('../../rust-arr/src/ui.html', import.meta.url), 'utf8');

test('dashboard connects to SSE and applies snapshots and connection state', async t => {
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const page = await browser.newPage();
  await page.addInitScript(() => {
    window.eventSources = [];
    window.EventSource = class {
      constructor(url) { this.url = url; window.eventSources.push(this); }
      addEventListener(name, handler) { this.dashboardHandler = handler; }
      emitDashboard(data) { this.dashboardHandler({ data: JSON.stringify(data) }); }
      open() { this.onopen?.(); }
      error() { this.onerror?.(); }
    };
  });
  await page.route('http://rawrarr.test/**', route => route.fulfill({ status: 503, json: {} }));
  await page.route('http://rawrarr.test/', route => route.fulfill({ contentType: 'text/html', body: html }));
  await page.goto('http://rawrarr.test/');
  assert.equal(await page.evaluate(() => eventSources.length), 1);
  assert.equal(await page.evaluate(() => eventSources[0].url), '/api/events');

  await page.evaluate(() => eventSources[0].emitDashboard({
    radarr: { configured: true, connected: true, appName: 'Fixture Radarr', version: '1.0', urlBase: '/' },
    metadata: { configured: true },
    library: { configured: true, name: 'Fixture Library' },
    devices: [],
  }));
  await page.waitForFunction(() => document.getElementById('live-status').textContent === 'Live updates connected');
  assert.equal(await page.locator('#radarr-state').textContent(), 'Connected');
  assert.match(await page.locator('#library-empty-title').textContent(), /Fixture Library/);

  await page.evaluate(() => eventSources[0].error());
  assert.equal(await page.locator('#live-status').textContent(), 'Reconnecting to live updates…');
  await page.evaluate(() => eventSources[0].open());
  assert.equal(await page.locator('#live-status').textContent(), 'Live updates connected');
});
