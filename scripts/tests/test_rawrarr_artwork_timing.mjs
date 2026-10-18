import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import vm from 'node:vm';

const ui = await readFile(new URL('../../rust-arr/src/ui.html', import.meta.url), 'utf8');
const code = ui.slice(ui.indexOf('    const artworkRequestTimings'), ui.indexOf('    function renderArrLibrary'));
function fixture() {
  let callback;
  let pending = [];
  const context = vm.createContext({ PerformanceObserver: class {
    constructor(fn) { callback = fn; }
    observe(options) { assert.deepEqual({ ...options }, { type: 'resource', buffered: true }); }
    takeRecords() { const result = pending; pending = []; return result; }
  } });
  vm.runInContext(code, context);
  return {
    deliver: entries => callback({ getEntries: () => entries }),
    queue: entries => { pending = entries; },
    duration: url => context.artworkRequestDuration(url)
  };
}
const image = (name, requestStart, responseEnd) => ({ name, requestStart, responseEnd, initiatorType: 'img' });
test('each poster excludes time before its request, even far down a lazy library', () => {
  const f = fixture();
  f.deliver([image('first', 100, 125), image('scrolled', 60000, 60040)]);
  assert.equal(f.duration('first'), 25);
  assert.equal(f.duration('scrolled'), 40);
});
test('load callback drains pending observer records and uses refreshed request timing', () => {
  const f = fixture();
  f.deliver([image('poster', 100, 200)]);
  f.queue([image('poster', 8000, 8012)]);
  assert.equal(f.duration('poster'), 12);
});
test('missing and restricted timing stays unavailable; cached zero duration remains valid', () => {
  const f = fixture();
  f.deliver([image('restricted', 0, 100), image('cached', 123, 123)]);
  assert.equal(f.duration('missing'), null);
  assert.equal(f.duration('restricted'), null);
  assert.equal(f.duration('cached'), 0);
});
test('collection works beyond the default browser resource buffer', () => {
  const f = fixture();
  f.deliver(Array.from({ length: 1000 }, (_, i) => image(`poster-${i}`, 1000 + i, 1007 + i)));
  assert.equal(f.duration('poster-999'), 7);
});
