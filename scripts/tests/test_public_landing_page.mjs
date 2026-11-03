import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const root = new URL('../../website/', import.meta.url);
const html = await readFile(new URL('index.html', root), 'utf8');
const analytics = await readFile(new URL('analytics.js', root), 'utf8');
const css = await readFile(new URL('styles.css', root), 'utf8');
const logo = await readFile(new URL('assets/rawrarr-logo.png', root));
const netlify = await readFile(new URL('../../netlify.toml', import.meta.url), 'utf8');
const contentSecurityPolicy = netlify.match(/Content-Security-Policy\s*=\s*"([^"]+)"/)?.[1];

test('public rawrarr.net landing page is static, accurate, and responsive', async t => {
  assert.match(netlify, /^\[build\][\s\S]*?publish\s*=\s*"website"/m);
  assert.match(netlify, /Content-Security-Policy\s*=\s*"default-src 'self';[\s\S]*?script-src 'self';[\s\S]*?connect-src 'self'"/);
  assert.match(netlify, /X-Content-Type-Options\s*=\s*"nosniff"/);
  assert.match(html, /<link rel="canonical" href="https:\/\/rawrarr\.net\/">/);
  assert.match(html, /<meta name="color-scheme" content="dark light">/);
  assert.match(css, /:root\s*\{\s*color-scheme:\s*dark;/);
  assert.match(css, /@media \(prefers-color-scheme: light\)/);
  assert.match(html, /The coordinator can see account, device, routing, timing, and traffic-size information/);
  assert.match(html, /has not had an independent security audit/i);
  assert.match(html, /Anonymous site analytics/);
  assert.doesNotMatch(html, /href="\/auth"/i, 'public page has no private sign-in link');
  assert.doesNotMatch(html, /<script(?![^>]*\bsrc=)[^>]*>/i, 'public page has no inline scripts');
  assert.match(analytics, /website, url: '\/website', title: 'Rawrarr website'/);
  assert.doesNotMatch(css, /@import\s/i, 'styles load only from the site itself');

  const analyticsEvents = [];
  const server = createServer(async (request, response) => {
    const path = request.url?.split('?')[0];
    if (path === '/' || path === '/index.html') {
      response.writeHead(200, {
        'content-type': 'text/html; charset=utf-8',
        'content-security-policy': contentSecurityPolicy,
      });
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
    if (path === '/analytics.js') {
      response.writeHead(200, { 'content-type': 'application/javascript; charset=utf-8' });
      response.end(analytics);
      return;
    }
    if (path === '/analytics/script.js') {
      response.writeHead(200, { 'content-type': 'application/javascript; charset=utf-8' });
      response.end(`window.umami = { track: event => fetch('/analytics/api/send', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(event) }).then(response => { window.__analyticsSendStatus = response.status; }) };`);
      return;
    }
    if (path === '/analytics/api/send' && request.method === 'POST') {
      const chunks = [];
      for await (const chunk of request) chunks.push(chunk);
      analyticsEvents.push(JSON.parse(Buffer.concat(chunks).toString()));
      response.writeHead(200, { 'content-type': 'application/json' });
      response.end('{"ok":true}');
      return;
    }
    response.writeHead(404);
    response.end();
  });

  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  t.after(() => new Promise(resolve => server.close(resolve)));
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 }, colorScheme: 'no-preference' });
  const origin = `http://127.0.0.1:${server.address().port}`;
  const offsiteRequests = [];
  page.on('request', request => {
    if (!request.url().startsWith(origin)) offsiteRequests.push(request.url());
  });
  const analyticsScriptUrl = `${origin}/analytics/script.js`;
  const analyticsEventUrl = `${origin}/analytics/api/send`;
  const analyticsRequest = page.waitForRequest(request => request.url() === analyticsEventUrl && request.method() === 'POST', { timeout: 5000 });
  await page.goto(origin);

  await page.getByRole('heading', { name: 'Your library. Your devices.' }).waitFor();
  await page.getByRole('heading', { name: 'Your media stack, in one place.' }).waitFor();
  await page.getByRole('heading', { name: 'Useful boundaries. Honest limits.' }).waitFor();
  assert.equal(await page.getByRole('link', { name: 'Open Rawrarr app' }).count(), 1);
  assert.equal(await page.getByRole('link', { name: 'Open app' }).getAttribute('href'), 'https://roarlay.rawrarr.net/');
  assert.equal(await page.getByRole('link', { name: 'Open Rawrarr app' }).getAttribute('href'), 'https://roarlay.rawrarr.net/');
  assert.equal(await page.locator('script').count(), 2);
  assert.equal(await page.locator('script[data-auto-track="false"]').getAttribute('data-website-id'), '72da7a2e-fe1b-4058-b07b-912d526c939e');
  const sentAnalyticsRequest = await analyticsRequest;
  assert.equal(sentAnalyticsRequest.method(), 'POST');
  assert.equal((await sentAnalyticsRequest.response()).status(), 200);
  assert.deepEqual(JSON.parse(sentAnalyticsRequest.postData()), {
    website: '72da7a2e-fe1b-4058-b07b-912d526c939e',
    url: '/website',
    title: 'Rawrarr website',
  });
  await page.waitForFunction(() => window.__analyticsSendStatus === 200);
  assert.deepEqual(analyticsEvents, [{
    website: '72da7a2e-fe1b-4058-b07b-912d526c939e',
    url: '/website',
    title: 'Rawrarr website',
  }]);
  assert.equal(await page.locator('img').first().evaluate(image => image.complete && image.naturalWidth > 0), true);
  assert.equal(offsiteRequests.length, 0, `all browser requests are same-origin: ${offsiteRequests.join(', ')}`);

  assert.match(css, /:root\s*\{\s*color-scheme:\s*dark;[\s\S]*?--paper:\s*#101611;/, 'dark is the stylesheet fallback before the light media override');
  assert.match(css, /@media \(prefers-color-scheme: light\)\s*\{\s*:root\s*\{\s*color-scheme:\s*light;/, 'explicit system light overrides the dark fallback');
  const colorSchemes = [
    { name: 'light', scheme: 'light', root: 'light', body: 'rgb(245, 246, 241)', text: 'rgb(23, 37, 28)', surface: 'rgb(255, 255, 255)', privacy: 'rgb(34, 58, 42)' },
    { name: 'dark', scheme: 'dark', root: 'dark', body: 'rgb(16, 22, 17)', text: 'rgb(237, 243, 233)', surface: 'rgb(23, 31, 24)', privacy: 'rgb(26, 42, 29)' },
  ];

  for (const colorScheme of colorSchemes) {
    await page.emulateMedia({ colorScheme: colorScheme.scheme });
    const palette = await page.evaluate(() => {
      const style = element => getComputedStyle(element);
      return {
        scheme: style(document.documentElement).colorScheme,
        body: style(document.body).backgroundColor,
        text: style(document.body).color,
        surface: style(document.querySelector('.step-card')).backgroundColor,
        privacy: style(document.querySelector('.privacy-section')).backgroundColor,
      };
    });
    assert.deepEqual(palette, {
      scheme: colorScheme.root,
      body: colorScheme.body,
      text: colorScheme.text,
      surface: colorScheme.surface,
      privacy: colorScheme.privacy,
    }, `${colorScheme.name} palette is applied`);

    for (const viewport of [{ width: 1280, height: 800 }, { width: 375, height: 812 }]) {
      await page.setViewportSize(viewport);
      await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
      assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth), `${colorScheme.name} ${viewport.width}px layout has no horizontal overflow`);
      const action = await page.getByRole('link', { name: 'Open Rawrarr app' }).boundingBox();
      assert.ok(action, `${colorScheme.name} ${viewport.width}px primary action is rendered`);
      assert.ok(action.width >= 44 && action.height >= 44, `${colorScheme.name} ${viewport.width}px primary action has a usable hit area`);
      assert.ok(action.x >= 0 && action.y >= 0 && action.x + action.width <= viewport.width && action.y + action.height <= viewport.height, `${colorScheme.name} ${viewport.width}px primary action is in the initial viewport`);
    }
  }
});
