import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const ui = await readFile(new URL('../../rust-arr/src/ui.html', import.meta.url), 'utf8');

test('Movies and TV show the year and Arr detail icon in the artwork and title column', () => {
  const rendererStart = ui.indexOf('function renderArrLibrary');
  const renderer = ui.slice(rendererStart, ui.indexOf("document.getElementById('refresh-movies')", rendererStart));
  assert.match(renderer, /title\.textContent = `\$\{movie\.title\} \(\$\{movie\.year \|\| '—'\}\)`/);
  assert.match(renderer, /const detailLinks = \[\]/);
  assert.match(renderer, /const titleLink = titleCell\.querySelector\('a'\);\s*\(titleLink \|\| titleContent\)\.append\(detailLink\)/);
  assert.doesNotMatch(renderer, /yearCell\.append\(detailLink\)|yearCell\.append\(yearLink\)/);
});
