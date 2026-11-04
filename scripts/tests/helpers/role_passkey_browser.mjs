// Interactive owner-side bootstrap for the disposable role deployment test.
// Private age/root material remains in this browser process and is never printed.
import assert from 'node:assert/strict';
import readline from 'node:readline';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

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
