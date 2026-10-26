import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const root = new URL('../../', import.meta.url);
const read = path => readFile(new URL(path, root), 'utf8');
const [main, workspace, script] = await Promise.all([
  read('rust-arr/src/main.rs'), read('rust-arr/src/coordinator_workspace.html'), read('rust-arr/src/coordinator-workspace.mjs')
]);

test('coordinator exposes Radarr detail slugs through encrypted and temporary direct library responses', () => {
  assert.match(main, /struct BrowserRadarrMovie[\s\S]*?title_slug: &'a Option<String>/);
  assert.match(main, /BrowserRadarrMovie \{[\s\S]*?title_slug: &movie\.title_slug/);
  assert.match(main, /struct DirectRadarrMovie[\s\S]*?title_slug: Option<String>[\s\S]*?genres: Vec<String>/);
  const liveHandler = main.match(/async fn radarr_movies_live[\s\S]*?\n}\n\nasync fn radarr_movie_poster_live/)?.[0] || '';
  assert.match(liveHandler, /\/api\/v3\/movie/);
  assert.doesNotMatch(liveHandler, /quality_profiles::enrich|path:/);
  assert.match(main, /async fn radarr_movies_live[\s\S]*?\.get\(format!\("\{\}\/api\/v3\/movie", radarr\.base_url\)\)[\s\S]*?DirectRadarrMovie/);
  assert.match(main, /async fn radarr_movie_poster_live[\s\S]*?fetch_movie_poster\(&movie\)/);
  assert.match(main, /radarr\.movie_snapshot\.store_poster_movies\(movies\.clone\(\)\)\.await/);
  assert.match(main, /async fn movie_by_tmdb_id[\s\S]*?movie_snapshot\.poster_movie\(tmdb_id\)\.await[\s\S]*?\/api\/v3\/movie/);
  assert.match(main, /let direct_protected = Router::new\([\s\S]*?\.route\("\/api\/radarr\/live-library", get\(radarr_movies_live\)\)[\s\S]*?"\/api\/radarr\/live-movies\/\{tmdb_id\}\/poster",[\s\S]*?get\(radarr_movie_poster_live\)/);
  assert.match(script, /arrDirectRequest\('\/radarr\/live-library'\)/);
  assert.match(script, /image\.src = `\$\{artworkBase\}\/radarr\/live-movies\/\$\{encodeURIComponent\(movie\.tmdbId\)\}\/poster`/);
  assert.match(script, /async function attachMoviePosters\(movies, nodeId\)/);
  assert.match(script, /image\.src = `\$\{artworkBase\}\/radarr\/live-movies\/\$\{encodeURIComponent\(movie\.tmdbId\)\}\/poster`/);
  assert.doesNotMatch(script.match(/async function unlockMetadata\(\)[\s\S]*?\n\}/)?.[0] || '', /loadRadarrLibrary|selectedRoute\.base/);
});

test('Movies and TV add accessible detail icons only after a browser reachability probe', () => {
  assert.match(script, /mode: 'no-cors', credentials: 'omit',[\s\S]*?referrerPolicy: 'no-referrer',[\s\S]*?AbortSignal\.timeout\(3000\)/);
  assert.match(script, /function appendArrDetailLink\(titleCell, service, item\)/);
  assert.match(script, /appendArrDetailLink\(titleRow, config\.service, item\)/);
  assert.match(script, /appendArrDetailLink\(card\.querySelector\('\.library-title-row'\), 'radarr', movie\)/);
  assert.match(workspace, /\.arr-external-link svg/);
});
