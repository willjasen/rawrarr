import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const root = new URL('../../rust-arr/src/', import.meta.url);
const html = await readFile(new URL('coordinator_workspace.html', root), 'utf8');
const assets = Object.fromEntries(await Promise.all([
  'coordinator-workspace.mjs', 'coordinator-availability.mjs', 'coordinator-device-routes.mjs',
  'coordinator-node-avatar.mjs', 'profile-color.mjs', 'coordinator-ai.mjs',
].map(async name => [`/assets/${name}`, await readFile(new URL(name, root), 'utf8')])));
assets['/assets/coordinator-workspace.mjs'] += '\nwindow.testRenderMovies = renderMovies;';
assets['/assets/metadata-export.js'] = "window.rawrarrMiniExport = { deriveProfileRecipient: async () => 'age1pq1testrecipient', loadDeviceNames: async () => ({}) };";

for (const viewport of [{ width: 1280, height: 800 }, { width: 375, height: 812 }]) {
  test(`movie posters survive cached-to-live updates and filtering at ${viewport.width}px`, { timeout: 20000 }, async t => {
    const browser = await chromium.launch({ executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
    const page = await browser.newPage({ viewport });
    page.setDefaultTimeout(5000);
    const cached = [
      { tmdbId: 123, title: 'A deliberately long movie title that wraps on a narrow screen', year: 2026, hasFile: true, sizeOnDisk: 1024, artworkSource: 'shared' },
      { tmdbId: 456, title: 'Second movie', year: 2025, hasFile: false, sizeOnDisk: 0, artworkSource: 'shared' },
    ];
    const live = [{ ...cached[0], qualityProfile: 'Updated quality', monitored: true }, cached[1]];
    let releaseLive;
    const liveGate = new Promise(resolve => { releaseLive = resolve; });
    let releasePoster;
    const posterGate = new Promise(resolve => { releasePoster = resolve; });
    t.after(() => { releaseLive(); releasePoster(); return browser.close(); });
    const posterRequests = [];
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    await page.route('http://localhost/**', async route => {
      const path = new URL(route.request().url()).pathname;
      if (assets[path]) return route.fulfill({ contentType: 'text/javascript', body: assets[path] });
      if (path === '/') return route.fulfill({ contentType: 'text/html', body: html });
      if (path === '/health' || path === '/direct-api/health') return route.fulfill({ status: 204 });
      if (path === '/api/workspace/profile') return route.fulfill({ json: { displayName: 'Test user' } });
      if (path === '/api/workspace/nodes') return route.fulfill({ json: [{ nodeId: 'metadata-one', role: 'metadata', online: true, relayAvailable: true, directUrl: 'http://localhost/direct-api/' }] });
      if (path === '/api/workspace/pending') return route.fulfill({ json: [] });
      if (path === '/api/workspace/direct-session') return route.fulfill({ status: 204, headers: { 'set-cookie': `rawrarr_direct_session=${'a'.repeat(64)}; Path=/direct-api/; SameSite=Strict` } });
      if (path === '/direct-api/api/settings/age') return route.fulfill({ json: { unlocked: true, encryptedConfigExists: true } });
      if (path === '/direct-api/api/settings/radarr') return route.fulfill({ json: { configured: true, persisted: true } });
      if (path === '/direct-api/api/radarr/live-library') {
        await liveGate;
        return route.fulfill({ json: live });
      }
      if (path.endsWith('/poster') || path === '/replacement.svg') {
        posterRequests.push(path);
        if (path.includes('/456/')) await posterGate;
        return route.fulfill({ contentType: 'image/svg+xml', body: '<svg xmlns="http://www.w3.org/2000/svg" width="48" height="72"><rect width="48" height="72" fill="green"/></svg>' });
      }
      if (path.startsWith('/nodes/')) return route.fulfill({ status: 503, json: {} });
      if (path.startsWith('/direct-api/api/') || path.startsWith('/api/workspace/')) return route.fulfill({ json: {} });
      return route.fulfill({ status: 404 });
    });
    await page.addInitScript(items => {
      localStorage.setItem('rawrarr.browser.age-id', 'AGE-SECRET-KEY-PQ-1artwork-test-fixture');
      localStorage.setItem('rawrarr.library-text.v1:metadata-one:movies', JSON.stringify({ items }));
      sessionStorage.setItem('rawrarr.coordinator.selected-node', 'metadata-one');
    }, cached);
    await page.goto('http://localhost/#movies', { waitUntil: 'commit' });
    await page.waitForFunction(() => document.querySelector('[data-node-id="metadata-one"]')?.dataset.selected === 'true');
    await page.locator('#movies img[data-tmdb-id="123"]').scrollIntoViewIfNeeded();
    await page.waitForFunction(() => document.querySelector('#movies img[data-tmdb-id="123"]')?.naturalWidth > 0);
    const first = await page.locator('#movies img[data-tmdb-id="123"]').elementHandle();
    const second = await page.locator('#movies img[data-tmdb-id="456"]').elementHandle();
    await second.scrollIntoViewIfNeeded();
    if (!posterRequests.some(path => path.includes('/456/'))) await page.waitForRequest('**/456/poster');
    await page.evaluate(() => {
      window.posterRemovals = 0;
      const observer = new MutationObserver(records => {
        for (const record of records) for (const node of record.removedNodes) {
          if (node.nodeType === 1 && (node.matches('img') || node.querySelector('img'))) window.posterRemovals++;
        }
      });
      observer.observe(document.getElementById('movies'), { childList: true, subtree: true });
    });
    releaseLive();
    await page.waitForFunction(() => document.getElementById('content-status').textContent.includes('Radarr library entries loaded'));
    assert.equal(await first.evaluate(image => image === document.querySelector('#movies img[data-tmdb-id="123"]') && image.naturalWidth > 0), true);
    assert.equal(await second.evaluate(image => image === document.querySelector('#movies img[data-tmdb-id="456"]')), true, 'in-flight posters keep their original element');
    assert.match(await page.locator('#movies').textContent(), /Updated quality/);
    releasePoster();
    await page.waitForFunction(() => document.querySelector('#movies img[data-tmdb-id="456"]')?.naturalWidth > 0);

    await page.locator('#movies-search').fill('no matching title');
    assert.equal(await page.locator('#movies .library-card:visible').count(), 0);
    assert.match(await page.locator('#movies .library-empty').textContent(), /No titles match/);
    await page.locator('#movies-search').fill('');
    await page.locator('#movies-filter').selectOption('available');
    assert.equal(await page.locator('#movies .library-card:visible').count(), 1);
    await page.locator('#movies-filter').selectOption('all');
    assert.equal(await first.evaluate(image => image === document.querySelector('#movies img[data-tmdb-id="123"]') && image.naturalWidth > 0), true);
    assert.equal(await page.evaluate(() => window.posterRemovals), 0);
    assert.deepEqual(posterRequests.sort(), ['/direct-api/api/radarr/live-movies/123/poster', '/direct-api/api/radarr/live-movies/456/poster']);

    await page.locator('#movies .library-card').first().scrollIntoViewIfNeeded();
    const geometry = await page.evaluate(() => ({
      width: innerWidth, documentWidth: document.documentElement.scrollWidth,
      card: document.querySelector('#movies .library-card').getBoundingClientRect().toJSON(),
      title: document.querySelector('#movies .library-card-title').getBoundingClientRect().toJSON(),
    }));
    assert.ok(geometry.documentWidth <= geometry.width, 'Movies must not overflow horizontally');
    for (const rect of [geometry.card, geometry.title]) assert.ok(rect.width > 0 && rect.left >= 0 && rect.right <= viewport.width);
    const gridColumns = await page.evaluate(() => {
      const grid = document.createElement('div');
      grid.className = 'library-grid';
      Object.assign(grid.style, { position: 'fixed', visibility: 'hidden', width: '1000px', top: '0', left: '0' });
      for (let index = 0; index < 12; index++) {
        const card = document.createElement('div');
        card.className = 'library-card';
        grid.append(card);
      }
      document.body.append(grid);
      const columns = getComputedStyle(grid).gridTemplateColumns.trim().split(/\s+/).length;
      grid.remove();
      return columns;
    });
    assert.equal(gridColumns, viewport.width <= 560 ? 2 : 6, 'shared Movies and TV artwork grid should show six desktop columns and preserve two mobile columns');
    await page.locator('#refresh-movies').scrollIntoViewIfNeeded();
    assert.equal(await page.locator('#refresh-movies').isVisible(), true);

    await page.evaluate(items => window.testRenderMovies([{ ...items[0], title: 'Changed title', posterUrl: '/replacement.svg' }]), live);
    await page.waitForFunction(() => document.querySelector('#movies img')?.getAttribute('src') === '/replacement.svg' && document.querySelector('#movies img').complete);
    assert.equal(await page.locator('#movies .library-card').count(), 1, 'removed movies disappear');
    assert.equal(await page.locator('#movies .library-card-title').textContent(), 'Changed title');
    await page.evaluate(items => window.testRenderMovies([items[0], { ...items[1], tmdbId: 789 }]), live);
    assert.equal(await page.locator('#movies .library-card').count(), 2, 'new movies appear');
    assert.deepEqual(errors, []);
  });
}
