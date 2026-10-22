import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const read = path => readFileSync(new URL(path, import.meta.url), 'utf8');

test('only the coordinator launches the GitHub UI updater', () => {
  const entrypoint = read('../../rust-arr/container-entrypoint.sh');
  assert.ok(entrypoint.includes('\"${RAWARR_ROLE:-}\" = \"coordinator\"'));
});

test('the coordinator exposes authenticated server-owned publisher status', () => {
  const routes = read('../../rust-arr/src/coordinator.rs');
  const status = read('../../rust-arr/src/ui_assets.rs');
  assert.match(routes, /route\("\/api\/workspace\/ui-signature", get\(workspace_ui_signature\)\)/);
  assert.match(routes, /async fn workspace_ui_signature[\s\S]*authenticated_user_id[\s\S]*signature_status/);
  assert.match(status, /bundle\.signature-status/);
  assert.match(status, /Some\("rejected"\)/);
});

test('the signed-in profile displays the coordinator verification state', () => {
  const html = read('../../rust-arr/src/coordinator_workspace.html');
  const script = read('../../rust-arr/src/coordinator-workspace.mjs');
  assert.match(html, /id="sidebar-ui-signature"/);
  assert.match(script, /fetch\('\/api\/workspace\/ui-signature'/);
  assert.match(script, /'UI update rejected'/);
  assert.match(script, /'Signed'/);
});
