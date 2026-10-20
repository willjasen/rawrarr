import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';
import { runInNewContext } from 'node:vm';

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

function scheduleHarness(state) {
  const controls = {
    frequency: { value: '', parentElement: {} },
    time: { value: '', parentElement: {} },
    weekday: { value: '', parentElement: {} },
    timezone: {},
    save: {},
  };
  const form = {
    dataset: {},
    querySelector(selector) {
      if (selector === '.storage-scan-schedule-timezone') return controls.timezone;
      return controls[selector.match(/name="(\w+)"/)[1]];
    },
    querySelectorAll() { return [controls.frequency, controls.time, controls.weekday, controls.save]; },
  };
  const drafts = new Map();
  const requests = [];
  const busy = [];
  const api = runInNewContext(
    workspace.slice(workspace.indexOf('function formatStorageScanTime('), workspace.indexOf('async function scanStorageNode('))
      + '\n({ refreshStorageSchedule, saveStorageSchedule });',
    {
      Intl, Date,
      storageScheduleDrafts: drafts,
      storageScanRequest: async (...args) => {
        requests.push(args);
        return { ok: true, status: 200, state, text: async () => JSON.stringify(state) };
      },
      storageScanJson: async response => response.state,
      setStorageScanBusy: (...args) => busy.push(args),
    },
  );
  return { ...api, controls, form, drafts, requests, busy };
}

test('schedule refresh restores unsaved edits and shows scheduled progress', async () => {
  const state = {
    schedule: { frequency: 'daily', time: '04:00', timeZone: 'America/New_York', weekday: null },
    nextRunAt: '2026-10-20T08:00:00Z',
    scan: { active: true, scheduled: true, filesCompleted: 3, filesTotal: 30 },
  };
  const harness = scheduleHarness(state);
  const node = { nodeId: 'mini', online: true };
  harness.drafts.set('mini', { ...state.schedule, frequency: 'weekly', weekday: 'monday', time: '05:30' });
  const summary = {};
  const status = {};
  await harness.refreshStorageSchedule(node, harness.form, summary, status);
  assert.equal(harness.controls.frequency.value, 'weekly');
  assert.equal(harness.controls.weekday.value, 'monday');
  assert.equal(harness.controls.time.value, '05:30');
  assert.equal(harness.controls.weekday.parentElement.hidden, false);
  assert.match(summary.textContent, /Unsaved changes.*Current schedule: Daily.*Next scan/);
  assert.match(status.textContent, /Scheduled scan is running.*3 of 30/);
  assert.equal(harness.busy[0][1], true);
});

test('saving a weekly schedule sends PUT and clears its draft only after success', async () => {
  const schedule = { frequency: 'weekly', time: '05:30', timeZone: 'America/New_York', weekday: 'monday' };
  const state = { schedule, nextRunAt: '2026-10-26T09:30:00Z', scan: { active: false } };
  const harness = scheduleHarness(state);
  const node = { nodeId: 'mini', online: true };
  await harness.refreshStorageSchedule(node, harness.form, {}, {});
  harness.drafts.set('mini', schedule);
  const summary = {};
  await harness.saveStorageSchedule(node, harness.form, summary, harness.controls.save, {});
  const request = harness.requests.at(-1);
  assert.equal(request[1], '/api/library/scan/schedule');
  assert.equal(request[3], 'PUT');
  assert.deepEqual(JSON.parse(JSON.stringify(request[2])), schedule);
  assert.equal(harness.drafts.has('mini'), false);
  assert.match(summary.textContent, /Schedule saved.*Next scan/);
  assert.equal(harness.controls.save.disabled, false);
});
