import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const landing = await readFile(new URL('../../rust-arr/src/landing.html', import.meta.url), 'utf8');
const coordinator = await readFile(new URL('../../rust-arr/src/coordinator.rs', import.meta.url), 'utf8');

test('signed-out coordinator visitors see the public landing page and can continue to sign-in', async t => {
  assert.match(coordinator, /\.route\("\/", get\(coordinator_home\)\)/);
  assert.match(coordinator, /async fn coordinator_home\([\s\S]*?if crate::auth::is_authenticated\(&state\.coordinator_auth, &headers\)\.await \{[\s\S]*?ui_assets::text\("coordinator_workspace\.html", include_str!\("coordinator_workspace\.html"\)\)[\s\S]*?\} else \{\s*axum::response::Html\(crate::ui_assets::text\("landing\.html", include_str!\("landing\.html"\)\)/);
  assert.match(coordinator, /\.route\("\/auth", get\(coordinator_auth_page\)\)/);

  const server = createServer((request, response) => {
    if (request.url === '/') {
      response.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
      response.end(landing);
      return;
    }
    if (request.url === '/auth') {
      response.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
      response.end('<h1>Sign in</h1>');
      return;
    }
    if (request.url === '/assets/rawrarr-logo.png') {
      response.writeHead(200, { 'content-type': 'image/png' });
      response.end(Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/l8sAAAAASUVORK5CYII=', 'base64'));
      return;
    }
    response.writeHead(404);
    response.end();
  });

  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  t.after(() => new Promise(resolve => server.close(resolve)));
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  await page.goto(`http://127.0.0.1:${server.address().port}/`);

  await page.getByRole('heading', { name: 'Your library. Your devices.' }).waitFor();
  assert.equal(await page.locator('.brand').getByText('all the arrs', { exact: true }).innerText(), 'all the arrs');
  const brandNameBox = await page.locator('.brand-name').boundingBox();
  const taglineBox = await page.locator('.brand-tagline').boundingBox();
  assert.ok(brandNameBox && taglineBox && taglineBox.y > brandNameBox.y, 'tagline appears below the product name');
  assert.equal(await page.getByRole('link', { name: /Continue to sign in/ }).count(), 2);
  assert.equal(await page.getByRole('link', { name: 'Sign in', exact: true }).getAttribute('href'), '/auth');
  assert.match(await page.locator('.notice').innerText(), /under active development[\s\S]*independent security audit/i);
  assert.match(await page.locator('#privacy-title').innerText(), /Designed with boundaries/);
  assert.equal(await page.locator('script').count(), 0, 'public landing does not execute scripts');

  await page.setViewportSize({ width: 375, height: 812 });
  await page.waitForTimeout(100);
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth), 'mobile layout has no horizontal overflow');
  await page.getByRole('link', { name: /Continue to sign in/ }).first().click();
  await page.getByRole('heading', { name: 'Sign in' }).waitFor();
});
