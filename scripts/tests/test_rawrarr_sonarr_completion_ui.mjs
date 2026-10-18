import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';

const [source, workspace] = await Promise.all([
  readFile(new URL('../../rust-arr/src/coordinator-workspace.mjs', import.meta.url), 'utf8'),
  readFile(new URL('../../rust-arr/src/coordinator_workspace.html', import.meta.url), 'utf8'),
]);

test('completed Sonarr artwork scraping hides both completion messages and the progress bar', () => {
  assert.match(source, /const complete = progress\.scraped >= progress\.total;/, 'all posters saved is complete even while the worker checks the library');
  const updatedCompleteBranch = source.match(/const complete = progress\.scraped >= progress\.total;([\s\S]*?)label\.hidden = false;/)?.[1];
  assert.ok(updatedCompleteBranch, 'completion branch is present');
  assert.match(updatedCompleteBranch, /if \(complete\)\s*\{[\s\S]*?label\.hidden = true;[\s\S]*?bar\.hidden = true;/);
  assert.match(updatedCompleteBranch, /seriesMessage\.hidden = true/);
  assert.match(source, /seriesMessage\.dataset\.state !== 'error'/);
  assert.match(source, /message\.textContent = `Loaded \$\{series\.length\} series from Sonarr\.`/);
  assert.match(workspace, /\.scrape-progress:not\(\[hidden\]\) \{ display:block;/);
});
