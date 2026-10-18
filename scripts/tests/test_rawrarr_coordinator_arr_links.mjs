import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const root = new URL('../../', import.meta.url);
const read = path => readFile(new URL(path, root), 'utf8');
const [main, workspace, script] = await Promise.all([
  read('rust-arr/src/main.rs'), read('rust-arr/src/coordinator_workspace.html'), read('rust-arr/src/coordinator-workspace.mjs')
]);

test('coordinator exposes Radarr slugs only through the encrypted library response and direct metadata route', () => {
  assert.match(main, /struct BrowserRadarrMovie[\s\S]*?title_slug: &'a Option<String>/);
  assert.match(main, /BrowserRadarrMovie \{[\s\S]*?title_slug: &movie\.title_slug/);
  assert.match(main, /let direct_protected = Router::new\([\s\S]*?\.route\("\/api\/radarr\/movies", get\(radarr_movies\)\)/);
});

test('Movies and TV add accessible detail icons only after a browser reachability probe', () => {
  assert.match(script, /mode: 'no-cors', credentials: 'omit',[\s\S]*?referrerPolicy: 'no-referrer',[\s\S]*?AbortSignal\.timeout\(3000\)/);
  assert.match(script, /function appendArrDetailLink\(titleCell, service, item\)/);
  assert.match(script, /appendArrDetailLink\(title, 'sonarr', item\)/);
  assert.match(script, /appendArrDetailLink\(row\.children\[0\], 'radarr', movie\)/);
  assert.match(workspace, /\.arr-external-link svg/);
});
