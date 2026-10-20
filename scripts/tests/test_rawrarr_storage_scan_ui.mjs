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
  assert.match(workspace, /api\/library\/scan\/schedule/);
  assert.match(workspace, /Weekly on/);
  assert.match(workspace, /Next scan/);
  assert.match(workspace, /setStorageScheduleForm\(form, schedule\)/);
  assert.match(workspace, /Unsaved changes/);
  assert.match(workspace, /Save schedule/);
  assert.match(workspace, /const trigger = state\.scan\.scheduled \? 'Scheduled scan'/);
  assert.match(workspace, /Weekly on \$\{schedule\.weekday\[0\]\.toUpperCase\(\)\}/);
  assert.match(workspace, /Storage scan is running/);
  assert.match(workspace, /Storage node returned an empty API response/);
  assert.match(main, /\.route\("\/api\/library\/scan\/missing", post\(request_storage_scan\)\)/);
  assert.match(main, /\.route\("\/api\/library\/scan\/status", get\(storage_scan_status\)\)/);
  assert.match(main, /get\(storage_scan_schedule\)\.put\(update_storage_scan_schedule\)/);
  assert.match(coordinator, /storage_scan_relay_allowed\(/);
  assert.match(coordinator, /role != "storage"/);
  assert.match(coordinator, /api\/library\/scan\/schedule/);
});
