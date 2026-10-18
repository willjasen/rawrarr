import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';

const root = new URL('../../', import.meta.url);
const read = path => readFile(new URL(path, root), 'utf8');
const [main, sonarr, config, workspace, workspaceScript] = await Promise.all([
  read('rust-arr/src/main.rs'),
  read('rust-arr/src/sonarr.rs'),
  read('rust-arr/src/secure_config.rs'),
  read('rust-arr/src/coordinator_workspace.html'),
  read('rust-arr/src/coordinator-workspace.mjs'),
]);

test('Sonarr metadata scraping starts on metadata nodes and repeats after six hours', () => {
  assert.match(main, /if role == node_role::NodeRole::Metadata\s*\{[\s\S]*?tokio::spawn\(run_metadata_sonarr_backfill/);
  assert.match(main, /async fn run_metadata_sonarr_backfill[\s\S]*?sleep\(Duration::from_secs\(10\)\)/);
  assert.match(main, /wait_for_sonarr_backfill\(&state, Duration::from_secs\(6 \* 60 \* 60\)\)/);
  assert.match(main, /sonarr_scrape_notify\.notified\(\)/);
  assert.match(sonarr, /state\.sonarr_scrape_notify\.notify_one\(\)/);
});

test('Sonarr backfill is sequential, cache-first, and limited to series with posters and TVDB IDs', () => {
  assert.match(main, /sonarr::has_poster\(&item\)/);
  assert.match(main, /!vault\.tv_metadata_exists\(\*tvdb_id\) \|\| !vault\.tv_artwork_exists\(\*tvdb_id\)/);
  assert.match(main, /for \(tvdb_id, item\) in missing\s*\{[\s\S]*?sonarr::scrape_and_store\([\s\S]*?sleep\(Duration::from_millis\(250\)\)/);
  assert.match(sonarr, /"tvdbId": tvdb_id/);
  assert.match(sonarr, /"provider": "sonarr"/);
  assert.match(sonarr, /store_tv_metadata/);
  assert.match(sonarr, /store_tv_artwork/);
  assert.match(sonarr, /load_tv_artwork\(tvdb_id\)[\s\S]*?existing_artwork_is_valid\(existing_artwork\)/);
  assert.match(sonarr, /Err\(secure_config::VaultError::DecryptionFailed\) => Ok\(false\)/);
  assert.match(sonarr, /validated_tvdb_image_url\(remote\)/);
  assert.match(sonarr, /https/);
  assert.match(sonarr, /artworks\.thetvdb\.com/);
  assert.match(sonarr, /client\.http\.get\(url\)\.send\(\)/);
  assert.doesNotMatch(sonarr, /"path": series\.get\("path"\)/);
});

test('Sonarr metadata and artwork stay in separate HMAC-named age-encrypted TV folders', () => {
  assert.match(config, /join\("tv"\)\.join\("artwork"\)/);
  assert.match(config, /join\("tv"\)\.join\("text"\)/);
  assert.match(config, /opaque_metadata_filename\("artwork", "tvdb"/);
  assert.match(config, /opaque_metadata_filename\("text", "tvdb"/);
  assert.match(config, /store_tv_artwork[\s\S]*?store_artwork_with_recipient/);
  assert.match(config, /store_tv_metadata[\s\S]*?encrypt_shared_file/);
  assert.match(workspace, /id="sonarr-scrape-status"/);
  assert.match(workspace, /id="sonarr-scrape-progress"/);
  assert.match(workspaceScript, /arrDirectRequest\('\/metadata\/sonarr\/backfill'\)/);
  assert.match(workspaceScript, /Saving Sonarr metadata and artwork to encrypted shared metadata/);
  assert.match(workspaceScript, /poster\.src = `\$\{artworkBase\}\/sonarr\/series\/\$\{encodeURIComponent\(item\.tvdbId\)\}\/poster`/);
  assert.match(workspaceScript, /poster\.loading = 'lazy'/);
  assert.match(main, /\.route\("\/api\/sonarr\/series\/\{tvdb_id\}\/poster", get\(get_sonarr_poster\)\)/);
  assert.match(main, /vault\.load_tv_artwork\(tvdb_id\)/);
  assert.match(main, /\.route\("\/api\/metadata\/sonarr\/backfill", get\(metadata_sonarr_backfill_status\)\)/);
});
