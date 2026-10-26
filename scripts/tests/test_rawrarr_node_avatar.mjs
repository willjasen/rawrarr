import assert from 'node:assert/strict';
import { createHash, webcrypto } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { deriveNodeAvatarSeed } from '../../rust-arr/src/coordinator-node-avatar.mjs';

const root = new URL('../../', import.meta.url);
const read = path => readFile(new URL(path, root), 'utf8');

test('node avatar seed hashes the authenticated node public recipient', async () => {
  const recipient = 'age1pq1publicnoderecipient123';
  const requests = [];
  const seed = await deriveNodeAvatarSeed('node/a', async (url, options) => {
    requests.push([url, options]);
    return { ok: true, async text() { return `  ${recipient}\n`; } };
  }, webcrypto);

  assert.equal(seed, createHash('sha256').update(recipient).digest('hex'));
  assert.equal(requests[0][0], '/nodes/node%2Fa/api/metadata/export/node-recipient');
  assert.equal(requests[0][1].cache, 'no-store');
  assert.equal(requests[0][1].headers.Accept, 'text/plain');
});

test('node avatar seed rejects unavailable or malformed public recipients', async () => {
  await assert.rejects(
    deriveNodeAvatarSeed('node-a', async () => ({ ok: false, status: 503 }), webcrypto),
    /recipient is unavailable/,
  );
  await assert.rejects(
    deriveNodeAvatarSeed('node-a', async () => ({ ok: true, async text() { return 'private-key-material'; } }), webcrypto),
    /invalid age recipient/,
  );
  await assert.rejects(deriveNodeAvatarSeed('', async () => {}, webcrypto), /Node ID is unavailable/);
});

test('subscriber recipient endpoint and coordinator device cards use the node identity avatar', async () => {
  const [main, coordinator, workspace, html] = await Promise.all([
    read('rust-arr/src/main.rs'),
    read('rust-arr/src/coordinator.rs'),
    read('rust-arr/src/coordinator-workspace.mjs'),
    read('rust-arr/src/coordinator_workspace.html'),
  ]);
  assert.match(main, /AgeVault::ensure_node_identity\(\)/);
  assert.match(main, /AgeVault::node_identity_recipient/);
  assert.match(main, /api\/metadata\/export\/node-recipient/);
  assert.match(coordinator, /api\/metadata\/export\/node-recipient/);
  assert.match(coordinator, /coordinator-node-avatar\.mjs/);
  assert.match(workspace, /deriveNodeAvatarSeed\(node\.nodeId\)/);
  assert.match(workspace, /device-avatar/);
  assert.match(workspace, /device-picture\/\$\{seed\}\/jxl/);
  assert.match(workspace, /device-picture\/\$\{seed\}\/png/);
  assert.doesNotMatch(workspace, /async function loadNodeAvatar\(node, pictureSource, image\) \{\s*if \(!node\.online \|\| !node\.relayAvailable\)/);
  assert.match(html, /\.device-avatar/);
});
