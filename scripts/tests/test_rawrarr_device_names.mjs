import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';

const root = new URL('../../', import.meta.url);
const read = path => readFile(new URL(path, root), 'utf8');
const [config, main, coordinator, workspace, routes, browser, html] = await Promise.all([
  read('rust-arr/src/secure_config.rs'),
  read('rust-arr/src/main.rs'),
  read('rust-arr/src/coordinator.rs'),
  read('rust-arr/src/coordinator-workspace.mjs'),
  read('rust-arr/src/coordinator-device-routes.mjs'),
  read('rust-arr/web-export/metadata-export.js'),
  read('rust-arr/src/coordinator_workspace.html'),
]);

test('device names are persisted as a backward-compatible age config field', () => {
  assert.match(config, /pub device_names: std::collections::HashMap<String, String>/);
  assert.match(config, /device_names: existing\.as_ref\(\)\.map\(\|stored\| stored\.device_names\.clone\(\)\)/);
  assert.match(browser, /config\.device_names = names/);
  assert.match(browser, /canonicalizeConfig\(config\)/);
  assert.match(browser, /normalizeConfigShape\(config\)/);
  assert.match(browser, /config\.services = services/);
  assert.match(browser, /config\.schema_version = 1/);
  assert.match(browser, /new age\.Encrypter\(\)/);
});

test('Overview and device management stay separate, with age-encrypted name editing in a dialog', () => {
  assert.match(html, /data-view="fossils"[^>]*>[\s\S]*?Fossils/);
  assert.match(html, /id="overview-view"><h1 class="page-title">Overview<\/h1>/);
  assert.match(html, /id="fossils-view"[\s\S]*?id="pending"[\s\S]*?id="nodes"/);
  assert.match(workspace, /'overview', 'fossils', 'movies'/);
  assert.match(workspace, /nameLabel\.className = 'device-name'/);
  assert.match(workspace, /deviceEditForm\.addEventListener\('submit'/);
  assert.match(workspace, /deviceEditDialog\.showModal\(\)/);
  assert.match(workspace, /card\.addEventListener\('click', event =>/);
  assert.match(workspace, /card\.append\(identityRow, detail\)/);
  assert.doesNotMatch(workspace, /textContent = 'Open node'/);
  assert.match(workspace, /editButton\.textContent = 'Edit'/);
  assert.match(workspace, /nameGroup\.append\(nameLabel\)/);
  assert.match(html, /\.device-name-group \{ display:flex/);
});

test('coordinator UI decrypts names in the browser and only relays age ciphertext for writes', () => {
  assert.match(workspace, /loadDeviceNames\(ageIdentity/);
  assert.match(workspace, /saveDeviceName\(ageIdentity/);
  assert.match(browser, /decrypter\.addIdentity\(candidate\)/);
  assert.match(browser, /method: 'PUT'[\s\S]*body: ciphertext/);
  assert.match(coordinator, /request_body\.starts_with\(b"age-encryption\.org\/v1\\n"\)/);
  assert.match(coordinator, /api\/metadata\/export\/recipient/);
  assert.match(main, /replace_config_ciphertext/);
  assert.match(main, /const MAGIC: &\[u8\] = b"age-encryption\.org\/v1\\n"/);
  assert.doesNotMatch(main, /const MAGIC: &\[u8\] = b"age-encryption\.org\/v1\\\\n"/);
  assert.match(main, /export_age_recipient/);
  assert.doesNotMatch(coordinator, /device_names|display_name.*device/i);
});

test('device route status is exposed as a shared coordinator helper', () => {
  assert.match(workspace, /import \{ checkDeviceRoutes \} from '\.\/coordinator-device-routes\.mjs'/);
  assert.match(routes, /export async function checkDeviceRoutes\(devices/);
  assert.match(coordinator, /assets\/coordinator-device-routes\.mjs/);
});
