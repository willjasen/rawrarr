import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const root = new URL('../../', import.meta.url);
const [router, dockerfile, html, script] = await Promise.all([
  readFile(new URL('rust-arr/src/coordinator.rs', root), 'utf8'),
  readFile(new URL('rust-arr/Dockerfile', root), 'utf8'),
  readFile(new URL('rust-arr/src/coordinator_workspace.html', root), 'utf8'),
  readFile(new URL('rust-arr/src/coordinator-workspace.mjs', root), 'utf8'),
]);

test('coordinator exposes its immutable build revision and deployment timestamp', () => {
  assert.match(router, /\.route\("\/api\/version", get\(coordinator_version\)\)/);
  assert.match(router, /RAWARR_BUILD_REVISION/);
  assert.match(router, /RAWARR_DEPLOYED_AT/);
  assert.match(dockerfile, /ARG RAWARR_BUILD_REVISION=unknown/);
});

test('coordinator workspace renders deployment time in its sidebar', () => {
  assert.match(html, /id="coordinator-last-updated"/);
  assert.match(script, /fetch\('\/api\/version', \{ cache: 'no-store' \}\)/);
  assert.match(script, /Last updated on/);
});
