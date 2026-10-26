import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const workspace = await readFile(new URL('../../rust-arr/src/coordinator_workspace.html', import.meta.url), 'utf8');

test('Devices heading has a right-aligned Add a device link to node setup', () => {
  const devicesView = workspace.slice(workspace.indexOf('<section id="devices-view"'), workspace.indexOf('<section id="movies-view"'));
  assert.match(devicesView, /<div class="devices-page-heading"><h1 class="page-title">Devices<\/h1><a class="button-link" href="\/setup\/node">Add a device<\/a><\/div>/);
  assert.doesNotMatch(devicesView, /Add a device with Docker Compose/);
  assert.match(workspace, /\.devices-page-heading\s*\{[^}]*display:flex[^}]*flex-wrap:wrap[^}]*justify-content:space-between/);
  assert.match(workspace, /\.devices-page-heading \.button-link\s*\{[^}]*margin-left:auto/);
  assert.match(devicesView, /<a class="button-link" href="\/setup\/node">Set up a node and get its Docker Compose file<\/a>/);
});
