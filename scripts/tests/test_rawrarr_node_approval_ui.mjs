import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';

const ui = await readFile(new URL('../../rust-arr/src/ui.html', import.meta.url), 'utf8');
const coordinatorUi = await readFile(new URL('../../rust-arr/src/coordinator_workspace.html', import.meta.url), 'utf8');
const coordinatorWorkspace = await readFile(new URL('../../rust-arr/src/coordinator-workspace.mjs', import.meta.url), 'utf8');
const main = await readFile(new URL('../../rust-arr/src/main.rs', import.meta.url), 'utf8');
const coordinator = await readFile(new URL('../../rust-arr/src/coordinator.rs', import.meta.url), 'utf8');

test('signed-in workspace displays and refreshes pending node approvals', () => {
  assert.match(ui, /Pending node approvals/);
  assert.match(ui, /id="pending-node-list"/);
  assert.ok(ui.indexOf('class="pending-approvals"') < ui.indexOf('id="device-list"'), 'pending approvals should appear before connected nodes');
  assert.match(ui, /\.pending-approvals\.has-pending\s*\{[^}]*background:/);
  assert.match(ui, /id="pending-approvals-badge"[^>]*hidden>Needs review/);
  assert.match(ui, /approvalsSection\.classList\.toggle\('has-pending', nodes\.length > 0\)/);
  assert.match(ui, /approvalBadge\.hidden = nodes\.length === 0/);
  assert.match(ui, /fetch\('\/api\/nodes\/pending'/);
  assert.match(ui, /setInterval\(loadPendingNodes, 10000\)/);
  assert.match(ui, /New nodes are associated with your signed-in account by roarlay/);
  assert.match(ui, /ownerDisplayName/);
  assert.match(ui, /Add \$\{node\.displayName\} to \$\{node\.ownerDisplayName\}'s account/);
  assert.match(ui, /fetch\('\/api\/nodes\/approve'/);
  assert.match(main, /\.route\("\/api\/nodes\/pending", get\(node_auth::proxy\)\)/);
  assert.match(main, /\.route\("\/api\/nodes\/approve", post\(node_auth::proxy\)\)/);
  assert.match(main, /\.route\("\/api\/nodes\/enroll", post\(node_auth::enrollment_proxy\)\)/);
  assert.match(main, /"\/api\/nodes\/enroll\/status",\s*post\(node_auth::enrollment_proxy\)/);
});

test('coordinator workspace puts pending approvals first and highlights action needed', () => {
  assert.ok(coordinatorUi.indexOf('id="pending"') < coordinatorUi.indexOf('id="status"'), 'pending approvals should appear before connected nodes');
  assert.match(coordinatorUi, /#pending\.has-pending\s*\{[^}]*background:/);
  assert.match(coordinatorWorkspace, /pendingPanel\.classList\.toggle\('has-pending', pending\.length > 0\)/);
  assert.match(coordinatorWorkspace, /badge\.textContent = 'Needs review'/);
});

test('approved subscriber identity retains its requested name and localhost browser route', () => {
  assert.match(coordinator, /COALESCE\(e\.display_name, n\.id\) AS display_name/);
  assert.match(coordinator, /host_str\(\) != Some\("localhost"\) \|\| parsed\.port\(\) != Some\(7297\)[\s\S]*parsed\.set_port\(Some\(7298\)\)/);
  assert.match(coordinatorWorkspace, /deviceNames\[node\.nodeId\] \|\| node\.displayName \|\| node\.nodeId/);
});

test('the coordinator assigns enrollment to the signed-in account and filters approvals by owner', () => {
  assert.match(coordinator, /SELECT id FROM rawrarr_users ORDER BY created_at/);
  assert.match(coordinator, /e\.user_id=\$1/);
  assert.match(coordinator, /request_id=\$1 AND user_id=\$2/);
  assert.match(coordinator, /owner_display_name/);
  assert.match(coordinator, /rawrarr_nodes\(id, role, token_sha256, origin, rp_id, user_id\)/);
});

test('metadata enrollment pass-through preserves only a new node bearer token', async () => {
  const auth = await readFile(new URL('../../rust-arr/src/node_auth.rs', import.meta.url), 'utf8');
  assert.match(auth, /matches!\(path, "\/api\/nodes\/enroll" \| "\/api\/nodes\/enroll\/status"\)/);
  assert.match(auth, /\.bearer_auth\(token\)/);
  assert.match(auth, /token\.len\(\) != 64/);
  assert.match(auth, /body\.len\(\) > 64 \* 1024/);
  assert.doesNotMatch(auth, /ageIdentity|age_identity/);
});
