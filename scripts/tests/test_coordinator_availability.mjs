import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';
import { createCoordinatorAvailabilityMonitor } from '../../rust-arr/src/coordinator-availability.mjs';

const root = new URL('../../', import.meta.url);
const read = path => readFile(new URL(path, root), 'utf8');

function monitorWith(results, overrides = {}) {
  const listeners = new Map();
  const windowRef = {
    setTimeout: () => 1,
    clearTimeout: () => {},
    setInterval: callback => { windowRef.interval = callback; return 2; },
    clearInterval: () => {},
    addEventListener: (event, callback) => listeners.set(`window:${event}`, callback),
    removeEventListener: event => listeners.delete(`window:${event}`),
  };
  const documentRef = {
    hidden: false,
    addEventListener: (event, callback) => listeners.set(`document:${event}`, callback),
    removeEventListener: event => listeners.delete(`document:${event}`),
  };
  const events = [];
  let call = 0;
  const watcher = createCoordinatorAvailabilityMonitor({
    fetchImpl: async (url, options) => {
      assert.equal(url, '/health');
      assert.equal(options.cache, 'no-store');
      const result = results[call++];
      if (result instanceof Error) throw result;
      return result;
    },
    windowRef,
    documentRef,
    onUnavailable: error => events.push(['offline', error.message]),
    onAvailable: () => events.push(['online']),
    ...overrides,
  });
  return { watcher, events, listeners, windowRef, documentRef };
}

test('waits for repeated failures and stable recovery before changing availability', async () => {
  const { watcher, events } = monitorWith([
    new Error('request failed'),
    new Error('request failed'),
    { ok: true, status: 200 },
    new Error('one transient failure'),
    { ok: true, status: 200 },
    { ok: true, status: 200 },
  ]);

  await watcher.check();
  assert.deepEqual(events, []);
  await watcher.check();
  assert.deepEqual(events, [['offline', 'request failed']]);
  assert.equal(watcher.unavailable, true);
  await watcher.check();
  await watcher.check();
  assert.deepEqual(events, [['offline', 'request failed']]);
  await watcher.check();
  await watcher.check();
  assert.deepEqual(events, [['offline', 'request failed'], ['online']]);
  assert.equal(watcher.unavailable, false);
});

test('health polling checks on startup, network recovery, and visible-tab return', async () => {
  const { watcher, listeners, windowRef, documentRef } = monitorWith([
    { ok: true, status: 200 },
    { ok: true, status: 200 },
    { ok: true, status: 200 },
  ]);
  watcher.start();
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(typeof windowRef.interval, 'function');
  listeners.get('window:online')();
  await new Promise(resolve => setImmediate(resolve));
  documentRef.hidden = true;
  listeners.get('document:visibilitychange')();
  assert.equal(watcher.unavailable, false);
  documentRef.hidden = false;
  listeners.get('document:visibilitychange')();
  await new Promise(resolve => setImmediate(resolve));
  watcher.stop();
  assert.equal(listeners.size, 0);
});

test('coordinator page includes the modal pending view and serves its monitor module', async () => {
  const [workspace, script, routes] = await Promise.all([
    read('rust-arr/src/coordinator_workspace.html'),
    read('rust-arr/src/coordinator-workspace.mjs'),
    read('rust-arr/src/coordinator.rs'),
  ]);
  assert.match(workspace, /<dialog id="coordinator-pending-dialog"[\s\S]*?Roarlay may be updating or restarting/);
  assert.match(script, /createCoordinatorAvailabilityMonitor/);
  assert.match(script, /coordinatorPendingDialog\.showModal\(\)/);
  assert.match(script, /refreshWorkspaceAfterCoordinatorReconnect/);
  assert.match(routes, /"\/assets\/coordinator-availability\.mjs", get\(coordinator_availability_script\)/);
});
