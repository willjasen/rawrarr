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
    handlers: new Map(), attributes: {},
    addEventListener(name, handler) { this.handlers.set(name, handler); },
    async click() { if (!this.disabled) await this.handlers.get('click')?.({ currentTarget: this }); },
    async input() { await this.handlers.get('input')?.({ currentTarget: this }); },
    async change() { await this.handlers.get('change')?.({ currentTarget: this }); },
    showModal() { this.open = true; },
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
  assert.match(server, /static_asset\("image\/jxl", include_bytes!\("assets\/rawrarr-logo\.jxl"\)\)/);
  assert.match(server, /\.route\("\/assets\/rawrarr-logo\.jxl", get\(rawrarr_logo_jxl\)\)/);
  assert.deepEqual([...jxlLogo.subarray(0, 2)], [0xff, 0x0a], 'logo asset is a JPEG XL codestream');
  assert.doesNotMatch(page, /<textarea\b/i);
  assert.match(page, /id="identity" type="password"/);
  assert.match(page, /<dialog id="identity-copy-dialog"/);
  assert.match(page, /<span>I copied it and saved the private ID somewhere safe\.<\/span>/);
  assert.match(page, /Copy private identity/);
  assert.match(page, /Enter those three characters to continue/);
  assert.match(page, /Create ID/);
  assert.match(page, /Public age recipient/);
  assert.match(page, /Paste existing private identity/);
  assert.doesNotMatch(page, /identity-file|choose-file|Load identity from file|type="file"/);
  assert.match(page, />Unlock vault<\/button>/);
  assert.match(page, /rawrarr cannot recover this ID if it is lost/);
  assert.doesNotMatch(page, />Import and unlock<\/button>/);

  const ids = ['identity', 'message', 'public-card', 'public-recipient',
    'private-preview-card', 'private-preview', 'fingerprint', 'copy', 'generate', 'unlock',
    'backup-confirmation', 'backup-confirmed', 'identity-challenge',
    'identity-challenge-prompt', 'identity-challenge-input', 'identity-copy-dialog',
    'identity-copy-instructions', 'copy-private-identity', 'close-identity-copy'];
  const elements = Object.fromEntries(ids.map(id => [id, createElement()]));
  const rootStyles = new Map();
  const fetchCalls = [];
  const clipboard = [];
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
    fetch: async (url, options = {}) => {
      fetchCalls.push({ url, options });
      if (url.endsWith('/generate')) return { ok: true, async json() { return { identity: privateIdentity, recipient: publicRecipient, themeSeed }; } };
      if (options.method === 'POST') return { ok: true, async json() { return { unlocked: true, themeSeed }; } };
      return { ok: true, async json() { return { unlocked: false, encryptedConfigExists: false }; } };
    },
    window: { location: { replaced: [], replace(path) { this.replaced.push(path); } }, addEventListener() {} }
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
  assert.equal(elements['identity-challenge'].hidden, true, 'character challenge waits for acknowledgement');
  elements['backup-confirmed'].checked = true;
  await elements['backup-confirmed'].change();
  assert.equal(elements['identity-challenge'].hidden, false, 'acknowledgement reveals the character challenge');
  const challengePositions = [...elements['identity-challenge-prompt'].textContent.matchAll(/\d+/g)]
    .map(match => Number(match[0]));
  assert.equal(challengePositions.length, 3);
  assert.ok(challengePositions.every(position => position > 'AGE-SECRET-KEY-PQ-1'.length), 'challenge positions skip the fixed prefix');
  assert.ok(!JSON.stringify(elements).includes(privateIdentity), 'copy does not render the private identity');

  const beforeUnconfirmedUnlock = fetchCalls.filter(call => call.url === '/api/settings/age' && call.options.method === 'POST').length;
  await elements.unlock.click();
  assert.equal(fetchCalls.filter(call => call.url === '/api/settings/age' && call.options.method === 'POST').length, beforeUnconfirmedUnlock);
  assert.equal(elements.unlock.hidden, true, 'generated identity continues automatically after the challenge');
  elements['identity-challenge-input'].value = 'XXX';
  await elements['identity-challenge-input'].input();
  assert.equal(elements.unlock.disabled, true, 'incorrect characters keep unlock disabled');
  elements['identity-challenge-input'].value = challengePositions
    .map(position => privateKey[position - 1]).join('');
  await elements['identity-challenge-input'].input();
  assert.equal(fetchCalls.filter(call => call.url === '/api/settings/age' && call.options.method === 'POST').length, beforeUnconfirmedUnlock + 1, 'matching characters unlock the generated identity automatically');
  assert.equal(elements['identity-copy-dialog'].open, false, 'successful unlock closes the pop-up');
  assert.deepEqual(context.window.location.replaced, ['/'], 'successful unlock navigates to the main site');
  await elements['close-identity-copy'].click();
  await elements.generate.click();
  assert.equal(elements['identity-challenge'].hidden, true, 'creating a new identity clears the previous challenge');
  assert.equal(elements.unlock.disabled, true, 'creating a new identity resets unlock confirmation');
  await elements.copy.click();
  await elements['copy-private-identity'].click();
  assert.deepEqual(clipboard, [privateKey, privateKey]);
  elements['backup-confirmed'].checked = true;
  await elements['backup-confirmed'].change();
  const regeneratedChallengePositions = [...elements['identity-challenge-prompt'].textContent.matchAll(/\d+/g)]
    .map(match => Number(match[0]));
  elements['identity-challenge-input'].value = regeneratedChallengePositions
    .map(position => privateKey[position - 1]).join('');
  await elements['identity-challenge-input'].input();
  assert.equal(fetchCalls.filter(call => call.url === '/api/settings/age' && call.options.method === 'POST').length, 2, 'regenerated ID also unlocks after the challenge');
  const unlock = fetchCalls.filter(call => call.url === '/api/settings/age' && call.options.method === 'POST').at(-1);
  assert.equal(JSON.parse(unlock.options.body).identity, privateIdentity);
  assert.equal(elements.identity.value, '');

  const previousUnlockCount = fetchCalls.filter(call => call.url === '/api/settings/age' && call.options.method === 'POST').length;
  elements.identity.value = privateIdentity.replace(/\n/g, ' ');
  await elements.identity.input();
  assert.equal(elements.identity.value, privateIdentity.replace(/\n/g, ' '), 'pasted age-file text remains masked in the password field');
  await elements.copy.click();
  await elements['copy-private-identity'].click();
  assert.deepEqual(clipboard, [privateKey, privateKey, privateKey], 'copying a pasted age file extracts only its private key');
  assert.equal(elements['backup-confirmed'].checked, false, 'pasting an identity resets the backup confirmation');
  assert.equal(elements.unlock.disabled, true, 'a pasted identity requires a fresh backup confirmation');
  assert.equal(
    fetchCalls.filter(call => call.url === '/api/settings/age' && call.options.method === 'POST').length,
    previousUnlockCount,
    'pasting an identity does not unlock automatically'
  );
  elements['backup-confirmed'].checked = true;
  await elements['backup-confirmed'].change();
  const pastedChallengePositions = [...elements['identity-challenge-prompt'].textContent.matchAll(/\d+/g)]
    .map(match => Number(match[0]));
  assert.equal(elements.unlock.disabled, true, 'pasted identity also requires the character challenge');
  elements['identity-challenge-input'].value = pastedChallengePositions
    .map(position => privateKey[position - 1]).join('');
  await elements['identity-challenge-input'].input();
  await elements.unlock.click();
  assert.equal(
    fetchCalls.filter(call => call.url === '/api/settings/age' && call.options.method === 'POST').length,
    previousUnlockCount + 1,
    'unlocking after paste submits the identity'
  );
  const pastedUnlock = fetchCalls.filter(call => call.url === '/api/settings/age' && call.options.method === 'POST').at(-1);
  assert.equal(JSON.parse(pastedUnlock.options.body).identity, privateKey, 'flattened age-file paste submits only its private-key line');
  assert.equal(rootStyles.get('--theme-hue'), String(Number.parseInt(themeSeed.slice(0, 8), 16) % 360));

  assert.match(mainPage, /fetch\('\/api\/settings\/age'/);
  assert.match(mainPage, /applyIdentityTheme\(age\.themeSeed\)/);
  assert.match(mainPage, /--theme-hue/);
  assert.doesNotMatch(mainPage, /themeSeed[^\n]*recipient/);
});
