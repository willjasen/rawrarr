import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import vm from 'node:vm';

const page = await readFile(new URL('../../rust-arr/src/age_setup.html', import.meta.url), 'utf8');
const mainPage = await readFile(new URL('../../rust-arr/src/ui.html', import.meta.url), 'utf8');
const privateIdentity = '# created: test fixture\nAGE-SECRET-KEY-PQ-1-TEST-PRIVATE-DO-NOT-PRINT';
const publicRecipient = 'age1pq1public-test-recipient';
const themeSeed = createHash('sha256').update(publicRecipient).digest('hex');

function createElement() {
  return {
    value: '', textContent: '', disabled: false, hidden: true, checked: false, files: [], children: [],
    handlers: new Map(), attributes: {},
    addEventListener(name, handler) { this.handlers.set(name, handler); },
    async click() { if (!this.disabled) await this.handlers.get('click')?.({ currentTarget: this }); },
    async input() { await this.handlers.get('input')?.({ currentTarget: this }); },
    async change() { await this.handlers.get('change')?.({ currentTarget: this }); },
    append(element) { this.children.push(element); },
    replaceChildren() { this.children = []; },
    setAttribute(name, value) { this.attributes[name] = value; }
  };
}

test('generated private age identity has a short preview and full value is exposed only through copy', async () => {
  assert.match(page, /<h1>unlock your rawrchive<\/h1>/);
  assert.doesNotMatch(page, /<textarea\b/i);
  assert.match(page, /id="identity" type="password"/);
  assert.match(page, /Copy private identity/);
  assert.match(page, /Public age recipient/);
  assert.match(page, /Paste existing private identity/);
  assert.doesNotMatch(page, /identity-file|choose-file|Load identity from file|type="file"/);
  assert.match(page, />Unlock vault<\/button>/);
  assert.match(page, /To use a saved key, paste it into the masked field/);
  assert.match(page, /I have saved my private age identity somewhere safe and can retrieve it later/);
  assert.match(page, /Rawrarr cannot recover this identity if it is lost/);
  assert.doesNotMatch(page, />Import and unlock<\/button>/);

  const ids = ['identity', 'message', 'public-card', 'public-recipient',
    'private-preview-card', 'private-preview', 'fingerprint', 'copy', 'generate', 'unlock',
    'backup-confirmation', 'backup-confirmed'];
  const elements = Object.fromEntries(ids.map(id => [id, createElement()]));
  const rootStyles = new Map();
  const fetchCalls = [];
  const clipboard = [];
  const document = {
    documentElement: { style: { setProperty(name, value) { rootStyles.set(name, value); } } },
    getElementById(id) { return elements[id]; },
    createElementNS() { return createElement(); }
  };
  const context = {
    document,
    navigator: { clipboard: { async writeText(value) { clipboard.push(value); } } },
    crypto: { subtle: { async digest(_algorithm, bytes) {
      const input = Buffer.from(bytes);
      const digest = createHash('sha256').update(input).digest();
      return digest.buffer.slice(digest.byteOffset, digest.byteOffset + digest.byteLength);
    } } },
    TextEncoder,
    fetch: async (url, options = {}) => {
      fetchCalls.push({ url, options });
      if (url.endsWith('/generate')) return { ok: true, async json() { return { identity: privateIdentity, recipient: publicRecipient, themeSeed }; } };
      if (options.method === 'POST') return { ok: true, async json() { return { unlocked: true, themeSeed }; } };
      return { ok: true, async json() { return { unlocked: false, encryptedConfigExists: false }; } };
    },
    window: { location: { replace() {} }, addEventListener() {} }
  };
  const script = page.match(/<script>([\s\S]*?)<\/script>/i)?.[1];
  assert.ok(script, 'age setup page contains client script');
  vm.runInNewContext(script, context);

  await elements.generate.click();
  assert.equal(rootStyles.get('--theme-hue'), String(Number.parseInt(themeSeed.slice(0, 8), 16) % 360));
  assert.equal(elements['backup-confirmation'].hidden, false);
  assert.equal(elements.unlock.disabled, true, 'a generated identity cannot unlock before backup confirmation');
  assert.equal(elements.identity.value, '');
  assert.equal(elements['public-recipient'].textContent, 'age1pq1publi…ecipient');
  assert.notEqual(elements['public-recipient'].textContent, publicRecipient);
  assert.equal(elements['private-preview-card'].hidden, false);
  assert.equal(elements['private-preview'].textContent, 'AGE-SECRET-K…PRINT');
  assert.ok(elements.fingerprint.children.length > 0, 'public recipient has a visual fingerprint');
  const fingerprintColors = new Set(elements.fingerprint.children.map(square => square.attributes.fill));
  assert.ok(fingerprintColors.size > 1, 'public recipient fingerprint uses multiple colors');
  assert.ok([...fingerprintColors].every(color => ['#a6e3a1', '#77d4c8', '#8ab4f8', '#f2c879', '#d59cf6'].includes(color)), 'fingerprint colors come from the accessible palette');
  assert.ok(!JSON.stringify(elements).includes(privateIdentity), 'private identity never appears in page elements');
  assert.ok(!JSON.stringify(elements).includes(publicRecipient), 'full public recipient never appears in page elements');

  await elements.copy.click();
  assert.deepEqual(clipboard, [privateIdentity]);
  assert.ok(!JSON.stringify(elements).includes(privateIdentity), 'copy does not render the private identity');

  const beforeUnconfirmedUnlock = fetchCalls.filter(call => call.url === '/api/settings/age' && call.options.method === 'POST').length;
  await elements.unlock.click();
  assert.equal(fetchCalls.filter(call => call.url === '/api/settings/age' && call.options.method === 'POST').length, beforeUnconfirmedUnlock);
  elements['backup-confirmed'].checked = true;
  await elements['backup-confirmed'].change();
  assert.equal(elements.unlock.disabled, false);
  await elements.unlock.click();
  const unlock = fetchCalls.find(call => call.url === '/api/settings/age' && call.options.method === 'POST');
  assert.equal(JSON.parse(unlock.options.body).identity, privateIdentity);
  assert.equal(elements.identity.value, '');

  const previousUnlockCount = fetchCalls.filter(call => call.url === '/api/settings/age' && call.options.method === 'POST').length;
  elements.identity.value = privateIdentity;
  await elements.identity.input();
  assert.equal(elements.identity.value, privateIdentity, 'a pasted identity remains in the masked password field');
  assert.equal(elements['backup-confirmed'].checked, false, 'pasting an identity resets the backup confirmation');
  assert.equal(elements.unlock.disabled, true, 'a pasted identity requires a fresh backup confirmation');
  assert.equal(
    fetchCalls.filter(call => call.url === '/api/settings/age' && call.options.method === 'POST').length,
    previousUnlockCount,
    'pasting an identity does not unlock automatically'
  );
  elements['backup-confirmed'].checked = true;
  await elements['backup-confirmed'].change();
  await elements.unlock.click();
  assert.equal(
    fetchCalls.filter(call => call.url === '/api/settings/age' && call.options.method === 'POST').length,
    previousUnlockCount + 1,
    'unlocking after paste submits the identity'
  );
  assert.equal(rootStyles.get('--theme-hue'), String(Number.parseInt(themeSeed.slice(0, 8), 16) % 360));

  assert.match(mainPage, /fetch\('\/api\/settings\/age'/);
  assert.match(mainPage, /applyIdentityTheme\(age\.themeSeed\)/);
  assert.match(mainPage, /--theme-hue/);
  assert.doesNotMatch(mainPage, /themeSeed[^\n]*recipient/);
});
