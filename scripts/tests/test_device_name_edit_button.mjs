import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';

const root = new URL('../../', import.meta.url);
const [workspace, script] = await Promise.all([
  readFile(new URL('rust-arr/src/coordinator_workspace.html', root), 'utf8'),
  readFile(new URL('rust-arr/src/coordinator-workspace.mjs', root), 'utf8'),
]);

test('device name has an adjacent Edit button that unlocks the name field', () => {
  assert.match(script, /nameInput\.readOnly = true/);
  assert.match(script, /editNameButton\.textContent = 'Edit'/);
  assert.match(script, /nameGroup\.append\(nameInput, editNameButton\)/);
  assert.match(script, /nameInput\.readOnly = false;\s*nameInput\.focus\(\);/);
  assert.match(script, /saveDeviceName\(ageIdentity/);
  assert.match(workspace, /\.device-name-group \{ display:flex/);
  assert.match(workspace, /\.device-name-edit-button \{ flex:none/);
});
