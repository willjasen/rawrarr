import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const html = await readFile(new URL('../../rust-arr/src/coordinator_workspace.html', import.meta.url), 'utf8');
const js = await readFile(new URL('../../rust-arr/src/coordinator-workspace.mjs', import.meta.url), 'utf8');
const rust = await readFile(new URL('../../rust-arr/src/main.rs', import.meta.url), 'utf8');
const sonarr = await readFile(new URL('../../rust-arr/src/sonarr.rs', import.meta.url), 'utf8');
const prowlarr = await readFile(new URL('../../rust-arr/src/prowlarr.rs', import.meta.url), 'utf8');

test('Settings offers compact connection cards and service configuration dialogs', () => {
  for (const service of ['radarr', 'sonarr', 'prowlarr']) {
    assert.match(html, new RegExp(`settings-configure-${service}`));
    assert.match(html, new RegExp(`settings-${service}-dialog`));
    assert.match(html, new RegExp(`settings-test-${service}`));
  }
  assert.match(html, /\.settings-connections \.connection-card \{ padding:16px 18px \}/);
});

test('connection tests are sent to the selected node and validated by node code', () => {
  assert.match(js, /method: 'POST'[\s\S]*?body: JSON\.stringify\(payload\)/);
  assert.match(js, /Testing \$\{fields\.label\} from the selected node/);
  assert.match(rust, /\.post\(test_radarr_settings\)/);
  assert.match(sonarr, /test_connection[\s\S]*?client\.status\(\)/);
  assert.match(rust, /test_radarr_settings[\s\S]*?client\.status\(\)/);
  assert.match(rust, /\.post\(prowlarr::test_connection\)/);
  assert.match(prowlarr, /test_connection[\s\S]*?status\(&client\)/);
});
