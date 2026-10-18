import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const source = await readFile(new URL('../../rust-arr/src/coordinator-workspace.mjs', import.meta.url), 'utf8');
const html = await readFile(new URL('../../rust-arr/src/coordinator_workspace.html', import.meta.url), 'utf8');

test('Radarr and Sonarr summaries use one configurable four-card renderer', () => {
  assert.match(source, /const ARR_SUMMARY_DEFINITIONS = \{/);
  assert.match(source, /radarr:\s*\{[\s\S]*?containerId: 'radarr-summary',[\s\S]*?cards: \[/);
  assert.match(source, /sonarr:\s*\{[\s\S]*?containerId: 'sonarr-summary',[\s\S]*?cards: \[/);
  assert.equal((source.match(/\{ key: '(?:libraryCount|filesPresent|diskUsage|metadataSize)'/g) || []).length, 8);
  assert.match(source, /function renderArrSummaryCards\(service\)/);
  assert.match(source, /function setArrSummaryValues\(service, values\)/);
  assert.match(source, /function resetArrSummary\(service\)/);
  assert.match(source, /renderArrSummaryCards\('radarr'\);[\s\S]*renderArrSummaryCards\('sonarr'\);/);
  assert.match(html, /id="radarr-summary" class="summary"/);
  assert.match(html, /id="sonarr-summary" class="summary"/);
  assert.doesNotMatch(html, /Movies and TV metadata/);
});
