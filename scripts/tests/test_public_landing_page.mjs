import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const root = new URL('../../website/', import.meta.url);
const html = await readFile(new URL('index.html', root), 'utf8');
const css = await readFile(new URL('styles.css', root), 'utf8');
const logo = await readFile(new URL('assets/rawrarr-logo.png', root));
const netlify = await readFile(new URL('../../netlify.toml', import.meta.url), 'utf8');

test('public rawrarr.net landing page is static, accurate, and responsive', async t => {
  assert.match(netlify, /^\[build\][\s\S]*?publish\s*=\s*"website"/m);
  assert.match(netlify, /Content-Security-Policy\s*=\s*"default-src 'self'/);
  assert.match(netlify, /X-Content-Type-Options\s*=\s*"nosniff"/);
  assert.match(html, /<link rel="canonical" href="https:\/\/rawrarr\.net\/">/);
  assert.match(html, /The coordinator can see account, device, routing, timing, and traffic-size information/);
  assert.match(html, /has not had an independent security audit/i);
  assert.doesNotMatch(html, /href="\/auth"|<script\b/i, 'public page has no private sign-in link or executable script');
  assert.doesNotMatch(css, /@import\s/i, 'styles load only from the site itself');

  const server = createServer(async (request, response) => {
    const path = request.url?.split('?')[0];
    if (path === '/' || path === '/index.html') {
      response.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
      response.end(html);
      return;
    }
    if (path === '/styles.css') {
      response.writeHead(200, { 'content-type': 'text/css; charset=utf-8' });
      response.end(css);
      return;
    }
    if (path === '/assets/rawrarr-logo.png') {
      response.writeHead(200, { 'content-type': 'image/png' });
      response.end(logo);
      return;
    }
    response.writeHead(404);
    response.end();
  });

  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  t.after(() => new Promise(resolve => server.close(resolve)));
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  const origin = `http://127.0.0.1:${server.address().port}`;
  const offsiteRequests = [];
  page.on('request', request => {
    if (!request.url().startsWith(origin)) offsiteRequests.push(request.url());
  });
  await page.goto(origin);

  await page.getByRole('heading', { name: 'Your library. Your devices.' }).waitFor();
  await page.getByRole('heading', { name: 'Your media stack, in one place.' }).waitFor();
  await page.getByRole('heading', { name: 'Useful boundaries. Honest limits.' }).waitFor();
  assert.equal(await page.getByRole('link', { name: 'See how Rawrarr fits' }).count(), 1);
  assert.equal(await page.locator('script').count(), 0);
  assert.equal(await page.locator('img').first().evaluate(image => image.complete && image.naturalWidth > 0), true);
  assert.equal(offsiteRequests.length, 0, `all assets load locally: ${offsiteRequests.join(', ')}`);

  for (const viewport of [{ width: 1280, height: 800 }, { width: 375, height: 812 }]) {
    await page.setViewportSize(viewport);
    await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
    assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth), `${viewport.width}px layout has no horizontal overflow`);
    const action = await page.getByRole('link', { name: 'See how Rawrarr fits' }).boundingBox();
    assert.ok(action, `${viewport.width}px primary action is rendered`);
    assert.ok(action.width >= 44 && action.height >= 44, `${viewport.width}px primary action has a usable hit area`);
    assert.ok(action.x >= 0 && action.y >= 0 && action.x + action.width <= viewport.width && action.y + action.height <= viewport.height, `${viewport.width}px primary action is in the initial viewport`);
  }
});
