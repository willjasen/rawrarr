import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const coordinator = await readFile(new URL('../../rust-arr/src/coordinator.rs', import.meta.url), 'utf8');

test('signed-out coordinator root requests redirect to sign-in', async t => {
  assert.match(coordinator, /\.route\("\/", get\(coordinator_home\)\)/);
  assert.match(coordinator, /async fn coordinator_home\([\s\S]*?if crate::auth::is_authenticated\(&state\.coordinator_auth, &headers\)\.await \{\s*crate::ui_assets::text\("coordinator_workspace\.html", include_str!\("coordinator_workspace\.html"\)\)\.await\s*\} else \{\s*return axum::response::Redirect::to\("\/auth"\)\.into_response\(\);\s*\};[\s\S]*?with_analytics_tracker\(page/);
  assert.match(coordinator, /\.route\("\/auth", get\(coordinator_auth_page\)\)/);

  const server = createServer((request, response) => {
    if (request.url === '/') {
      response.writeHead(303, { location: '/auth' });
      response.end();
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
  let rootResponse;
  page.on('response', response => {
    if (new URL(response.url()).pathname === '/') rootResponse = response;
  });
  await page.goto(`http://127.0.0.1:${server.address().port}/`);

  await page.getByRole('heading', { name: 'Sign in' }).waitFor();
  assert.equal(new URL(page.url()).pathname, '/auth', 'root visit lands on the sign-in route');
  assert.equal(rootResponse?.status(), 303);
  assert.equal(rootResponse?.headers().location, '/auth');
});
