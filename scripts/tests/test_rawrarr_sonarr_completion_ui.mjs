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
  assert.match(updatedCompleteBranch, /libraryMessage\.hidden = true/);
  assert.match(source, /libraryMessage\.dataset\.state !== 'error'/);
  assert.match(source, /textContent = statusMessage/);
  assert.match(source, /statusMessage = `Loaded \$\{series\.length\} series from Sonarr\.`/);
  assert.match(workspace, /\.scrape-progress:not\(\[hidden\]\) \{ display:block;/);
});

test('Movies and TV share the provider-aware scraper status and progress bar updater', () => {
  assert.match(workspace, /id="radarr-scrape-status"/);
  assert.match(workspace, /id="radarr-scrape-progress"[^>]*aria-label="Radarr metadata and artwork scraping progress"/);
  assert.match(workspace, /id="sonarr-scrape-status"/);
  assert.match(workspace, /id="sonarr-scrape-progress"[^>]*aria-label="Sonarr metadata and artwork scraping progress"/);
  assert.match(source, /async function refreshArrScrapeStatus\(provider\)/);
  assert.match(source, /provider === 'radarr' \? '\/metadata\/artwork\/backfill' : '\/metadata\/sonarr\/backfill'/);
  assert.match(source, /Scraping \$\{name\} metadata and artwork to encrypted shared metadata/);
  assert.match(source, /location\.hash === '#movies' && selectedNodeDetails\?\.role === 'metadata'\) void refreshRadarrBackfillStatus\(\)/);
});
