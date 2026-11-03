import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const read = path => readFileSync(new URL(path, import.meta.url), 'utf8');

test('workspace no longer presents an unverified UI signature warning', () => {
  const html = read('../../rust-arr/src/coordinator_workspace.html');
  const script = read('../../rust-arr/src/coordinator-workspace.mjs');
  assert.doesNotMatch(html, /sidebar-ui-signature|UI signature unverified/);
  assert.doesNotMatch(script, /api\/workspace\/ui-signature|UI update rejected/);
});

test('coordinator no longer exposes the obsolete UI publisher status endpoint', () => {
  const coordinator = read('../../rust-arr/src/coordinator.rs');
  assert.doesNotMatch(coordinator, /api\/workspace\/ui-signature|workspace_ui_signature/);
});

test('updater no longer requires a publisher key, signature asset, or OpenSSL CLI', () => {
  const updater = read('../../rust-arr/update_ui_bundle.py');
  assert.doesNotMatch(updater, /PINNED_PUBLIC_KEY_FILE|pkeyutl|bundle\.zip\.sig/);
});

test('release workflow no longer requires a UI signing secret or creates signature assets', () => {
  const workflow = read('../../.github/workflows/tests.yml');
  assert.doesNotMatch(workflow, /RAWRARR_UI_SIGNING_KEY|ui-bundle\.zip\.sig|name: Sign UI bundle/);
  assert.match(workflow, /ui publisher-signature generation and verification are temporarily disabled/i);
});

test('device signing endpoints and checks remain in place', () => {
  const coordinator = read('../../rust-arr/src/coordinator.rs');
  assert.match(coordinator, /api\/nodes\/signing-certificate/);
  assert.match(coordinator, /device_signing::verify/);
});
