import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const source = await readFile(new URL('../../rust-arr/src/coordinator-workspace.mjs', import.meta.url), 'utf8');

test('Movies and TV render node-scoped browser-cached text before live refresh', () => {
  assert.match(source, /const libraryTextCachePrefix = 'rawrarr\.library-text\.v1'/);
  assert.match(source, /readLibraryTextCache\(selectedNode, 'tv'\)[\s\S]*?renderSonarrSeries\(cachedSeries, 'Cached TV library\. Updating…'\)/);
  assert.match(source, /function renderCachedMovies\(nodeId\)[\s\S]*?renderMovies\(cached\.map\(movie => \(\{ \.\.\.movie, posterUrl: null \}\)\)\)/);
  assert.match(source, /const hasCachedMovies = renderCachedMovies\(nodeId\);[\s\S]*?const liveMovies = await arrDirectRequest\('\/radarr\/live-library'\)/);
  assert.match(source, /writeLibraryTextCache\(nodeId, 'tv', series\)[\s\S]*?renderSonarrSeries\(series/);
  assert.match(source, /writeLibraryTextCache\(nodeId, 'movies', movies\.map/);
});

test('poster requests are attached after text rendering and cache excludes artwork bytes', () => {
  assert.match(source, /void attachSonarrPosters\(rows\)/);
  assert.match(source, /void attachMoviePosters\(movies, nodeId\)/);
  assert.match(source, /poster\.src = `\$\{artworkBase\}\/sonarr\/series/);
  assert.match(source, /image\.src = `\$\{artworkBase\}\/radarr\/live-movies/);
  assert.match(source, /writeLibraryTextCache\(nodeId, 'movies', movies\.map\(\(\{ posterUrl: _posterUrl, \.\.\.movie \}\) => movie\)\)/);
});
