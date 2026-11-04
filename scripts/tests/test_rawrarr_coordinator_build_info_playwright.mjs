import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const root = new URL('../../', import.meta.url);
const [workspaceHtml, workspaceScript] = await Promise.all([
  readFile(new URL('rust-arr/src/coordinator_workspace.html', root), 'utf8'),
  readFile(new URL('rust-arr/src/coordinator-workspace.mjs', root), 'utf8'),
]);
const versionScriptStart = workspaceScript.indexOf("fetch('/api/version'");
const versionScriptEnd = workspaceScript.indexOf('.catch(() => {});', versionScriptStart) + '.catch(() => {});'.length;
assert.ok(versionScriptStart >= 0 && versionScriptEnd > versionScriptStart, 'version display script exists');
const versionScript = workspaceScript.slice(versionScriptStart, versionScriptEnd);
const pageHtml = workspaceHtml.replace(
  '<script type="module" src="/assets/coordinator-workspace.mjs"></script>',
  `<script>${versionScript}</script>`,
);

test('coordinator deployment timestamp is visible at desktop and mobile sizes', async () => {
  const server = createServer((request, response) => {
    if (request.url === '/api/version') {
      response.writeHead(200, { 'content-type': 'application/json', 'cache-control': 'no-store' });
      response.end(JSON.stringify({ revision: 'a'.repeat(40), deployedAt: '2026-11-03T05:21:10Z' }));
      return;
    }
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
      const label = page.locator('#coordinator-last-updated');
      await page.waitForFunction(() => document.getElementById('coordinator-last-updated')?.textContent.startsWith('Last updated on'));
      assert.equal(await label.isVisible(), true, `timestamp is hidden at ${viewport.width}px`);
      const geometry = await page.evaluate(() => {
        const label = document.getElementById('coordinator-last-updated');
        const rect = label.getBoundingClientRect();
        return {
          width: window.innerWidth,
          documentWidth: document.documentElement.scrollWidth,
          x: rect.x,
          y: rect.y,
          right: rect.right,
          bottom: rect.bottom,
          viewportHeight: window.innerHeight,
        };
      });
      assert.ok(geometry.documentWidth <= geometry.width, `horizontal overflow at ${viewport.width}px`);
      assert.ok(geometry.x >= 0 && geometry.right <= viewport.width, `timestamp exceeds the viewport at ${viewport.width}px`);
      assert.ok(geometry.bottom <= geometry.viewportHeight && geometry.y > geometry.viewportHeight - 100,
        `timestamp is not at the bottom-left at ${viewport.width}px`);
      if (viewport.width <= 850) {
        assert.equal(await page.locator('.sidebar-bottom-profile').isVisible(), false);
        assert.ok(geometry.x >= 70, 'mobile timestamp overlaps the collapsed navigation rail');
      }
      await page.close();
    }
  } finally {
    await browser.close();
    await new Promise(resolve => server.close(resolve));
  }
});
