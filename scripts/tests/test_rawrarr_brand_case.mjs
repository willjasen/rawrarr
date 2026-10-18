import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const displayedPages = ['age_setup.html', 'auth.html', 'ui.html'];
const runtimeMessages = ['devices.rs', 'main.rs', 'shared_logs.rs'];

test('displayed rawrarr name is lowercase in pages and runtime messages', async () => {
  for (const file of displayedPages) {
    const content = await readFile(new URL(`../../rust-arr/src/${file}`, import.meta.url), 'utf8');
    assert.doesNotMatch(content, /Rawrarr/, `${file} should display the brand in lowercase`);
  }

  for (const file of runtimeMessages) {
    const content = await readFile(new URL(`../../rust-arr/src/${file}`, import.meta.url), 'utf8');
    const quotedMessages = content.match(/"[^"\n]*Rawrarr[^"\n]*"/g) ?? [];
    assert.deepEqual(quotedMessages, [], `${file} should keep displayed messages lowercase`);
  }
});
