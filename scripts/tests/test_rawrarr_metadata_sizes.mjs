import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const ui = await readFile(new URL('../../rust-arr/src/coordinator-workspace.mjs', import.meta.url), 'utf8');
const server = await readFile(new URL('../../rust-arr/src/main.rs', import.meta.url), 'utf8');
const html = await readFile(new URL('../../rust-arr/src/coordinator_workspace.html', import.meta.url), 'utf8');

test('Movies and TV cards use their own metadata subtree sizes', () => {
  assert.match(server, /movies_bytes:\s*u64,[\s\S]*tv_bytes:\s*u64/);
  assert.match(server, /directory_size\(&metadata_dir\.join\("movies"\)\)/);
  assert.match(server, /directory_size\(&metadata_dir\.join\("tv"\)\)/);
  assert.match(server, /tv_bytes:\s*bytes\.2/);
  assert.match(ui, /setArrSummaryValues\('radarr', \{ metadataSize: formatBytes\(sizes\.moviesBytes\), allMetadataSize: formatBytes\(sizes\.bytes\) \}\)/);
  assert.match(ui, /setArrSummaryValues\('sonarr', \{ metadataSize: formatBytes\(sizes\.tvBytes\), allMetadataSize: formatBytes\(sizes\.bytes\) \}\)/);
  assert.match(html, /id="radarr-summary" class="summary"/);
  assert.match(html, /id="sonarr-summary" class="summary"/);
  assert.match(ui, /key: 'metadataSize', label: 'Metadata size', detail: 'Radarr movie metadata'/);
  assert.match(ui, /key: 'metadataSize', label: 'Metadata size', detail: 'Sonarr series metadata'/);
  assert.doesNotMatch(ui, /Movies and TV metadata/);
});
