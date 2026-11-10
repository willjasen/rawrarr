import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const authPage = await readFile(new URL('../../rust-arr/src/auth.html', import.meta.url), 'utf8');
const passkeyClient = await readFile(new URL('../../rust-arr/src/passkey-client.mjs', import.meta.url), 'utf8');

test('passkey sign-in and account creation fit desktop and mobile auth pages', async t => {
  let userExists = false;
  const server = createServer((request, response) => {
    const pathname = new URL(request.url, 'http://127.0.0.1').pathname;
    if (pathname === '/auth') {
      response.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
      response.end(authPage);
    } else if (pathname === '/assets/passkey-client.mjs') {
      response.writeHead(200, { 'content-type': 'text/javascript; charset=utf-8' });
      response.end(passkeyClient);
    } else if (pathname === '/api/auth/status') {
      response.writeHead(200, { 'content-type': 'application/json' });
      response.end(JSON.stringify({ userExists }));
    } else {
      response.writeHead(404);
      response.end();
    }
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  t.after(() => new Promise(resolve => server.close(resolve)));

  const browser = await chromium.launch({
    headless: true,
    executablePath: '/usr/bin/chromium',
    args: ['--no-sandbox'],
  });
  t.after(() => browser.close());
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  const authUrl = `http://127.0.0.1:${server.address().port}/auth`;

  await page.goto(authUrl);
  const createPasskey = page.getByRole('button', { name: 'Create Passkey' });
  const signIn = page.getByRole('button', { name: 'Sign in with passkey' });
  await createPasskey.waitFor();
  assert.equal(await createPasskey.isVisible(), true);
  assert.equal(await signIn.isVisible(), false, 'new visitors can create an account without a prior sign-in');

  userExists = true;
  await page.reload();
  await signIn.waitFor();
  assert.equal(await createPasskey.isVisible(), true, 'existing accounts do not hide account creation');
  assert.equal(await page.getByRole('button', { name: 'Sign in with ENS wallet' }).isVisible(), true);

  for (const viewport of [
    { width: 1280, height: 800 },
    { width: 375, height: 812 },
  ]) {
    await page.setViewportSize(viewport);
    await page.reload();
    await signIn.waitFor();

    assert.equal(
      await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
      true,
      `auth page has no horizontal overflow at ${viewport.width}px`,
    );
    const signInBox = await signIn.boundingBox();
    const createBox = await createPasskey.boundingBox();
    const walletBox = await page.getByRole('button', { name: 'Sign in with ENS wallet' }).boundingBox();
    assert.ok(signInBox && createBox && walletBox, 'all primary authentication actions are rendered');
    assert.ok(createBox.y >= signInBox.y + signInBox.height, 'Create Passkey is below passkey sign-in');
    for (const box of [signInBox, createBox, walletBox]) {
      assert.ok(box.x >= 0 && box.x + box.width <= viewport.width, 'authentication action fits viewport width');
      assert.ok(box.y >= 0 && box.y + box.height <= viewport.height, 'authentication action is in view');
    }
  }
});
