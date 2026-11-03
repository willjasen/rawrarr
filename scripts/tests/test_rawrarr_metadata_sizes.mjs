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

test('subscriber startup initializes metadata before any identity or backfill writes', () => {
  const coordinatorReturn = server.indexOf('println!("rawrarr coordinator listening on {address}")');
  const initialization = server.indexOf('secure_config::AgeVault::initialize_metadata_storage()?');
  const identity = server.indexOf('secure_config::AgeVault::ensure_node_identity()?');
  assert.ok(coordinatorReturn >= 0 && initialization > coordinatorReturn);
  assert.ok(identity > initialization);
  assert.match(server, /\*state\.vault\.write\(\)\.await = Some\(vault\);\s*state\.artwork_scrape_notify\.notify_one\(\);\s*state\.sonarr_scrape_notify\.notify_one\(\)/);
});

test('metadata storage read failures are explicit rather than zero-byte successes', () => {
  const handler = server.slice(server.indexOf('async fn metadata_storage_status()'), server.indexOf('async fn run_metadata_artwork_backfill'));
  assert.match(handler, /Result<Json<MetadataStorageStatus>/);
  assert.match(handler, /StatusCode::INTERNAL_SERVER_ERROR/);
  assert.match(handler, /Could not read encrypted metadata storage/);
  assert.doesNotMatch(handler, /unwrap_or/);
});
