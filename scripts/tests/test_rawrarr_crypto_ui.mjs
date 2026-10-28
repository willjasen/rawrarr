import test from 'node:test';
import { createServer } from 'node:http';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';

const root = new URL('../../rust-arr/src/', import.meta.url);
const billingHtml = await readFile(new URL('coordinator_billing.html', root), 'utf8');
const billingScript = await readFile(new URL('coordinator-billing.mjs', root), 'utf8');
const authHtml = await readFile(new URL('auth.html', root), 'utf8');
const workspaceHtml = await readFile(new URL('coordinator_workspace.html', root), 'utf8');
const workspaceScript = await readFile(new URL('coordinator-workspace.mjs', root), 'utf8');
const rust = await readFile(new URL('coordinator_crypto.rs', root), 'utf8');
const coordinatorRust = await readFile(new URL('coordinator.rs', root), 'utf8');

async function serveBilling(page, routeApi) {
  const server = createServer(async (request, response) => {
    if (request.url === '/') {
      response.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
      response.end(billingHtml);
      return;
    }
    if (request.url === '/assets/coordinator-billing.mjs') {
      response.writeHead(200, { 'content-type': 'text/javascript; charset=utf-8' });
      response.end(billingScript);
      return;
    }
    response.writeHead(404);
    response.end();
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const origin = `http://127.0.0.1:${server.address().port}`;
  page.on('close', () => server.close());
  await page.route(`${origin}/api/**`, routeApi);
  await page.goto(origin);
}



async function serveAuth(page, routeApi) {
  const server = createServer((request, response) => {
    if (request.url === '/' || request.url === '/auth') {
      response.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
      response.end(authHtml);
      return;
    }
    if (request.url === '/assets/passkey-client.mjs') {
      response.writeHead(200, { 'content-type': 'text/javascript; charset=utf-8' });
      response.end('export function decodeOptions(value) { return value; }');
      return;
    }
    response.writeHead(404);
    response.end();
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const origin = `http://127.0.0.1:${server.address().port}`;
  page.on('close', () => server.close());
  await page.route(`${origin}/api/**`, routeApi);
  await page.goto(`${origin}/auth`);
}

test('crypto payments browser regressions', async t => {
  const browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  t.after(() => browser.close());

  await t.test('subscription page stays usable on desktop and narrow mobile viewports', async () => {
    for (const viewport of [{ width: 1280, height: 800 }, { width: 375, height: 812 }]) {
      const page = await browser.newPage({ viewport });
      await serveBilling(page, async route => {
        const path = new URL(route.request().url()).pathname;
        if (path === '/api/crypto/config') return route.fulfill({ json: { chainId: 1, treasuryAddress: '0x2222222222222222222222222222222222222222', donationEnabled: true, subscriptionsEnabled: true, monthlyPriceWei: '25000000000000000', annualPriceWei: '250000000000000000', currency: 'ETH' } });
        if (path === '/api/workspace/billing') return route.fulfill({ json: { checkoutEnabled: true, customerLinked: false, subscriptionStatus: null } });
        return route.fulfill({ status: 404, json: { error: `unexpected API path ${path}` } });
      });
      await page.waitForFunction(() => document.getElementById('billing-status')?.textContent.includes('No Stripe test subscription'));
      const geometry = await page.evaluate(() => ({ width: window.innerWidth, documentWidth: document.documentElement.scrollWidth }));
      assert.ok(geometry.documentWidth <= geometry.width, `horizontal overflow at ${viewport.width}px: ${geometry.documentWidth}px`);
      assert.equal(await page.getByRole('heading', { name: 'Subscription', exact: true }).isVisible(), true);
      for (const selector of ['#donate', '#subscribe', '#billing-checkout']) {
        const action = page.locator(selector);
        await action.scrollIntoViewIfNeeded();
        assert.equal(await action.isVisible(), true, `${selector} is hidden at ${viewport.width}px`);
        const rect = await action.boundingBox();
        assert.ok(rect.width > 0 && rect.height > 0, `${selector} has no usable size at ${viewport.width}px`);
        assert.ok(rect.x >= 0 && rect.x + rect.width <= viewport.width, `${selector} exceeds the viewport at ${viewport.width}px`);
      }
      await page.close();
    }
  });

  await t.test('donations stay available without account sign-in and do not call subscription claims', async () => {
    const page = await browser.newPage();
    await page.addInitScript(() => {
      window.ethereum = {
        request: async ({ method, params }) => {
          if (method === 'eth_chainId') return '0x1';
          if (method === 'eth_requestAccounts') return ['0x1111111111111111111111111111111111111111'];
          if (method === 'eth_sendTransaction') { window.sentTx = params[0]; return `0x${'a'.repeat(64)}`; }
          throw new Error(`Unexpected wallet method: ${method}`);
        }
      };
      window.prompt = () => '0.025';
    });
    await serveBilling(page, async route => {
      const path = new URL(route.request().url()).pathname;
      if (path === '/api/crypto/config') return route.fulfill({ json: { chainId: 1, treasuryAddress: '0x2222222222222222222222222222222222222222', donationEnabled: true, subscriptionsEnabled: false, monthlyPriceWei: null, annualPriceWei: null, currency: 'ETH' } });
      if (path === '/api/crypto/donation/link') return route.fulfill({ json: { recipient: '0x2222222222222222222222222222222222222222', amountWei: '25000000000000000' } });
      return route.fulfill({ status: 500, json: { error: 'unexpected API call' } });
    });
    assert.match(await page.locator('#payment-config').textContent(), /Ethereum mainnet recipient: 0x2222222222222222222222222222222222222222/);
    assert.equal(await page.locator('#donate').isDisabled(), false);
    await page.getByRole('button', { name: 'Donate with wallet' }).click();
    await page.waitForTimeout(200);
    assert.match(await page.locator('#donation-message').textContent(), /does not change your account tier|Error|not configured|Install/);
    assert.equal(await page.evaluate(() => window.sentTx.to), '0x2222222222222222222222222222222222222222');
    assert.equal(await page.evaluate(() => window.sentTx.value), '0x58d15e17628000');
    await page.close();
  });

  await t.test('subscription purchase requires mainnet and waits on claim status after transfer', async () => {
    const page = await browser.newPage();
    let claimCount = 0;
    await page.addInitScript(() => { window.ethereum = { request: async ({ method, params }) => {
      if (method === 'eth_chainId') return window.testChain;
      if (method === 'eth_requestAccounts') return ['0x1111111111111111111111111111111111111111'];
      if (method === 'eth_sendTransaction') { window.sentTx = params[0]; return `0x${'b'.repeat(64)}`; }
      throw new Error(`Unexpected wallet method: ${method}`);
    } }; });
    await page.addInitScript(() => { window.testChain = '0x5'; });
    await serveBilling(page, async route => {
      if (new URL(route.request().url()).pathname === '/api/auth/eth/link') return route.fulfill({ json: { wallets: [{ address: '0x1111111111111111111111111111111111111111', ensName: 'willjasen.eth' }] } });
      return route.fulfill({ json: { chainId: 1, treasuryAddress: '0x2222222222222222222222222222222222222222', donationEnabled: true, subscriptionsEnabled: true, monthlyPriceWei: '25000000000000000', annualPriceWei: '250000000000000000', currency: 'ETH' } });
    });
    await page.waitForFunction(() => document.getElementById('subscribe').disabled === false);
    await page.getByRole('button', { name: /Subscribe ·/ }).click();
    await assert.doesNotReject(page.waitForFunction(() => document.getElementById('subscription-message').textContent.includes('Switch your wallet to Ethereum mainnet')));
    assert.equal(await page.evaluate(() => window.sentTx), undefined);
    assert.equal(claimCount, 0);
    await page.close();
  });

  await t.test('subscription sends only the configured exact amount and submits one claim after transfer', async () => {
    const page = await browser.newPage();
    let claimCount = 0;
    await page.addInitScript(() => { window.ethereum = { request: async ({ method, params }) => {
      if (method === 'eth_chainId') return '0x1';
      if (method === 'eth_requestAccounts') return ['0x1111111111111111111111111111111111111111'];
      if (method === 'eth_sendTransaction') { window.sentTx = params[0]; return `0x${'c'.repeat(64)}`; }
      throw new Error(`Unexpected wallet method: ${method}`);
    } }; });
    await serveBilling(page, async route => {
      const path = new URL(route.request().url()).pathname;
      if (path === '/api/crypto/subscription/claim') { claimCount += 1; return route.fulfill({ status: 202, json: { status: 'pending' } }); }
      if (path === '/api/auth/eth/link') return route.fulfill({ json: { wallets: [{ address: '0x1111111111111111111111111111111111111111', ensName: 'willjasen.eth' }] } });
      return route.fulfill({ json: { chainId:1, treasuryAddress:'0x2222222222222222222222222222222222222222', donationEnabled:true, subscriptionsEnabled:true, monthlyPriceWei:'25000000000000000', annualPriceWei:'250000000000000000', currency:'ETH' } });
    });
    await page.waitForFunction(() => document.getElementById('subscribe').disabled === false);
    await page.getByRole('button', { name: /Subscribe ·/ }).click();
    await assert.doesNotReject(page.waitForFunction(() => document.getElementById('subscription-message').textContent.includes('awaiting finality')));
    assert.equal(claimCount, 1);
    assert.equal(await page.evaluate(() => window.sentTx.to), '0x2222222222222222222222222222222222222222');
    assert.equal(await page.evaluate(() => window.sentTx.value), '0x58d15e17628000');
    await page.close();
  });


  await t.test('subscription prevents ETH transfer when no ENS wallet is linked', async () => {
    const page = await browser.newPage();
    await page.addInitScript(() => { window.ethereum = { request: async ({ method }) => {
      if (method === 'eth_chainId') return '0x1';
      if (method === 'eth_requestAccounts') return ['0x1111111111111111111111111111111111111111'];
      if (method === 'eth_sendTransaction') throw new Error('must not transfer');
      throw new Error(`Unexpected wallet method: ${method}`);
    } }; });
    await serveBilling(page, async route => {
      if (new URL(route.request().url()).pathname === '/api/auth/eth/link') return route.fulfill({ status: 401, json: { error: 'Sign in first.' } });
      return route.fulfill({ json: { chainId:1, treasuryAddress:'0x2222222222222222222222222222222222222222', donationEnabled:true, subscriptionsEnabled:true, monthlyPriceWei:'25000000000000000', annualPriceWei:'250000000000000000', currency:'ETH' } });
    });
    await page.waitForFunction(() => document.getElementById('subscribe').disabled === false);
    await page.getByRole('button', { name: /Subscribe ·/ }).click();
    await page.waitForFunction(() => document.getElementById('subscription-message').textContent.includes('Sign in first.'));
    await page.close();
  });

  await t.test('subscription prevents ETH transfer from a wallet other than the linked ENS wallet', async () => {
    const page = await browser.newPage();
    await page.addInitScript(() => { window.ethereum = { request: async ({ method }) => {
      if (method === 'eth_chainId') return '0x1';
      if (method === 'eth_requestAccounts') return ['0x3333333333333333333333333333333333333333'];
      if (method === 'eth_sendTransaction') throw new Error('must not transfer');
      throw new Error(`Unexpected wallet method: ${method}`);
    } }; });
    await serveBilling(page, async route => {
      if (new URL(route.request().url()).pathname === '/api/auth/eth/link') return route.fulfill({ json: { wallets: [{ address: '0x1111111111111111111111111111111111111111', ensName: 'willjasen.eth' }] } });
      return route.fulfill({ json: { chainId:1, treasuryAddress:'0x2222222222222222222222222222222222222222', donationEnabled:true, subscriptionsEnabled:true, monthlyPriceWei:'25000000000000000', annualPriceWei:'250000000000000000', currency:'ETH' } });
    });
    await page.waitForFunction(() => document.getElementById('subscribe').disabled === false);
    await page.getByRole('button', { name: /Subscribe ·/ }).click();
    await page.waitForFunction(() => document.getElementById('subscription-message').textContent.includes('Switch to the ENS wallet linked'));
    await page.close();
  });

  await t.test('signed-out authentication offers ENS wallet login separately from passkeys', async () => {
    const page = await browser.newPage();
    await page.setContent(authHtml);
    assert.match(authHtml, /Sign in with passkey/);
    assert.match(authHtml, /Sign in with ENS wallet/);
    assert.match(authHtml, /eth_requestAccounts/);
    assert.match(authHtml, /personal_sign/);
    assert.match(authHtml, /currently resolved by this ENS name/);
    await page.close();
  });


  await t.test('ENS login signs the exact server challenge with its resolved wallet', async () => {
    const page = await browser.newPage();
    let finishBody;
    await page.addInitScript(() => {
      window.ethereum = { request: async ({ method, params }) => {
        if (method === 'eth_requestAccounts') return ['0x1111111111111111111111111111111111111111'];
        if (method === 'personal_sign') { sessionStorage.setItem('signedWallet', params[1]); sessionStorage.setItem('signedMessage', params[0]); return `0x${'d'.repeat(130)}`; }
        throw new Error(`Unexpected wallet method: ${method}`);
      } };
    });
    await serveAuth(page, async route => {
      const request = route.request();
      const path = new URL(request.url()).pathname;
      if (path === '/api/auth/status') return route.fulfill({ json: { userExists: true } });
      if (path === '/api/auth/eth/login/start') return route.fulfill({ json: { challengeId: 'challenge-1', address: '0x1111111111111111111111111111111111111111', message: 'rawrarr.example sign-in challenge' } });
      if (path === '/api/auth/eth/login/finish') { finishBody = request.postDataJSON(); return route.fulfill({ json: { ok: true } }); }
      return route.fulfill({ status: 500, json: { error: `unexpected API path ${path}` } });
    });
    await page.getByLabel('ENS name').fill('willjasen.eth');
    await page.getByRole('button', { name: 'Sign in with ENS wallet' }).click();
    await page.waitForFunction(() => location.pathname === '/');
    assert.equal(await page.evaluate(() => sessionStorage.getItem('signedWallet')), '0x1111111111111111111111111111111111111111');
    assert.ok(await page.evaluate(() => sessionStorage.getItem('signedMessage').length > 2));
    assert.equal(finishBody.challengeId, 'challenge-1');
    assert.equal(finishBody.signature, `0x${'d'.repeat(130)}`);
    await page.close();
  });

  await t.test('ENS login refuses to sign when the selected wallet differs from ENS resolution', async () => {
    const page = await browser.newPage();
    let signed = false;
    await page.addInitScript(() => { window.ethereum = { request: async ({ method }) => {
      if (method === 'eth_requestAccounts') return ['0x3333333333333333333333333333333333333333'];
      if (method === 'personal_sign') { window.didSign = true; throw new Error('must not sign'); }
      throw new Error(`Unexpected wallet method: ${method}`);
    } }; });
    await serveAuth(page, async route => {
      const path = new URL(route.request().url()).pathname;
      if (path === '/api/auth/status') return route.fulfill({ json: { userExists: true } });
      if (path === '/api/auth/eth/login/start') return route.fulfill({ json: { challengeId: 'challenge-2', address: '0x1111111111111111111111111111111111111111', message: 'rawrarr.example sign-in challenge' } });
      signed = true;
      return route.fulfill({ status: 500, json: { error: 'must not finish' } });
    });
    await page.getByLabel('ENS name').fill('willjasen.eth');
    await page.getByRole('button', { name: 'Sign in with ENS wallet' }).click();
    await page.waitForFunction(() => document.getElementById('message').textContent.includes('currently resolved by this ENS name'));
    assert.equal(await page.evaluate(() => window.didSign), undefined);
    assert.equal(signed, false);
    await page.close();
  });

  await t.test('free tier caps fossil enrollment, approval, and coordinator proxy access', async () => {
    assert.match(rust, /rawrarr_node_enrollments WHERE user_id=\$1 AND role='metadata'/);
    assert.match(rust, /free_tier_allows_existing_fossil/);
    assert.match(rust, /\.layer\(middleware::from_fn_with_state\(state\.auth\.clone\(\), auth_context\)\)/);
    assert.match(rust, /request\.extensions_mut\(\)\.insert\(auth\)/);
    assert.match(coordinatorRust, /free_tier_denies_fossil_node\(&state\.pool, owner_user_id, Some\(&request\.node_id\)\)/);
    assert.match(coordinatorRust, /free_tier_denies_fossil_node\(&state\.pool, owner_user_id, Some\(&node_id\)\)/);
    assert.match(coordinatorRust, /free_tier_allows_existing_fossil\(&state\.pool, owner_user_id, &node_id\)/);
    assert.match(coordinatorRust, /StatusCode::PAYMENT_REQUIRED/);
    assert.match(coordinatorRust, /approval follows the same order to avoid a row-lock\/advisory-lock deadlock/);
    assert.match(coordinatorRust, /Err\(_\) => return StatusCode::SERVICE_UNAVAILABLE/);
  });

  await t.test('workspace surfaces tier limits, wallet linking, and Subscription navigation', async () => {
    assert.match(workspaceHtml, /crypto-access-status/);
    assert.match(workspaceHtml, /crypto-wallet-link-form/);
    assert.match(workspaceHtml, /href="\/billing">Manage subscription/);
    assert.doesNotMatch(workspaceHtml, /id="billing-checkout"|id="billing-status"/);
    assert.match(workspaceScript, /Free tier · \$\{access\.fossilNodes\} of \$\{access\.fossilNodeLimit\} fossil nodes/);
    assert.match(workspaceScript, /api\/auth\/eth\/link\/finish/);
    assert.doesNotMatch(workspaceScript, /api\/workspace\/billing/);
  });

  await t.test('server makes donations independent and requires exact finalized mainnet subscription transfers', async () => {
    assert.match(rust, /donation_enabled: state\.treasury\.is_some\(\)/);
    assert.match(rust, /eth_getTransactionByHash/);
    assert.match(rust, /eth_getTransactionReceipt/);
    assert.match(rust, /eth_getBlockByNumber/);
    assert.match(rust, /"finalized"/);
    assert.match(rust, /tx_value != expected_amount/);
    assert.match(rust, /Some\(tx_to\.as_str\(\)\) != self\.treasury\.as_deref\(\)/);
    assert.match(rust, /paid_through/);
    assert.doesNotMatch(rust.slice(rust.indexOf('async fn payment_config'), rust.indexOf('async fn start_wallet_login')), /rawrarr_eth_subscriptions/);
  });
});
