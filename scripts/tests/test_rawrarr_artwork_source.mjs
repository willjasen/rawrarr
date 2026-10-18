import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const source = await readFile(new URL('../../rust-arr/src/main.rs', import.meta.url), 'utf8');
const ui = await readFile(new URL('../../rust-arr/src/ui.html', import.meta.url), 'utf8');

test('Radarr movie API reports whether poster artwork is cached in shared data', () => {
  assert.match(source, /artwork_source: Option<String>/);
  assert.match(source, /join\("movies"\)[\s\S]*?join\("artwork"\)[\s\S]*?format!\("\{id\}\.aged"\)/);
  assert.match(source, /if cached \{ "shared" \} else \{ "radarr" \}/);
});

test('Movies table shows artwork source and marks failed poster loads unavailable', () => {
  assert.match(ui, /key: 'load', label: \(\) => 'Load time'/);
  assert.match(ui, /key: 'artwork', label: \(\) => 'Artwork source'/);
  assert.match(ui, /movie\.artworkSource === 'shared'[\s\S]*?'Shared folder'[\s\S]*?app\.name/);
  assert.match(ui, /entry\.responseEnd - entry\.requestStart/);
  assert.match(ui, /poster\.addEventListener\('load', \(\) => \{[\s\S]*?artworkRequestDuration\(poster\.currentSrc \|\| poster\.src\)[\s\S]*?artworkLoadTime\.textContent/);
  assert.match(ui, /artworkLoadTime\.textContent = 'Failed'/);
  assert.match(ui, /artworkSource\.textContent = 'Unavailable'/);
  assert.match(ui, /load: artworkLoadTime, artwork: artworkSource/);
});
