import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from 'playwright';

const root = new URL('../../', import.meta.url);
const read = path => readFile(new URL(path, root));
const readText = path => readFile(new URL(path, root), 'utf8');
const [serverSource, svg, ico, htmlPages, healthHtml] = await Promise.all([
  readText('rust-arr/src/main.rs'),
  readText('rust-arr/src/assets/rawrarr-favicon.svg'),
  read('rust-arr/src/assets/rawrarr-favicon.ico'),
  Promise.all([
    'ui.html', 'auth.html', 'age_setup.html', 'landing.html', 'coordinator_health.html',
    'coordinator_workspace.html', 'coordinator_passkey.html', 'coordinator_billing.html',
    'subscriber_onboarding.html',
  ].map(path => readText(`rust-arr/src/${path}`))),
  readText('rust-arr/src/main.rs').then(source => source.match(/async fn health\(\)[\s\S]*?\n\}/)?.[0] ?? ''),
]);

test('all Rawrarr HTML entry pages advertise self-contained SVG and ICO favicons', () => {
  for (const [index, page] of htmlPages.entries()) {
    assert.match(page, /<link rel="icon" type="image\/svg\+xml" href="\/assets\/rawrarr-favicon\.svg">/, `page ${index} has the SVG favicon`);
    assert.match(page, /<link rel="icon" type="image\/x-icon" href="\/favicon\.ico">/, `page ${index} has the ICO fallback`);
  }
  assert.match(healthHtml, /<link rel="icon" type="image\/svg\+xml" href="\/assets\/rawrarr-favicon\.svg">/);
  assert.match(healthHtml, /<link rel="icon" type="image\/x-icon" href="\/favicon\.ico">/);

  assert.match(svg, /viewBox="0 0 64 64"/);
  assert.doesNotMatch(svg, /<(?:image|use)\b|\b(?:href|xlink:href)=/i, 'SVG favicon has no external image dependencies');
});

test('SVG and /favicon.ico routes serve their matching formats with embedded fallback assets', () => {
  assert.match(serverSource, /static_asset\(\s*"image\/svg\+xml",\s*ui_assets::text\("assets\/rawrarr-favicon\.svg",\s*include_str!\("assets\/rawrarr-favicon\.svg"\)\)/);
  assert.match(serverSource, /static_asset\(\s*"image\/x-icon",\s*ui_assets::bytes\("assets\/rawrarr-favicon\.ico",\s*include_bytes!\("assets\/rawrarr-favicon\.ico"\)\)/);
  assert.match(serverSource, /\.route\("\/assets\/rawrarr-favicon\.svg", get\(rawrarr_favicon\)\)/);
  assert.match(serverSource, /\.route\("\/favicon\.ico", get\(rawrarr_favicon_ico\)\)/);

  assert.equal(ico.readUInt16LE(0), 0);
  assert.equal(ico.readUInt16LE(2), 1);
  assert.ok(ico.readUInt16LE(4) >= 1, 'ICO contains at least one size');
});

test('Chromium renders both favicon assets without loading external image resources', async t => {
  let server;
  let browser;
  t.after(async () => {
    await browser?.close();
    await new Promise(resolve => server?.close(resolve));
  });

  server = createServer(async (request, response) => {
    const path = new URL(request.url, 'http://localhost').pathname;
    if (path === '/') {
      response.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
      response.end('<!doctype html><link rel="icon" type="image/svg+xml" href="/assets/rawrarr-favicon.svg"><link rel="icon" type="image/x-icon" href="/favicon.ico"><img id="svg" src="/assets/rawrarr-favicon.svg"><img id="ico" src="/favicon.ico">');
    } else if (path === '/assets/rawrarr-favicon.svg') {
      response.writeHead(200, { 'content-type': 'image/svg+xml' });
      response.end(svg);
    } else if (path === '/favicon.ico') {
      response.writeHead(200, { 'content-type': 'image/x-icon' });
      response.end(ico);
    } else {
      response.writeHead(404);
      response.end();
    }
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  const page = await browser.newPage();
  const unexpectedRequests = [];
  page.on('request', request => {
    if (!['/', '/assets/rawrarr-favicon.svg', '/favicon.ico'].includes(new URL(request.url()).pathname)) {
      unexpectedRequests.push(request.url());
    }
  });
  await page.goto(`http://127.0.0.1:${server.address().port}/`);
  await page.waitForFunction(() => [...document.images].every(image => image.complete && image.naturalWidth > 0));

  assert.deepEqual(await page.locator('img').evaluateAll(images => images.map(image => [image.naturalWidth, image.naturalHeight])), [[64, 64], [48, 48]]);
  assert.deepEqual(unexpectedRequests, [], 'favicon rendering does not request the wordmark or other external assets');
});
