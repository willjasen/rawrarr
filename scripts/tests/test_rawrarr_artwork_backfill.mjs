import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const ui = await readFile(new URL('../../rust-arr/src/ui.html', import.meta.url), 'utf8');
const source = await readFile(new URL('../../rust-arr/src/main.rs', import.meta.url), 'utf8');

test('metadata nodes backfill one uncached Radarr poster at a time', () => {
  assert.match(source, /if role == node_role::NodeRole::Metadata\s*\{\s*tokio::spawn\(run_metadata_artwork_backfill/);
  assert.match(source, /movie\.poster_url\.is_some\(\) && movie\.artwork_source\.as_deref\(\) == Some\("radarr"\)/);
  assert.match(source, /let total = movies\.iter\(\)\.filter\(\|movie\| movie\.poster_url\.is_some\(\)\)\.count\(\)/);
  assert.match(source, /let cached = vault\.as_ref\(\)\.is_some_and\(\|vault\| vault\.artwork_exists\(tmdb_id\)\)/);
  assert.match(source, /for tmdb_id in missing\s*\{[\s\S]*?scrape_and_store_movie_artwork\(&radarr, vault\.clone\(\), tmdb_id, state\.artwork_scrape_lock\.clone\(\)\)\.await/);
  assert.match(source, /\.route\("\/api\/metadata\/artwork\/backfill", get\(metadata_artwork_backfill_status\)\)/);
  assert.match(ui, /id="artwork-backfill-progress"/);
  assert.match(ui, /bar\.value = progress\.scraped/);
  assert.match(ui, /progress\.scraped\.toLocaleString\(\).*progress\.total\.toLocaleString\(\).*posters saved/);
  assert.match(ui, /const complete = !progress\.active && progress\.scraped >= progress\.total/);
  assert.match(ui, /bar\.hidden = !progress\.active/);
  assert.match(ui, /Artwork cache complete/);
  assert.match(ui, /id="metadata-size"/);
  assert.match(ui, /class="grid movies-summary-grid"/);
  assert.match(ui, /\.movies-summary-grid \{ grid-template-columns: repeat\(4, minmax\(0, 1fr\)\); \}/);
  assert.match(ui, /fetch\('\/api\/metadata\/storage'/);
  assert.match(ui, /window\.setInterval\(refreshMetadataStorageSize, 3000\)/);
  assert.match(source, /\.route\("\/api\/metadata\/storage", get\(metadata_storage_status\)\)/);
  assert.match(source, /directory_size\(&metadata_dir\)/);
  assert.doesNotMatch(ui, /saveMissingMovieArtwork/);
});
