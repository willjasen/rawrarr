// Invoked only by test_rawrarr_role_deployment.py against its disposable metadata node.
import assert from 'node:assert/strict';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';
const browser = await chromium.launch({headless:true, executablePath:'/usr/bin/chromium', args:['--no-sandbox']});
try {
  const context = await browser.newContext();
  const page = await context.newPage();
  page.on('response', response => {
    if (response.url().includes('/api/auth/register/finish')) {
      console.log(`registration finish HTTP ${response.status()}`);
    }
  });
  const cdp = await context.newCDPSession(page);
  await cdp.send('WebAuthn.enable');
  await cdp.send('WebAuthn.addVirtualAuthenticator', {options:{protocol:'ctap2', transport:'internal', hasResidentKey:true, hasUserVerification:true, isUserVerified:true, automaticPresenceSimulation:true}});
  await page.goto('http://localhost:8080/auth');
  await page.locator('#display-name').fill('Fixture Owner');
  await page.locator('#create-passkey').click();
  try { await page.waitForURL('**/setup/age', {timeout:15000}); }
  catch { throw new Error(`Initial passkey setup did not continue: ${page.url()} — ${await page.locator('#message').innerText()}`); }
  assert.equal((await page.request.get('http://localhost:8080/api/devices')).status(),200);
  assert.equal((await page.request.post('http://localhost:8080/api/auth/logout')).status(),200);
  await page.goto('http://localhost:8080/auth');
  await page.locator('#sign-in').click();
  await page.waitForURL('**/setup/age', {timeout:15000});
  assert.equal((await page.request.get('http://localhost:8080/api/devices')).status(),200);
  await page.goto('http://localhost:8080/setup/age');
  await page.locator('#add-coordinator-passkey').click();
  await page.locator('#create-passkey').click();
  await page.getByText(/Passkey added to your existing Rawrarr account/).waitFor({timeout:15000});
  assert.match(page.url(), /^http:\/\/localhost:8081\/passkeys\/add$/);
  await page.goto('http://localhost:8081/');
  await page.locator('#sign-in').click();
  await page.waitForURL('http://localhost:8081/', {timeout:15000});
  await page.locator('#node-page').waitFor({state:'visible', timeout:15000});
  await page.frameLocator('#node-page').getByText(/unlock your rawrchive/i).waitFor({timeout:15000});
  assert.match(page.url(), /^http:\/\/localhost:8081\/$/);
  assert.equal(await page.locator('#node-page').getAttribute('src'), '/nodes/metadata-test/');
  console.log('PASS: real browser virtual-passkey registration, coordinator sign-in, and metadata page relay');
} finally { await browser.close(); }
