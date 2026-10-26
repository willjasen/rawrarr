import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import vm from 'node:vm';

const page = await readFile(new URL('../../rust-arr/src/age_setup.html', import.meta.url), 'utf8');
const mainPage = await readFile(new URL('../../rust-arr/src/ui.html', import.meta.url), 'utf8');
const server = await readFile(new URL('../../rust-arr/src/main.rs', import.meta.url), 'utf8');
const jxlLogo = await readFile(new URL('../../rust-arr/src/assets/rawrarr-logo.jxl', import.meta.url));
const publicRecipient = 'age1pq1public-test-recipient';
const privateKey = 'AGE-SECRET-KEY-PQ-1TESTPRIVATEDONOTPRINT';
const privateIdentity = `# created: test fixture\n# public key: ${publicRecipient}\n${privateKey}`;
const themeSeed = createHash('sha256').update(publicRecipient).digest('hex');

function createElement() {
  return {
    value: '', textContent: '', disabled: false, hidden: true, checked: false, open: false, files: [], children: [],
    handlers: new Map(), attributes: {}, dataset: {},
    focused: false,
    addEventListener(name, handler) { this.handlers.set(name, handler); },
    async click() { if (!this.disabled) await this.handlers.get('click')?.({ currentTarget: this }); },
    async input() { await this.handlers.get('input')?.({ currentTarget: this }); },
    async change() { await this.handlers.get('change')?.({ currentTarget: this }); },
    showModal() { this.open = true; },
    focus() { this.focused = true; },
    close() { this.open = false; },
    append(element) { this.children.push(element); },
    replaceChildren() { this.children = []; },
    setAttribute(name, value) { this.attributes[name] = value; }
  };
}

test('generated private age identity has a short preview and full value is exposed only through copy', async () => {
  assert.match(page, /<h1>unlock your rawrchive<\/h1>/);
  assert.match(page, /<source srcset="\/assets\/rawrarr-logo\.jxl" type="image\/jxl">/);
  assert.match(page, /<img class="brand-logo" src="\/assets\/rawrarr-logo\.png" alt="rawrarr"/);
  assert.match(mainPage, /<source srcset="\/assets\/rawrarr-logo\.jxl" type="image\/jxl">/);
  assert.match(mainPage, /<img class="brand-logo" src="\/assets\/rawrarr-logo\.png" alt="rawrarr"/);
  assert.match(server, /static_asset\("image\/jxl", ui_assets::bytes\("assets\/rawrarr-logo\.jxl", include_bytes!\("assets\/rawrarr-logo\.jxl"\)\)\.await\)/);
  assert.match(server, /\.route\("\/assets\/rawrarr-logo\.jxl", get\(rawrarr_logo_jxl\)\)/);
  assert.deepEqual([...jxlLogo.subarray(0, 2)], [0xff, 0x0a], 'logo asset is a JPEG XL codestream');
  assert.doesNotMatch(page, /<textarea\b/i);
  assert.match(page, /id="identity" type="password"/);
  assert.match(page, /<dialog id="identity-copy-dialog"/);
  assert.match(page, /<span>I copied it and saved the private ID somewhere safe\.<\/span>/);
  assert.match(page, /Copy private identity/);
  assert.match(page, /unlockButton\.textContent = generatedIdentity \? 'Continue' : 'Unlock vault'/);
  assert.match(page, /Create ID/);
  assert.match(page, /Public age recipient/);
  assert.match(page, /id="import-id" type="button">Import ID/);
  assert.match(page, /<dialog id="import-dialog"/);
  assert.match(page, /id="import-submit" type="button" disabled>Unlock vault/);
  assert.match(page, /Paste a post-quantum age private ID beginning with AGE-SECRET-KEY-PQ-1/);
  assert.doesNotMatch(page, /identity-file|choose-file|Load identity from file|type="file"/);
  assert.match(page, />Unlock vault<\/button>/);
  assert.doesNotMatch(page, />Close<\/button>/);
  assert.match(page, /rawrarr cannot recover this ID if it is lost/);
  assert.match(page, /Keep this ID on this device/);
  assert.match(page, /rememberOnDevice: document\.getElementById\('remember-on-device'\)\.checked/);
  assert.match(page, /\.backup-warning \{[^}]*color:#f3d778/);
  assert.doesNotMatch(page, />Import and unlock<\/button>/);

  const ids = ['identity', 'message', 'public-card', 'public-recipient',
    'private-preview-card', 'private-preview', 'fingerprint', 'copy', 'generate', 'unlock', 'import-id', 'import-submit', 'import-dialog', 'import-message', 'review-generated',
    'backup-confirmation', 'backup-confirmed', 'remember-on-device',
    'identity-copy-status', 'identity-copy-dialog',
    'identity-copy-instructions', 'copy-private-identity',
    'coordinator-passkey-section', 'add-coordinator-passkey'];
  const elements = Object.fromEntries(ids.map(id => [id, createElement()]));
  const rootStyles = new Map();
  const fetchCalls = [];
  const clipboard = [];
  const stored = new Map();
  let randomValue = 0;
  const document = {
    documentElement: { style: { setProperty(name, value) { rootStyles.set(name, value); } } },
    getElementById(id) { return elements[id]; },
    createElementNS() { return createElement(); }
  };
  const context = {
    document,
    navigator: { clipboard: { async writeText(value) { clipboard.push(value); } } },
    crypto: {
      getRandomValues(array) { array[0] = randomValue++; return array; },
      subtle: { async digest(_algorithm, bytes) {
        const input = Buffer.from(bytes);
        const digest = createHash('sha256').update(input).digest();
        return digest.buffer.slice(digest.byteOffset, digest.byteOffset + digest.byteLength);
      } }
    },
    TextEncoder,
    localStorage: { setItem(key, value) { stored.set(key, value); }, getItem(key) { return stored.get(key) || null; } },
    fetch: async (url, options = {}) => {
      fetchCalls.push({ url, options });
      if (url.endsWith('/generate')) return { ok: true, async json() { return { identity: privateIdentity, recipient: publicRecipient, themeSeed }; } };
      if (options.method === 'POST') return { ok: true, async json() { return { unlocked: true, themeSeed }; } };
      return { ok: true, async json() { return { unlocked: false, encryptedConfigExists: false }; } };
    },
    window: { location: { origin: 'http://localhost:7297', replaced: [], replace(path) { this.replaced.push(path); } }, addEventListener() {} }
  };
  const script = page.match(/<script>([\s\S]*?)<\/script>/i)?.[1];
  assert.ok(script, 'age setup page contains client script');
  vm.runInNewContext(script, context);

  await elements.generate.click();
  assert.equal(elements['identity-copy-dialog'].open, true, 'creating an ID opens the copy pop-up');
  assert.equal(rootStyles.get('--theme-hue'), String(Number.parseInt(themeSeed.slice(0, 8), 16) % 360));
  assert.equal(elements['backup-confirmation'].hidden, true, 'the copied checkbox waits until after the copy action');
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
  assert.equal(elements['identity-copy-dialog'].open, true, 'copy action opens the identity pop-up');
  await elements['copy-private-identity'].click();
  assert.deepEqual(clipboard, [privateKey]);
  assert.ok(!clipboard[0].includes(publicRecipient), 'copy excludes the public recipient header');
  assert.equal(elements['backup-confirmation'].hidden, false, 'successful copy reveals the acknowledgement checkbox');
  assert.equal(elements.unlock.disabled, true, 'copy alone does not acknowledge backup');
  assert.equal(elements.unlock.hidden, false, 'Continue is visible');
  const unlockCount = () => fetchCalls.filter(call => call.url === '/api/settings/age' && call.options.method === 'POST').length;
  await elements.unlock.click();
  assert.equal(unlockCount(), 0);
  elements['backup-confirmed'].checked = true;
  await elements['backup-confirmed'].change();
  assert.equal(elements.unlock.disabled, false, 'acknowledgement enables Continue without a character check');
  assert.equal(unlockCount(), 0, 'acknowledgement does not auto-submit');
  elements['backup-confirmed'].checked = false;
  await elements['backup-confirmed'].change();
  assert.equal(elements.unlock.disabled, true, 'removing acknowledgement disables Continue');
  elements['backup-confirmed'].checked = true;
  await elements['backup-confirmed'].change();
  await elements.unlock.click();
  assert.equal(unlockCount(), 1);
  assert.equal(stored.get('rawrarr.browser.age-id'), privateIdentity, 'successful signup vault setup remembers the age ID in this browser');
  assert.equal(elements['identity-copy-dialog'].open, false);
  assert.deepEqual(context.window.location.replaced, ['/']);
  assert.doesNotMatch(page, /identity-challenge-input|identity-backup-check/);

  await elements.generate.click();
  assert.equal(elements.unlock.disabled, true, 'new generation resets acknowledgement');
  await elements['import-id'].click();
  assert.equal(elements['import-dialog'].open, true, 'Import ID opens the paste dialog');
  elements.identity.value = privateKey;
  await elements.identity.input();
  assert.equal(elements['import-submit'].disabled, false, 'pasting a post-quantum ID enables import');
  await elements['import-submit'].click();
  assert.equal(unlockCount(), 2, 'imported ID unlocks directly without creating a new ID');
  const submitted = fetchCalls.filter(call => call.url === '/api/settings/age' && call.options.method === 'POST').at(-1);
  assert.equal(JSON.parse(submitted.options.body).identity, privateKey);
});
