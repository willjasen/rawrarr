import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';

const root = new URL('../../', import.meta.url);
const [workspace, coordinator, main] = await Promise.all([
  readFile(new URL('rust-arr/src/coordinator-workspace.mjs', root), 'utf8'),
  readFile(new URL('rust-arr/src/coordinator.rs', root), 'utf8'),
  readFile(new URL('rust-arr/src/main.rs', root), 'utf8'),
]);

test('Devices offers owner-routed storage scans with a bounded 1–2 GB movie sample', () => {
  assert.match(workspace, /Scan, hash & ffprobe media/);
  assert.match(workspace, /Scan, hash & ffprobe 3 movies \(1–2 GB\)/);
  assert.match(workspace, /minimumSizeBytes: 1_000_000_000/);
  assert.match(workspace, /maximumSizeBytes: 2_000_000_000/);
  assert.match(workspace, /maxFiles: 3/);
  assert.match(workspace, /api\/library\/scan\/missing/);
  assert.match(workspace, /api\/library\/scan\/status/);
  assert.match(workspace, /Storage scan is running/);
  assert.match(workspace, /Storage node returned an empty scan response/);
  assert.match(main, /\.route\("\/api\/library\/scan\/missing", post\(request_storage_scan\)\)/);
  assert.match(main, /\.route\("\/api\/library\/scan\/status", get\(storage_scan_status\)\)/);
  assert.match(coordinator, /storage_scan_relay_allowed\(/);
  assert.match(coordinator, /role != "storage"/);
});
