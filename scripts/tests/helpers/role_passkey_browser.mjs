// Interactive owner-side bootstrap for the disposable role deployment test.
// Private age/root material remains in this browser process and is never printed.
import assert from 'node:assert/strict';
import readline from 'node:readline';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';
import { Decrypter, Encrypter, armor, generateHybridIdentity, identityToRecipient } from '/opt/rawrarr-web-export/node_modules/age-encryption/dist/index.js';
import { ml_dsa65 } from '/opt/rawrarr-web-export/node_modules/@noble/post-quantum/ml-dsa.js';
import { sha256 } from '/opt/rawrarr-web-export/node_modules/@noble/hashes/sha2.js';

const encoder = new TextEncoder();
function toBase64(bytes) {
  let binary = '';
  for (let offset = 0; offset < bytes.length; offset += 0x8000) binary += String.fromCharCode(...bytes.subarray(offset, offset + 0x8000));
  return btoa(binary);
}
function fromBase64(value) { return Uint8Array.from(atob(value), character => character.charCodeAt(0)); }
function canonicalJson(value) {
  if (value === null || typeof value !== 'object') return JSON.stringify(value);
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(',')}]`;
  return `{${Object.keys(value).sort().map(key => `${JSON.stringify(key)}:${canonicalJson(value[key])}`).join(',')}}`;
}

const origin = 'http://localhost:8081';
const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
try {
  const context = await browser.newContext();
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  const page = await context.newPage();
  const cdp = await context.newCDPSession(page);
  await cdp.send('WebAuthn.enable');
  await cdp.send('WebAuthn.addVirtualAuthenticator', { options: { protocol: 'ctap2', transport: 'internal', hasResidentKey: true, hasUserVerification: true, isUserVerified: true, automaticPresenceSimulation: true } });

  await page.goto(`${origin}/auth`);
  await page.locator('#create-passkey').click();
  await page.waitForURL('**/setup/node', { timeout: 20000 });
  await page.locator('#create-age').click();
  await page.locator('#copy-age').click();
  const ageIdentity = await page.evaluate(() => navigator.clipboard.readText());
  assert.match(ageIdentity, /^AGE-SECRET-KEY-PQ-1/);
  await page.locator('#age-backed-up').check();
  await page.locator('#create-root').click();
  await page.locator('#root-fingerprint').getByText(/Instance root fingerprint:/).waitFor({ timeout: 30000 });
  const rootFingerprint = (await page.locator('#root-fingerprint').innerText()).match(/[0-9a-f]{64}/i)?.[0];
  assert.ok(rootFingerprint);
  await page.locator('#copy-root').click();
  const rootBackup = await page.evaluate(() => navigator.clipboard.readText());
  assert.match(rootBackup, /^-----BEGIN AGE ENCRYPTED FILE-----/);
  console.log(`READY ${rootFingerprint}`);

  const lines = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
  for await (const command of lines) {
    if (command === 'quit') break;
    const grantMatch = /^(grant|wrong-target) ([a-z0-9-]+) ([a-f0-9]{64})$/.exec(command);
    if (grantMatch) {
      const [_, action, nodeId, sessionToken] = grantMatch;
      const recipientResponse = await fetch('http://127.0.0.1:8080/api/metadata/export/node-recipient', {
        cache: 'no-store', headers: { Cookie: `rawrarr_session=${sessionToken}` },
      });
      if (!recipientResponse.ok) throw new Error('could not read the target fossil recipient');
      const deviceRecipient = (await recipientResponse.text()).trim();
      let appIdentity = await generateHybridIdentity();
      const appRecipient = await identityToRecipient(appIdentity);
      const appInstanceId = 'a762f7d8-cad2-41e2-a2ab-3e79e99181da';
      const envelopeWriter = new Encrypter();
      envelopeWriter.addRecipient(deviceRecipient);
      const envelope = new Uint8Array(await envelopeWriter.encrypt(encoder.encode(appIdentity)));
      const digest = [...sha256(envelope)].map(byte => byte.toString(16).padStart(2, '0')).join('');

      const rootDecryptor = new Decrypter();
      rootDecryptor.addIdentity(ageIdentity);
      const rootPlaintext = await rootDecryptor.decrypt(armor.decode(rootBackup));
      const root = JSON.parse(new TextDecoder().decode(rootPlaintext));
      const secretKey = fromBase64(root.secretKey);
      const signedPayload = {
        domain: 'rawrarr-app-key-grant-v1', schemaVersion: 1, appInstanceId,
        appType: 'radarr', appRecipient, deviceId: nodeId, deviceRecipient,
        keyEpoch: 1, encryptedIdentitySha256: digest,
      };
      const signature = ml_dsa65.sign(encoder.encode(canonicalJson(signedPayload)), secretKey);
      const grant = {
        algorithm: 'ML-DSA-65', rootPublicKey: root.publicKey,
        rootFingerprint, signedPayload, signature: toBase64(signature),
        encryptedIdentity: toBase64(envelope),
      };
      const grantBytes = encoder.encode(JSON.stringify(grant));
      await page.goto(origin);
      const firstStatus = await page.evaluate(async bundle => (await fetch('/api/workspace/app-key-grants', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(bundle),
      })).status, grant);
      const retryStatus = await page.evaluate(async bundle => (await fetch('/api/workspace/app-key-grants', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(bundle),
      })).status, grant);
      const expectedStatuses = action === 'grant' ? [201, 204] : [403, 403];
      assert.deepEqual([firstStatus, retryStatus], expectedStatuses);
      if (action === 'grant') {
        console.log(`GRANT ${appInstanceId} ${deviceRecipient} ${toBase64(encoder.encode(appIdentity))} ${toBase64(grantBytes)} ${firstStatus} ${retryStatus}`);
      } else {
        console.log(`WRONG_TARGET ${nodeId} ${firstStatus} ${retryStatus}`);
      }
      envelope.fill(0); secretKey.fill(0); rootPlaintext.fill(0); root.secretKey = ''; appIdentity = '';
      continue;
    }
    const revokeMatch = /^revoke ([a-z0-9-]+)$/.exec(command);
    if (revokeMatch) {
      await page.goto(origin);
      const status = await page.evaluate(async nodeId => (await fetch(`/api/workspace/nodes/${encodeURIComponent(nodeId)}`, { method: 'DELETE' })).status, revokeMatch[1]);
      assert.equal(status, 204);
      console.log(`REVOKED ${revokeMatch[1]} ${status}`);
      continue;
    }
    const match = /^approve ([a-z0-9-]+)$/.exec(command);
    if (!match) throw new Error('unknown test-helper command');
    const nodeId = match[1];
    await page.goto(origin);
    const signIn = page.locator('#sign-in');
    if (await signIn.isVisible().catch(() => false)) {
      await signIn.click();
      await page.waitForURL(origin, { timeout: 15000 });
    }
    await page.locator('[data-view="fossils"]').click();
    const request = page.locator('#pending .pending-item').filter({ hasText: nodeId });
    await request.waitFor({ timeout: 60000 });
    await request.getByRole('button', { name: 'Approve' }).click();
    await page.locator('#approve-node-age-id').fill(ageIdentity);
    await page.locator('#approve-node-root-backup').fill(rootBackup);
    await page.locator('#approve-node-root-fingerprint').fill(rootFingerprint);
    await page.locator('#approve-node-form [type="submit"]').click();
    try {
      await page.locator('#approve-node-dialog').waitFor({ state: 'hidden', timeout: 60000 });
    } catch {
      const status = await page.locator('#approve-node-status').innerText().catch(() => 'approval dialog did not close');
      throw new Error(`owner approval failed: ${status}`);
    }
    await request.waitFor({ state: 'detached', timeout: 30000 });
    console.log(`APPROVED ${nodeId}`);
  }
  await page.close();
} finally {
  await browser.close();
}
