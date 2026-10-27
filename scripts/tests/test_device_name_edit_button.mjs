import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';

const root = new URL('../../', import.meta.url);
const [workspace, script] = await Promise.all([
  readFile(new URL('rust-arr/src/coordinator_workspace.html', root), 'utf8'),
  readFile(new URL('rust-arr/src/coordinator-workspace.mjs', root), 'utf8'),
]);

test('device card Edit action opens a dialog to rename or delete the device', () => {
  assert.match(script, /editButton\.textContent = 'Edit'/);
  assert.match(script, /editButton\.setAttribute\('aria-label', `Edit \$\{currentDeviceName\}`\)/);
  assert.match(script, /deviceEditDialog\.showModal\(\)/);
  assert.match(script, /deviceEditForm\.addEventListener\('submit'/);
  assert.match(script, /saveDeviceName\(ageIdentity/);
  assert.match(script, /deviceEditDelete\.addEventListener\('click'/);
  assert.match(script, /method: 'DELETE'/);
  assert.match(script, /does not delete data on the device/);
  assert.doesNotMatch(script, /device-delete-button|textContent = 'Delete device'/);
  assert.match(workspace, /<button id="device-edit-delete" class="device-edit-delete" type="button">Delete device<\/button>/);
  assert.match(workspace, /<button id="device-edit-save" type="submit">Save name<\/button>/);
  assert.match(workspace, /\.device-edit-dialog::backdrop/);
  assert.match(workspace, /\.device-edit-delete:hover/);
});
