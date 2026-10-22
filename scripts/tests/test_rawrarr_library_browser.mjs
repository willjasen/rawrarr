import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const root = new URL('../../', import.meta.url);
const [html, script] = await Promise.all([
  readFile(new URL('rust-arr/src/coordinator_workspace.html', root), 'utf8'),
  readFile(new URL('rust-arr/src/coordinator-workspace.mjs', root), 'utf8'),
]);

test('Movies and TV share the poster browser, search, filters, and service-specific availability', () => {
  assert.equal((script.match(/renderLibraryBrowser\(\{/g) || []).length, 3); // definition plus both libraries
  assert.match(script, /function renderLibraryBrowser\(\{ containerId, searchId, filterId, countId, items, type \}\)/);
  assert.match(script, /containerId:'movies', searchId:'movies-search', filterId:'movies-filter'/);
  assert.match(script, /containerId:'sonarr-series', searchId:'tv-search', filterId:'tv-filter'/);
  assert.match(script, /mode === 'available' && !available \|\| mode === 'missing' && available \|\| mode === 'monitored' && !item\.monitored/);
  assert.match(script, /available:item => item\.hasFile/);
  assert.match(script, /available:item => Number\(item\.episodeFileCount\) > 0/);
  assert.match(html, /id="movies" class="library-grid"/);
  assert.match(html, /id="sonarr-series" class="library-grid"/);
  assert.match(html, /\.library-grid \{ display:grid; grid-template-columns:repeat\(auto-fill/);
});
