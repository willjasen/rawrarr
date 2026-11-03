import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const root = new URL('../../', import.meta.url);
const workspaceHtml = await readFile(new URL('rust-arr/src/coordinator_workspace.html', root), 'utf8');
const pageHtml = workspaceHtml.replace(
  '<script type="module" src="/assets/coordinator-workspace.mjs"></script>',
  '',
);

test('Settings separates profile and Ethereum/ENS cards responsively', async () => {
  const server = createServer((_request, response) => {
    response.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
    response.end(pageHtml);
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const address = server.address();
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });

  try {
    for (const viewport of [{ width: 1280, height: 800 }, { width: 375, height: 812 }]) {
      const page = await browser.newPage({ viewport });
      await page.goto(`http://127.0.0.1:${address.port}/`);
      await page.locator('#settings-view').evaluate(element => { element.hidden = false; });

      const profile = page.locator('.profile-card');
      const crypto = page.locator('.settings-crypto-card');
      assert.equal(await profile.locator('#profile-name').count(), 1);
      assert.equal(await profile.locator('#trial-status').count(), 1);
      assert.equal(await profile.locator('#crypto-access-status').count(), 0);
      assert.equal(await crypto.getByRole('heading', { name: 'Ethereum & ENS' }).count(), 1);
      assert.equal(await crypto.locator('#crypto-access-status').count(), 1);
      assert.equal(await crypto.locator('#crypto-wallet-status').count(), 1);
      assert.equal(await crypto.locator('#crypto-wallet-link-form').count(), 1);
      assert.equal(await crypto.locator('#crypto-ens-name').inputValue(), '');
      assert.equal(await crypto.locator('#crypto-ens-name').getAttribute('placeholder'), null);
      assert.equal(await crypto.getByRole('link', { name: 'Manage subscription' }).getAttribute('href'), '/billing');

      const geometry = await page.evaluate(() => {
        const bounds = selector => {
          const rect = document.querySelector(selector).getBoundingClientRect();
          return { x: rect.x, y: rect.y, right: rect.right, bottom: rect.bottom };
        };
        return {
          width: window.innerWidth,
          documentWidth: document.documentElement.scrollWidth,
          profile: bounds('.profile-card'),
          crypto: bounds('.settings-crypto-card'),
          input: bounds('#crypto-ens-name'),
          button: bounds('#crypto-link-wallet'),
        };
      });
      assert.ok(geometry.documentWidth <= geometry.width, `horizontal overflow at ${viewport.width}px`);
      for (const [name, bounds] of Object.entries({
        crypto: geometry.crypto,
        input: geometry.input,
        button: geometry.button,
      })) {
        assert.ok(bounds.x >= 0 && bounds.right <= geometry.width, `${name} exceeds the viewport at ${viewport.width}px`);
      }
      if (viewport.width > 900) {
        assert.ok(Math.abs(geometry.profile.y - geometry.crypto.y) < 1, 'profile and ETH/ENS cards should share a desktop row');
      } else {
        assert.ok(geometry.crypto.y > geometry.profile.y, 'ETH/ENS card should stack below the profile on mobile');
      }
      await page.close();
    }
  } finally {
    await browser.close();
    await new Promise(resolve => server.close(resolve));
  }
});
