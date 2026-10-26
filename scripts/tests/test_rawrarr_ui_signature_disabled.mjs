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
  const assets = read('../../rust-arr/src/ui_assets.rs');
  assert.doesNotMatch(coordinator, /api\/workspace\/ui-signature|workspace_ui_signature/);
  assert.doesNotMatch(assets, /SignatureStatus|bundle\.signature-status/);
});

test('runtime image and updater no longer require a publisher key or OpenSSL CLI', () => {
  const dockerfile = read('../../rust-arr/Dockerfile');
  const updater = read('../../rust-arr/update_ui_bundle.py');
  assert.doesNotMatch(dockerfile, /ui-publisher-public-key|\bopenssl\b/);
  assert.doesNotMatch(updater, /PINNED_PUBLIC_KEY_FILE|pkeyutl|bundle\.zip\.sig/);
});

test('device signing endpoints and checks remain in place', () => {
  const coordinator = read('../../rust-arr/src/coordinator.rs');
  assert.match(coordinator, /api\/nodes\/signing-certificate/);
  assert.match(coordinator, /device_signing::verify/);
});
